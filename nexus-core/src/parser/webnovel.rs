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
        Some(format!("https:{}", src))
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
            if let Some(author_id) = href.split('/').last()?.parse::<u64>().ok() {
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

    for strong in document.select(&Selector::parse("div.det-hd-detail strong").ok()?) {
        let text = strong.text().collect::<String>();
        if text.contains("Chapter") {
            if let Some(span) = strong.select(&Selector::parse("span").ok()?).next() {
                let span_text = span.text().collect::<String>();
                return parse_number(&span_text);
            }
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
        });
    }

    chapters
}

pub fn extract_story_id_from_url(url: &str) -> Option<u64> {
    let parts: Vec<&str> = url.split('/').collect();
    for part in parts {
        if part.contains('_') {
            if let Some(id) = part.split('_').last() {
                return id.parse::<u64>().ok();
            }
        }
    }
    None
}
