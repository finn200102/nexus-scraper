use crate::models::{Author, Chapter, Stories, Story};
use crate::parse_date;
use chrono::{TimeZone, Utc};
use ego_tree::NodeRef;
use scraper::{node::Node, ElementRef, Html, Selector};
use std::collections::HashSet;

pub fn parse_fanfiction_chapter(html: &str, chapter_number: u32) -> Chapter {
    let document = Html::parse_document(html);

    let selector = Selector::parse("div#storytext").unwrap();
    let text = document
        .select(&selector)
        .next()
        .map(|div| format_story_text(div))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        site: "fanfiction".to_string(),
        text: Some(text),
        chapter_number: Some(chapter_number),
        ..Default::default()
    }
}

fn format_story_text(div: ElementRef<'_>) -> String {
    let paragraph_selector = Selector::parse("p").unwrap();
    let mut paragraphs: Vec<String> = div
        .select(&paragraph_selector)
        .map(|p| collect_paragraph_text(&p))
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        let mut buffer = String::new();
        collect_children_text(&div, &mut buffer);
        return normalize_paragraph(&buffer);
    }

    paragraphs.iter_mut().for_each(|p| {
        let normalized = normalize_paragraph(p);
        *p = normalized;
    });

    paragraphs.join("\n\n")
}

fn collect_paragraph_text(element: &ElementRef<'_>) -> String {
    let mut buffer = String::new();
    collect_children_text(element, &mut buffer);
    buffer
}

fn collect_children_text(element: &ElementRef<'_>, buffer: &mut String) {
    for child in element.children() {
        collect_node_text(child, buffer);
    }
}

fn collect_node_text(node: NodeRef<'_, Node>, buffer: &mut String) {
    match node.value() {
        Node::Text(text) => buffer.push_str(&text.text),
        Node::Element(element) => {
            if element.name() == "br" {
                buffer.push('\n');
                return;
            }

            for child in node.children() {
                collect_node_text(child, buffer);
            }
        }
        _ => {}
    }
}

fn normalize_paragraph(raw: &str) -> String {
    let cleaned = raw.replace("\r", "");
    cleaned
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_fanfiction_chapters(html: &str) -> Vec<Chapter> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("select#chap_select > option").unwrap();

    let mut chapters = Vec::new();
    let mut seen = HashSet::new();

    for chapter_element in document.select(&selector) {
        let chapter_number = chapter_element
            .value()
            .attr("value")
            .and_then(|number_str| number_str.parse::<u32>().ok())
            .unwrap_or(0);

        let title = chapter_element
            .text()
            .collect::<String>()
            .trim()
            .to_string();

        if !seen.insert(chapter_number) {
            continue;
        }

        chapters.push(Chapter {
            site: "fanfiction".to_string(),
            title: Some(title),
            chapter_number: Some(chapter_number),
            ..Default::default()
        });
    }

    chapters
}

/// Parses the tags of a fanfiction story
pub fn parse_tags(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);

    // Select the a inside span.tags
    let selector = Selector::parse("span.tags a").unwrap();

    let mut tags = Vec::new();

    for tag in document.select(&selector) {
        if let Some(tag_name) = tag
            .value()
            .attr("href")
            .and_then(|href| href.split('/').next_back())
            .and_then(|s| s.split('=').next_back())
        {
            tags.push(tag_name.to_string());
        }
    }

    tags
}

