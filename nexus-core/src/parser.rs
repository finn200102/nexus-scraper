use scraper::{Html, Selector};
use crate::models::Chapter;

pub fn parse_fanfiction_chapter(html: &str) -> Chapter {

    let document = Html::parse_document(html);
    
    let selector = Selector::parse("div#storytext").unwrap();
    let text = document.select(&selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        title: "Chapter".into(),
        text,
    }
}
