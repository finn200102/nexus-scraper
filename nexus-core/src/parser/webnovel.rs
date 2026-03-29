use crate::models::{Author, Chapter};
use regex::Regex;
use scraper::{Html, Selector};

pub fn parse_story_name(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    if let Some(el) = document
        .select(&Selector::parse("h1.pt4.pb4.pr4.oh.mb4.fs36.lh40").ok()?)
        .next()
    {
        return Some(el.text().collect::<String>().trim().to_string());
    }

    if let Some(el) = document
        .select(&Selector::parse("h1.pt4.pb4.oh.mb4.auto_height.fs36.lh40").ok()?)
        .next()
    {
        return Some(el.text().collect::<String>().trim().to_string());
    }

    None
}

pub fn parse_cover(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("i.g_thumb img").ok()?;

    let src = document.select(&selector).next()?.value().attr("src")?;

    if src.starts_with("//") {
        Some(format!("https:{src}"))
    } else {
        Some(src.to_string())
    }
}

pub fn parse_author(html: &str) -> Option<Author> {
    let document = Html::parse_document(html);

    if let Some(element) = document
        .select(&Selector::parse("address a.c_primary").ok()?)
        .next()
    {
        let author_name = element.text().collect::<String>().trim().to_string();
        if let Some(href) = element.value().attr("href") {
            if let Ok(author_id) = href.split('/').next_back()?.parse::<u64>() {
                return Some(Author {
                    author_name: Some(author_name),
                    author_id: Some(author_id),
                });
            }
        }
    }

    if let Some(div) = document
        .select(&Selector::parse("address div.ell.dib.vam.fs16.fw500").ok()?)
        .next()
    {
        let author_name = div.text().collect::<String>();
        if let Some(start) = author_name.find("Author:") {
            let name = author_name[start + 7..].trim().to_string();
            if !name.is_empty() {
                return Some(Author {
                    author_name: Some(name),
                    author_id: None,
                });
            }
        }
    }

    None
}

pub fn parse_description(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.j_synopsis").ok()?;

    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| s.trim().to_string())
}

pub fn parse_tags(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = match Selector::parse("div.m-tags a.fs12") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    document
        .select(&selector)
        .filter_map(|el| {
            let title = el.value().attr("title")?;
            let tag = title.trim();
            if tag.is_empty() {
                None
            } else {
                Some(tag.to_string())
            }
        })
        .collect()
}

pub fn parse_chapter_count(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);

    let selector = Selector::parse("div.det-hd-detail strong").ok()?;
    for strong in document.select(&selector) {
        let text = strong.text().collect::<String>();
        if !text.to_lowercase().contains("chapter") {
            continue;
        }

        if let Some(span) = strong.select(&Selector::parse("span").ok()?).next() {
            if let Some(value) = parse_number(&span.text().collect::<String>()) {
                return Some(value);
            }
        }

        if let Some(value) = parse_number(&text) {
            return Some(value);
        }
    }

    None
}

pub fn parse_views(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);

    for strong in document.select(&Selector::parse("div.det-hd-detail strong").ok()?) {
        let text = strong.text().collect::<String>();
        if text.contains("View") {
            if let Some(span) = strong.select(&Selector::parse("span").ok()?).next() {
                let span_text = span.text().collect::<String>();
                return parse_number(&span_text);
            }
        }
    }
    None
}

pub fn parse_rating(html: &str) -> Option<f64> {
    let document = Html::parse_document(html);

    if let Some(strong) = document
        .select(&Selector::parse("span._score strong.fs24").ok()?)
        .next()
    {
        let text = strong.text().collect::<String>();
        if let Ok(rating) = text.trim().parse::<f64>() {
            return Some(rating);
        }
    }

    if let Some(small) = document
        .select(&Selector::parse("p.g_star_num small").ok()?)
        .next()
    {
        let text = small.text().collect::<String>();
        if let Ok(rating) = text.trim().parse::<f64>() {
            return Some(rating);
        }
    }

    None
}

pub fn parse_rating_count(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("span._score small").ok()?;

    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .and_then(|s| {
            let re = Regex::new(r"\(([\d,]+)\s*ratings?\)").ok()?;
            let caps = re.captures(&s)?;
            let num = caps.get(1)?.as_str();
            let num = num.replace(',', "");
            num.parse::<u64>().ok()
        })
}

pub fn parse_reviews(html: &str) -> Option<u64> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("i.j_total_book_review").ok()?;

    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| s.replace(',', ""))
        .and_then(|s| s.parse::<u64>().ok())
}

pub fn parse_chapter_title(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.cha-tit h1").ok()?;

    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| s.trim().to_string())
}

pub fn parse_chapter_content(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.cha-content div.j_paragraph p").ok()?;

    let paragraphs: Vec<String> = document
        .select(&selector)
        .map(|el| el.text().collect::<String>())
        .collect();

    if paragraphs.is_empty() {
        None
    } else {
        Some(paragraphs.join("\n\n"))
    }
}

fn parse_number(text: &str) -> Option<u64> {
    let text = text.trim();

    let multiplier: u64 = if text.contains('M') {
        1_000_000
    } else if text.contains('K') {
        1_000
    } else {
        1
    };

    let re = Regex::new(r"([\d,\.]+)").ok()?;
    let caps = re.captures(text)?;
    let num_str = caps.get(1)?.as_str();
    let num_str = num_str.replace(',', "");
    let num: f64 = num_str.parse().ok()?;

    Some((num * multiplier as f64) as u64)
}

