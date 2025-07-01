use scraper::{Html, Selector};
use crate::models::{Chapter, Story, Stories};

pub fn parse_spacebattles_pages(html: &str) -> u32 {
    let document = Html::parse_document(html);

    let ul_selector = Selector::parse("ul.pageNav-main").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let last_page = document
        .select(&ul_selector)
        .next()
        .and_then(|ul| ul.select(&li_selector).last())
        .and_then(|li| li.select(&a_selector).next())
        .and_then(|a| a.value().attr("href"))
        .and_then(|href| {
            href.split('/')
                .filter(|s| !s.is_empty())
                .last()
                .and_then(|segment| segment.split('-').last())
                .and_then(|s| s.parse::<u32>().ok())
        })
        .unwrap_or(1);

    last_page
}


pub fn parse_spacebattles_chapters(html: &str) -> Vec<Chapter> {
    let document = Html::parse_document(html);
    let post_selector = Selector::parse("article.message--post").unwrap();
    let title_selector = Selector::parse("span.threadmarkLabel").unwrap();
    let body_selector = Selector::parse("div.bbWrapper").unwrap();

    let mut chapters = Vec::new();

    for chapter_element in document.select(&post_selector) {
        let chapter_id = chapter_element
            .value()
            .id()
            .and_then(|id| id.strip_prefix("js-post-"))
            .and_then(|post_id| post_id.parse::<u64>().ok());

        if let Some(chapter_id) = chapter_id {
            let title = chapter_element
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_else(|| "Untitled Chapter".into());

            let text = chapter_element
                .select(&body_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();

            chapters.push(Chapter {
                site: "spacebattles".to_string(),
                title: Some(title),
                chapter_id: Some(chapter_id),
                ..Default::default()
            });
        } else {
            eprintln!("Warning: Chapter element missing valid ID");
        }
    }

    chapters
}


pub fn parse_spacebattles_chapter(html: &str, chapter_id: u64) -> Chapter {

    let document = Html::parse_document(html);
    let article_selector = format!("article#js-post-{}", chapter_id);
    let article = Selector::parse(&article_selector).unwrap();
    let title_selector = Selector::parse("span.threadmarkLabel").unwrap();
    let text_selector = Selector::parse("div.bbWrapper").unwrap();

    let article_element = document.select(&article).next();

    let text = article_element
        .and_then(|el| el.select(&text_selector).next())
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();

    let title = article_element
        .and_then(|el| el.select(&title_selector).next())
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Untitled Chapter".into());

    Chapter {
        site: "spacebattles".to_string(),
        title: Some(title),
        text: Some(text),
        chapter_id: Some(chapter_id),
        ..Default::default()
    }
}
pub fn parse_spacebattles_stories(html: &str) -> Stories {
    let document = Html::parse_document(html);

    let story_selector = Selector::parse("div.structItem-cell--main").unwrap();
    let title_selector = Selector::parse("div.structItem-title > a").unwrap();
    let author_selector = Selector::parse("ul.structItem-parts > li > a.username").unwrap();

    let mut stories = Vec::new();

    for story_element in document.select(&story_selector) {
        // Thread title and ID
        let (title_slug, story_id) = story_element
            .select(&title_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|href| href.split('/').nth(2)) 
            .and_then(|part| {
                let mut split = part.split('.');
                let slug = split.next()?.to_string();
                let id = split.next()?.parse::<u64>().ok()?;
                Some((slug, id))
            })
            .unwrap_or_else(|| ("unknown-title".into(), 0));

        // Author name and ID
        let (author_slug, author_id) = story_element
            .select(&author_selector)
            .next()
            .and_then(|a| a.value().attr("href"))
            .and_then(|href| href.split('/').nth(2)) 
            .and_then(|part| {
                let mut split = part.split('.');
                let slug = split.next()?.to_string();
                let id = split.next()?.parse::<u64>().ok()?;
                Some((slug, id))
            })
            .unwrap_or_else(|| ("unknown-author".into(), 0));

        stories.push(Story {
            site: "spacebattles".to_string(),
            title: Some(title_slug),
            author_name: Some(author_slug),
            author_id: Some(author_id),
            story_id: Some(story_id),
            ..Default::default()
        });
    }

    Stories { stories }
}