pub fn parse_genre(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);

    // "- language - genre1/genre2/genre3 - character1, character2 - Chapters: number - Words: number - Reviews:"
    let selector = match Selector::parse(r#"a[href*="fictionratings.com"]"#) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    document
        .select(&selector)
        .next()
        .and_then(|a| a.next_sibling())
        .and_then(|n| n.value().as_text())
        .and_then(|t| t.trim().split(" - ").find(|p| p.contains('/')))
        .map(|g| {
            g.split('/')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Parses the img url of the cover img
pub fn parse_cover(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Select the img.thumbnail inside div.cover-art-container
    let selector = Selector::parse("div#profile_top img.cimage").unwrap();

    document
        .select(&selector)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(|url| format!("https://www.fanfiction.net{url}"))
}

pub fn parse_fanfiction_stories(html: &str, author_id: u64) -> Stories {
    let document = Html::parse_document(html);

    let container_selector = Selector::parse("div#st_inside").unwrap();

    let story_selector = Selector::parse("div.z-list").unwrap();

    let mut stories = Vec::new();

    if let Some(container) = document.select(&container_selector).next() {
        for story_element in container.select(&story_selector) {
            // Extract story title from element
            let title_selector = Selector::parse("a.stitle").unwrap();

            let title = story_element
                .select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .and_then(|href| href.split('/').next_back())
                .unwrap_or("")
                .to_string();

            let story_id = story_element
                .select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .and_then(|href| href.split('/').nth(2))
                .and_then(|id_str| id_str.parse::<u64>().ok())
                .unwrap_or(0);

            stories.push(Story {
                site: "fanfiction".to_string(),
                story_name: Some(title),
                author_id: Some(author_id),
                story_id: Some(story_id),
                ..Default::default()
            });
        }
    }

    Stories { stories }
}

pub fn parse_fanfiction_stories_by_series(html: &str) -> Stories {
    let document = Html::parse_document(html);

    // Try multiple selectors to find story listings
    let selectors = ["div.z-list", "li.z-list", "div.story", ".z-list"];

    let mut selector = None;
    for s in &selectors {
        if let Ok(s) = Selector::parse(s) {
            if document.select(&s).next().is_some() {
                selector = Some(s);
                break;
            }
        }
    }

    let selector = match selector {
        Some(s) => s,
        None => {
            eprintln!(
                "Debug: No story selectors found. HTML length: {}",
                html.len()
            );
            return Stories { stories: vec![] };
        }
    };

    eprintln!(
        "Debug: Using selector, found {} elements",
        document.select(&selector).count()
    );

    let mut stories = Vec::new();

    for story_element in document.select(&selector) {
        // Extract story title from element
        let title_selector = Selector::parse("a.stitle").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let title = story_element
            .select(&title_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split('/').next_back())
            .unwrap_or("")
            .to_string();

        let story_id = story_element
            .select(&title_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split('/').nth(2))
            .and_then(|id_str| id_str.parse::<u64>().ok())
            .unwrap_or(0);

        let author_id = story_element
            .select(&a_selector)
            .nth(2)
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split("/").nth(2))
            .and_then(|id_str| id_str.parse::<u64>().ok())
            .unwrap_or(0);

        // Parse author name - find the author link (by href containing /u/)
        let author_name = story_element
            .select(&a_selector)
            .filter(|a| {
                a.value()
                    .attr("href")
                    .map(|h| h.contains("/u/"))
                    .unwrap_or(false)
            })
            .next()
            .map(|a| a.text().collect::<String>())
            .unwrap_or_else(|| "Unknown".to_string());

        // Parse img URL from the story image
        let img_url = story_element
            .select(&Selector::parse("img.cimage").ok().unwrap())
            .next()
            .and_then(|img| {
                img.value()
                    .attr("data-original")
                    .or_else(|| img.value().attr("src"))
                    .map(|s| format!("https://www.fanfiction.net{s}"))
            });

        // Parse description from div.z-indent.z-padtop (text content only, excluding nested div)
        let desc_selector = Selector::parse("div.z-indent.z-padtop").ok().unwrap();
        let description = story_element
            .select(&desc_selector)
            .next()
            .map(|div| {
                // Get text but exclude text from nested div.z-padtop2
                let nested_selector = Selector::parse("div.z-padtop2").ok().unwrap();
                let nested_text: String = div
                    .select(&nested_selector)
                    .flat_map(|e| e.text())
                    .collect();

                let full_text: String = div.text().collect();
                // Remove the nested div text from the full text
                full_text.replace(&nested_text, "").trim().to_string()
            })
            .filter(|s| !s.is_empty());

        // Parse metadata from div.z-indent.z-padtop
        let metadata_selector = Selector::parse("div.z-padtop2.xgray").unwrap();
        let metadata_text = story_element
            .select(&metadata_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        // Parse chapters
        let chapters = metadata_text
            .split("Chapters:")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.replace(',', "").parse::<u64>().ok());

        // Parse word count
        let word_count = metadata_text
            .split("Words:")
            .nth(1)
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.trim().replace(',', "").parse::<u64>().ok());

        // Parse reviews
        let reviews = metadata_text
            .split("Reviews:")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.replace(',', "").parse::<u64>().ok());

        // Parse favorites
        let favorites = metadata_text
            .split("Favs:")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.replace(',', "").parse::<u64>().ok());

        // Parse follows
        let follows = metadata_text
            .split("Follows:")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.replace(',', "").parse::<u64>().ok());

        // Parse updated date - first span with data-xutime
        let updated_date = story_element
            .select(&Selector::parse("span[data-xutime]").ok().unwrap())
            .next()
            .and_then(|e| e.value().attr("data-xutime"))
            .and_then(|timestamp| {
                chrono::DateTime::from_timestamp(timestamp.parse().ok()?, 0)
                    .map(|dt| dt.format("%b %d, %Y").to_string())
            });

        // Parse publish date - second span with data-xutime
        let publish_date = story_element
            .select(&Selector::parse("span[data-xutime]").ok().unwrap())
            .nth(1)
            .and_then(|e| e.value().attr("data-xutime"))
            .and_then(|timestamp| {
                chrono::DateTime::from_timestamp(timestamp.parse().ok()?, 0)
                    .map(|dt| dt.format("%b %d, %Y").to_string())
            });

        // Parse status (Complete or Ongoing)
        let status = if metadata_text.contains("Complete") {
            Some("Complete".to_string())
        } else if metadata_text.contains("In Progress") {
            Some("In Progress".to_string())
        } else {
            None
        };

        stories.push(Story {
            site: "fanfiction".to_string(),
            story_name: Some(title),
            author_id: Some(author_id),
            author_name: Some(author_name),
            story_id: Some(story_id),
            chapters: vec![],
            description,
            img_url,
            word_count,
            reviews,
            favorites,
            follows,
            publish_date,
            updated_date,
            status,
            chapter_count: chapters,
            url: Some(format!("https://www.fanfiction.net/s/{story_id}/")),
            ..Default::default()
        });
    }

    Stories { stories }
}

