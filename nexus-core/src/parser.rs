use scraper::{Html, Selector};
use crate::models::{Chapter, Story, Stories};

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
                .and_then(|href| href.split('/').last())
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
                title,
                author: String::new(),
                author_id,
                story_id,
                chapters: Vec::new(),
            });
        }
    }

    Stories {stories}

}
