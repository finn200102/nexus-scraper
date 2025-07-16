use scraper::{Html, Selector};
use std::collections::HashSet;
use crate::models::{Chapter, Story, Stories, Author};
use regex::Regex;
use serde::{Deserialize};
use serde_json::Value;



pub fn parse_chapter(html: &str, chapter_id: u64) -> Chapter {

    let document = Html::parse_document(html);
    let selector = Selector::parse("div.chapter-content").unwrap();
    let text = document.select(&selector)
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
            })
            .collect(),
        Err(_) => vec![],
    }
}

pub fn parse_author_from_story (html: &str) -> Author {
    // parse author on story site to get name and id
    let document = Html::parse_document(html);
    let author_selector = Selector::parse("h4 > span > a").unwrap();

    let author_element = document
        .select(&author_selector)
        .next();

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
