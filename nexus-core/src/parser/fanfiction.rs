use scraper::{Html, Selector};
use std::collections::HashSet;
use crate::models::{Chapter, Story, Stories, Author};


pub fn parse_fanfiction_chapter(html: &str, chapter_number: u32) -> Chapter {

    let document = Html::parse_document(html);
    
    let selector = Selector::parse("div#storytext").unwrap();
    let text = document.select(&selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        site: "fanfiction".to_string(),
        text: Some(text),
        chapter_number: Some(chapter_number),
        ..Default::default()
    }
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

        if !seen.insert((chapter_number)) {
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
                site: "fanfiction".to_string(),
                story_name: Some(title),
                author_id: Some(author_id),
                story_id: Some(story_id),
                ..Default::default()
            });
        }
    }

    Stories {stories}

}

pub fn parse_fanfiction_stories_by_series(html: &str) -> Stories {

    let document = Html::parse_document(html);

    let selector = Selector::parse("div.z-list.zhover.zpointer").unwrap();

    let mut stories = Vec::new();


    for story_element in document.select(&selector) {
        // Extract story title from element
        let title_selector = Selector::parse("a.stitle").unwrap();
        let a_selector = Selector::parse("a").unwrap();

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

        let author_id = story_element
            .select(&a_selector)
            .nth(2)
            .and_then(|e| e.value().attr("href"))
            .and_then(|href| href.split("/").nth(2))
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

    Stories {stories}

}



pub fn parse_author_from_story (html: &str) -> Author {
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
