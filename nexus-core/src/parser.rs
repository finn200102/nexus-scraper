use scraper::{Html, Selector};
use crate::models::{Chapter, Story, Stories};


pub fn parse_fanfiction_chapter(html: &str, chapter_number: u32) -> Chapter {

    let document = Html::parse_document(html);
    
    let selector = Selector::parse("div#storytext").unwrap();
    let text = document.select(&selector)
        .next()
        .map(|div| div.text().collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|| "Chapter not found".into());

    Chapter {
        title: "Chapter".into(),
        text,
        chapter_number,
        chapter_id: u64::MAX,
    }
}

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
        title: "Chapter".into(),
        text,
        chapter_id,
        chapter_number,
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
            title: "Chapter".into(),
            text: "".into(),
            chapter_id,
            chapter_number: u32::MAX,
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
            title,
            author: String::new(),
            author_name: author_name.to_string(),
            author_id: u64::MAX,
            story_id,
            chapters: Vec::new(),
        });
}

    Stories {stories}


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
                author_name: String::new(),
                story_id,
                chapters: Vec::new(),
            });
        }
    }

    Stories {stories}

}
