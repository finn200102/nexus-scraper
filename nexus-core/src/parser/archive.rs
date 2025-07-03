use scraper::{Html, Selector};
use crate::models::{Chapter, Story, Stories, Author};

pub fn parse_archive_chapter(html: &str, chapter_id: u64) -> Chapter {

    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"div#chapters > div > div[role="article"]"#).unwrap();
    let chapter_number_selector = Selector::parse("div#chapters > div").unwrap();

    let chapter_number = document.select(&chapter_number_selector)
        .next()
        .and_then(|e| e.value().attr("id"))
        .and_then(|id_str| id_str.split('-').last())
        .and_then(|number_str| number_str.parse::<u32>().ok())
        .unwrap_or(0);
    
    let text = document.select(&selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        site: "archive".to_string(),
        text: Some(text),
        chapter_id: Some(chapter_id),
        chapter_number: Some(chapter_number),
        ..Default::default()
    }
}


pub fn parse_archive_chapters(html: &str) -> Vec<Chapter> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div#main > ol > li").unwrap();

    let mut chapters = Vec::new();

    for chapter_element in document.select(&selector) {
        let chapter_selector = Selector::parse("a").unwrap();
        
        let chapter_id = chapter_element
            .select(&chapter_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split('/').last())
            .and_then(|id_str| id_str.parse::<u64>().ok())
            .unwrap_or(0);

        chapters.push(Chapter {
            site: "archive".to_string(),
            chapter_id: Some(chapter_id),
            ..Default::default()
        });
    }

    chapters
}



pub fn parse_archive_stories(html: &str, author_name: &str) -> Stories {

    let document = Html::parse_document(html);

    let story_selector = Selector::parse("div#main > ul > li").unwrap();

    let mut stories = Vec::new();

    for story_element in document.select(&story_selector) {
        // Extract story title from element
        let title_selector = Selector::parse("div h4 a").unwrap();

        let title = story_element
            .select(&title_selector)
            .next()
            .map(|a| a.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_else(|| "Story title not found".into());
            

        let story_id = story_element
            .select(&title_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split('/').last())
            .and_then(|id_str| id_str.parse::<u64>().ok())
            .unwrap_or(0);


        stories.push(Story {
            site: "archive".to_string(),
            story_name: Some(title),
            author_name: Some(author_name.to_string()),
            story_id: Some(story_id),
            ..Default::default()
        });
}

    Stories {stories}


}

pub fn parse_author_from_story (html: &str) -> Author {
    // parse author on story site to get name and id
    let document = Html::parse_document(html);
    let author_selector = Selector::parse("h3.heading > a").unwrap();

    let author_name = document 
            .select(&author_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|part| {
                let split: Vec<_> = part.split('/').collect();
                let slug = split.last()?.to_string();
                Some(slug)
            })
            .unwrap_or_else(|| "unknown-author".into());

    Author {
        author_name: Some(author_name),
        ..Default::default()
    }
}
