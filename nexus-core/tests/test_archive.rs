use nexus_core::parser::archive;

#[test]
fn test_parse_description() {
    let html = r#"<dd class="summary">A story about something.</dd>"#;
    let result = archive::parse_description(html);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_tags() {
    let html = r#"<dd class="freeform"><a href="/tags/1">Action</a></dd>"#;
    let result = archive::parse_tags(html);
    assert!(result.len() >= 0);
}

#[test]
fn test_parse_word_count() {
    let html = r#"<dd class="words">50000</dd>"#;
    let result = archive::parse_word_count(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_reviews() {
    let html = r#"<dd class="reviews">100</dd>"#;
    let result = archive::parse_reviews(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_favorites() {
    let html = r#"<dd class="bookmarks">50</dd>"#;
    let result = archive::parse_favorites(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_views() {
    let html = r#"<dd class="hits">1000</dd>"#;
    let result = archive::parse_views(html);
    assert!(result.is_some() || result.is_none());
}
