use scraper::{Html, Selector};
use std::collections::HashSet;
use crate::models::{Chapter, Story, Stories, Author};


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

