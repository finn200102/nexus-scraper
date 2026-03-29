use nexus_core::parser::webnovel;

#[test]
fn test_parse_story_name() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_story_name(html);
    assert_eq!(result, Some("Endless Path : Infinite Cosmos".to_string()));
}

#[test]
fn test_parse_author() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_author(html).unwrap();
    assert_eq!(result.author_name, Some("Einlion".to_string()));
}

#[test]
fn test_parse_description() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_description(html);
    assert!(result.is_some());
    assert!(result.unwrap().starts_with("Vahn was an atypical youth"));
}

#[test]
fn test_parse_tags() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_tags(html);
    assert!(result.len() > 0);
    assert!(result.contains(&"System Stories".to_string()));
}

#[test]
fn test_parse_chapter_count() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_chapter_count(html);
    assert_eq!(result, Some(2365));
}

#[test]
fn test_parse_views() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_views(html);
    assert!(result.is_some());
}

#[test]
#[ignore = "rating selector needs verification with real HTML"]
fn test_parse_rating() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_rating(html);
    assert!(result.is_some());
}

#[test]
fn test_parse_cover() {
    let html = include_str!("fixtures/webnovel/story.html");
    let result = webnovel::parse_cover(html);
    assert!(result.is_some());
    assert!(result.unwrap().starts_with("https://book-pic.webnovel.com"));
}

#[test]
fn test_parse_catalog() {
    let html = include_str!("fixtures/webnovel/catalog.html");
    let result = webnovel::parse_catalog(html);
    assert!(result.len() > 0);

    let first = result.first().unwrap();
    assert_eq!(first.site, "webnovel");
    assert!(first.chapter_number.is_some());
    assert!(first.chapter_id.is_some());
    assert!(first.url.is_some());
    assert!(first.url.as_ref().unwrap().starts_with("/book/"));
}

#[test]
fn test_parse_chapter_title() {
    let html = include_str!("fixtures/webnovel/chapter.html");
    let result = webnovel::parse_chapter_title(html);
    assert!(result.is_some());
}

#[test]
fn test_parse_chapter_content() {
    let html = include_str!("fixtures/webnovel/chapter.html");
    let result = webnovel::parse_chapter_content(html);
    assert!(result.is_some());
    let content = result.unwrap();
    assert!(content.len() > 100);
    assert!(content.contains("December 31st, 1999"));
}

#[test]
fn test_extract_story_id_from_url() {
    let url = "https://www.webnovel.com/book/endless-path-infinite-cosmos_11766562205519505";
    let result = webnovel::extract_story_id_from_url(url);
    assert_eq!(result, Some(11766562205519505));
}

#[test]
fn test_parse_chapter_list_api() {
    let json = include_str!("fixtures/webnovel/chapter_list_api.json");
    let result = webnovel::parse_chapter_list_api(json);
    assert_eq!(result.len(), 3);

    let first = result.first().unwrap();
    assert_eq!(first.site, "webnovel");
    assert_eq!(first.chapter_id, Some(31586142793028464));
    assert_eq!(
        first.title,
        Some("The Beginning of the End. Part 1/2".to_string())
    );
    assert_eq!(first.chapter_number, Some(1));

    let second = result.get(1).unwrap();
    assert_eq!(second.chapter_id, Some(31587736511109142));
    assert_eq!(second.chapter_number, Some(2));

    let third = result.get(2).unwrap();
    assert_eq!(third.chapter_id, Some(31591153677686381));
    assert_eq!(third.chapter_number, Some(3));
}

#[test]
fn test_has_more_chapters_true() {
    let json = include_str!("fixtures/webnovel/chapter_list_api.json");
    assert!(webnovel::has_more_chapters(json));
}

#[test]
fn test_parse_chapter_list_api_empty_response() {
    let json = r#"{"code": 0, "msg": "Success", "data": {"volumeItems": []}}"#;
    let result = webnovel::parse_chapter_list_api(json);
    assert!(result.is_empty());
}

#[test]
fn test_has_more_chapters_false() {
    let json = r#"{"code": 0, "msg": "Success", "data": {"volumeItems": []}}"#;
    assert!(!webnovel::has_more_chapters(json));
}

#[test]
fn test_has_more_chapters_invalid_json() {
    let json = "not valid json";
    assert!(!webnovel::has_more_chapters(json));
}