pub fn parse_catalog(html: &str) -> Vec<Chapter> {
    let document = Html::parse_document(html);
    let mut chapters = Vec::new();
    let mut global_chapter_number: u32 = 1;

    for li in document.select(&Selector::parse("li[data-cid]").ok().unwrap()) {
        let chapter_id = li
            .value()
            .attr("data-cid")
            .and_then(|s| s.parse::<u64>().ok());

        let chapter_number = global_chapter_number;
        global_chapter_number += 1;

        let title = li
            .select(&Selector::parse("strong").ok().unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        let href = li
            .select(&Selector::parse("a").ok().unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.to_string());

        chapters.push(Chapter {
            site: "webnovel".to_string(),
            title,
            chapter_number: Some(chapter_number),
            chapter_id,
            text: None,
            url: href,
        });
    }

    chapters
}

pub fn extract_story_id_from_url(url: &str) -> Option<u64> {
    let parts: Vec<&str> = url.split('/').collect();
    for part in parts {
        if part.contains('_') {
            if let Some(id) = part.split('_').next_back() {
                return id.parse::<u64>().ok();
            }
        }
    }
    None
}

pub fn parse_search_results(html: &str) -> crate::models::Stories {
    use crate::models::Story;
    let document = Html::parse_document(html);
    let selector = Selector::parse("li.pr.pb20.mb12").ok().unwrap();

    let mut stories = Vec::new();

    for li in document.select(&selector) {
        // Extract story ID from data-bookid attribute on the anchor tag
        let story_id = li
            .select(&Selector::parse("a.g_thumb").ok().unwrap())
            .next()
            .and_then(|el| el.value().attr("data-bookid"))
            .and_then(|s| s.parse::<u64>().ok());

        // Extract story name from h3
        let story_name = li
            .select(&Selector::parse("h3.g_h3 a").ok().unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        // Extract URL
        let url = li
            .select(&Selector::parse("h3.g_h3 a").ok().unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| format!("https://www.webnovel.com{}", s));

        // Extract author name
        let author_name = li
            .select(&Selector::parse("p.g_tags + p + p a").ok().unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        // Extract author ID from href like /profile/123456
        let author_id = li
            .select(&Selector::parse("p.g_tags + p + p a").ok().unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .and_then(|href| href.split('/').last())
            .and_then(|s| s.parse::<u64>().ok());

        // Extract description
        let description = li
            .select(&Selector::parse("p.fs16.c_000").ok().unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|s| s.trim().to_string());

        // Extract tags
        let tags: Vec<String> = li
            .select(&Selector::parse("p.g_tags a").ok().unwrap())
            .map(|el| el.text().collect::<String>())
            .collect();

        // Extract cover image
        let img_url = li
            .select(&Selector::parse("a.g_thumb img").ok().unwrap())
            .next()
            .and_then(|el| el.value().attr("src"))
            .map(|s| {
                if s.starts_with("//") {
                    format!("https:{}", s)
                } else {
                    s.to_string()
                }
            });

        // Extract rating
        let rating = li
            .select(&Selector::parse("p.g_star_num small").ok().unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .and_then(|s| s.trim().parse::<f64>().ok());

        // Chapter count - not available in search results
        let chapter_count = None;

        let story = Story {
            site: "webnovel".to_string(),
            story_id,
            story_name,
            author_id,
            author_name,
            description,
            img_url,
            tags,
            chapter_count,
            url,
            rating,
            ..Default::default()
        };

        stories.push(story);
    }

    crate::models::Stories { stories }
}

#[derive(Debug, serde::Deserialize)]
struct ApiVolumeItem {
    #[serde(rename = "volumeName", default)]
    volume_name: Option<String>,
    #[serde(rename = "chapterItems", default)]
    chapter_items: Vec<ApiChapterItem>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiChapterItem {
    #[serde(rename = "id", default)]
    id: Option<u64>,
    #[serde(rename = "chapterName", default)]
    chapter_name: Option<String>,
    #[serde(rename = "chapterLevel", default)]
    chapter_level: Option<u32>,
    #[serde(rename = "index", default)]
    index: Option<u32>,
    #[serde(rename = "chapterIndex", default)]
    chapter_index: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
struct ChapterListResponse {
    #[serde(rename = "code", default)]
    code: Option<i32>,
    #[serde(rename = "msg", default)]
    msg: Option<String>,
    #[serde(rename = "data", default)]
    data: Option<ChapterListData>,
}

#[derive(Debug, serde::Deserialize)]
struct ChapterListData {
    #[serde(rename = "volumeItems", default)]
    volume_items: Vec<ApiVolumeItem>,
}

pub fn parse_chapter_list_api(json: &str) -> Vec<Chapter> {
    let response: ChapterListResponse = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let data = match response.data {
        Some(d) => d,
        None => return vec![],
    };

    let mut chapters = Vec::new();
    let mut global_chapter_number: u32 = 1;

    for volume in data.volume_items {
        for chapter_item in volume.chapter_items {
            if chapter_item.chapter_level.unwrap_or(0) > 0 {
                continue;
            }

            let chapter_id = chapter_item
                .id
                .or_else(|| chapter_item.chapter_index.map(|i| i as u64));

            let chapter_number = global_chapter_number;
            global_chapter_number += 1;

            chapters.push(Chapter {
                site: "webnovel".to_string(),
                title: chapter_item.chapter_name,
                chapter_number: Some(chapter_number),
                chapter_id,
                text: None,
                url: None,
            });
        }
    }

    chapters
}

pub fn has_more_chapters(json: &str) -> bool {
    let response: ChapterListResponse = match serde_json::from_str(json) {
        Ok(r) => r,
        Err(_) => return false,
    };

    match response.data {
        Some(data) => !data.volume_items.is_empty(),
        None => false,
    }
}