/// Parses the description of a fanfiction.net story
pub fn parse_description(html: &str) -> String {
    let document = Html::parse_document(html);
    let description_selector = Selector::parse("div#profile_top > div").unwrap();

    let description = document
        .select(&description_selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Description not found".into());
    description
}

/// Parses the story_name of a fanfiction.net story
pub fn parse_story_name(html: &str) -> String {
    let document = Html::parse_document(html);
    let title_selector = Selector::parse("div#profile_top > b").unwrap();

    let title = document
        .select(&title_selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "story_name not found".into());
    title
}

pub fn parse_author_from_story(html: &str) -> Author {
    // parse author on story site to get name and id
    let document = Html::parse_document(html);
    let author_selector = Selector::parse("div#profile_top > a").unwrap();

    let (author_name, author_id) = document
        .select(&author_selector)
        .next()
        .and_then(|a| a.value().attr("href"))
        .and_then(|part| {
            let split: Vec<_> = part.split('/').collect();
            let slug = split.last()?.to_string();
            let id = split.get(2)?.parse::<u64>().ok()?;
            Some((slug, id))
        })
        .unwrap_or_else(|| ("unknown-author".into(), 0));

    Author {
        author_name: Some(author_name),
        author_id: Some(author_id),
    }
}

pub fn parse_word_count(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    span_text
        .split("Words:")
        .nth(1)?
        .split('-')
        .next()?
        .trim()
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_reviews(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    span_text
        .split("Reviews:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_favorites(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    span_text
        .split("Favs:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_follows(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    span_text
        .split("Follows:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_chapter_count(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    if let Some(count_str) = span_text
        .split("Chapters:")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
    {
        return count_str.replace(',', "").parse().ok();
    }

    if span_text.contains("Words:") && span_text.contains("Status:") {
        return Some(1);
    }

    None
}

pub fn parse_publish_date(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    for element in document.select(&selector) {
        let text: String = element.text().collect();
        if !text.contains("Published:") {
            continue;
        }

        let nested_selector = Selector::parse("span[data-xutime]").ok()?;
        for nested in element.select(&nested_selector) {
            if let Some(timestamp_str) = nested.value().attr("data-xutime") {
                if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                    if let Some(date) = Utc.timestamp_opt(timestamp, 0).single() {
                        return Some(date.format("%Y-%m-%d").to_string());
                    }
                }
            }
        }

        let date_str = text
            .split("Published:")
            .nth(1)?
            .split('-')
            .next()?
            .trim()
            .to_string();

        return parse_date(&date_str);
    }

    None
}

pub fn parse_updated_date(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    for element in document.select(&selector) {
        let text: String = element.text().collect();
        if !text.contains("Updated:") {
            continue;
        }

        let nested_selector = Selector::parse("span[data-xutime]").ok()?;
        for nested in element.select(&nested_selector) {
            if let Some(timestamp_str) = nested.value().attr("data-xutime") {
                if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                    if let Some(date) = Utc.timestamp_opt(timestamp, 0).single() {
                        return Some(date.format("%Y-%m-%d").to_string());
                    }
                }
            }
        }

        let date_str = text
            .split("Updated:")
            .nth(1)?
            .split("Published:")
            .next()?
            .trim()
            .trim_end_matches('-')
            .trim()
            .to_string();

        return parse_date(&date_str);
    }

    None
}

pub fn parse_status(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span.xgray").ok()?;

    let span_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    span_text
        .split("Status:")
        .nth(1)?
        .split_whitespace()
        .next()?
        .to_string()
        .into()
}

pub fn is_story_not_found(html: &str) -> bool {
    let document = Html::parse_document(html);
    let panel_warning_selector = Selector::parse("div.panel_warning").ok();

    if let Some(selector) = panel_warning_selector {
        if let Some(panel) = document.select(&selector).next() {
            let text: String = panel.text().collect();
            return text.contains("Story Not Found");
        }
    }
    false
}
