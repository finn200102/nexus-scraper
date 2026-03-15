use crate::models::{Author, Chapter};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

pub fn parse_chapter(html: &str, chapter_id: u64) -> Chapter {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.chapter-content").unwrap();
    let text = document
        .select(&selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        site: "royalroad".to_string(),
        text: Some(text),
        chapter_id: Some(chapter_id),
        ..Default::default()
    }
}

#[derive(Debug, Deserialize)]
struct RawChapter {
    id: u64,
    title: String,
    order: u32,
    url: String,
}

pub fn parse_chapters(html: &str) -> Vec<Chapter> {
    let re = Regex::new(r"window\.chapters\s*=\s*(\[[^\]]+\])").unwrap();

    let Some(caps) = re.captures(html) else {
        return vec![];
    };

    let json_str = caps.get(1).unwrap().as_str();

    let parsed: Result<Vec<RawChapter>, _> = serde_json::from_str(json_str);
    match parsed {
        Ok(raw_chapters) => raw_chapters
            .into_iter()
            .map(|rc| Chapter {
                site: "royalroad".to_string(),
                title: Some(rc.title),
                text: None,
                chapter_number: Some(rc.order),
                chapter_id: Some(rc.id),
                url: Some(rc.url),
            })
            .collect(),
        Err(_) => vec![],
    }
}

/// Parses the description of a royalroad story
pub fn parse_description(html: &str) -> String {
    let document = Html::parse_document(html);

    // Select the div.hidden-content inside div.description
    let selector = Selector::parse("div.description div.hidden-content").unwrap();

    if let Some(root) = document.select(&selector).next() {
        root.text()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "Description not found".into()
    }
}

/// Parses the tags of a royalroad story
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

/// Parses the img url of the cover img
pub fn parse_cover(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Select the img.thumbnail inside div.cover-art-container
    let selector = Selector::parse("div.cover-art-container img.thumbnail").unwrap();

    document
        .select(&selector)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(String::from)
}

pub fn parse_author_from_story(html: &str) -> Author {
    // parse author on story site to get name and id
    let document = Html::parse_document(html);
    let author_selector = Selector::parse("h4 > span > a").unwrap();

    let author_element = document.select(&author_selector).next();

    let author_id = author_element
        .and_then(|a| a.value().attr("href"))
        .and_then(|part| {
            let split: Vec<_> = part.split('/').collect();
            split.get(2)?.parse::<u64>().ok()
        })
        .unwrap_or(0);

    let author_name = author_element
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "unkown author".to_string());

    Author {
        author_name: Some(author_name),
        author_id: Some(author_id),
    }
}

pub fn parse_total_views(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.stats-content").ok()?;

    let stats_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    stats_text
        .split("Total Views :")
        .nth(1)?
        .split("Average Views :")
        .next()?
        .trim()
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_followers(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.stats-content").ok()?;

    let stats_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    stats_text
        .split("Followers :")
        .nth(1)?
        .split("Favorites :")
        .next()?
        .trim()
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_favorites(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.stats-content").ok()?;

    let stats_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    stats_text
        .split("Favorites :")
        .nth(1)?
        .split("Ratings :")
        .next()?
        .trim()
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_ratings(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.stats-content").ok()?;

    let stats_text = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>();

    stats_text
        .split("Ratings :")
        .nth(1)?
        .split("Pages")
        .next()?
        .trim()
        .replace(',', "")
        .parse()
        .ok()
}

pub fn parse_word_count_from_pages(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);

    // Look for the popover that contains the word count info
    let selector = Selector::parse("i.popovers").ok()?;

    for element in document.select(&selector) {
        let data_content = element.value().attr("data-content")?;
        if data_content.contains("words") {
            // Extract word count from text like "calculated from 806,306 words."
            let words_part = data_content.split("from").nth(1)?;
            let word_count_str = words_part.split('w').next()?.trim().replace(',', "");
            return word_count_str.parse().ok();
        }
    }

    // Fallback: try to get pages and multiply by 275
    let stats_selector = Selector::parse("div.stats-content").ok()?;
    let stats_text = document
        .select(&stats_selector)
        .next()?
        .text()
        .collect::<String>();

    let pages: u64 = stats_text
        .split("Pages")
        .nth(1)?
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()?;

    Some(pages * 275)
}

pub fn parse_overall_score(html: &str) -> Option<f64> {
    let document = Html::parse_document(html);

    // Look for the span with "Overall Score" in the data-original-title attribute
    let selector = Selector::parse(r#"span[data-original-title="Overall Score"]"#).ok()?;

    let element = document.select(&selector).next()?;

    // Get the score from data-content attribute
    let data_content = element.value().attr("data-content")?;
    let score_str = data_content.split('/').next()?.trim();
    score_str.parse().ok()
}
