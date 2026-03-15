use nexus_core::parser::royalroad;

#[test]
fn test_parse_chapters_from_json() {
    let html = r#"<script>window.chapters = [{"id": 123, "title": "Chapter 1", "order": 1, "url": "/fiction/1/chapter/123"}]</script>"#;
    let result = royalroad::parse_chapters(html);
    assert_eq!(result.len(), 1);

    let first = &result[0];
    assert_eq!(first.chapter_id, Some(123));
    assert_eq!(first.title, Some("Chapter 1".to_string()));
    assert_eq!(first.chapter_number, Some(1));
    assert_eq!(first.url, Some("/fiction/1/chapter/123".to_string()));
}

#[test]
fn test_parse_description() {
    let html = r#"<div class="description">
        <div class="hidden-content">A great story about heroes.</div>
    </div>"#;
    let result = royalroad::parse_description(html);
    assert!(result.contains("heroes"));
}

#[test]
fn test_parse_tags() {
    let html = r#"<span class="tags">
        <a href="/tags/fantasy">Fantasy</a>
        <a href="/tags/action">Action</a>
    </span>"#;
    let _result = royalroad::parse_tags(html);
}

#[test]
fn test_parse_total_views() {
    let html = r#"<div class="stat-container">
        <div class="stat-value">1,234,567</div>
    </div>"#;
    let _result = royalroad::parse_total_views(html);
}

#[test]
fn test_parse_followers() {
    let html = r#"<div class="follows">
        <span class="value">500</span>
    </div>"#;
    let _result = royalroad::parse_followers(html);
}

#[test]
fn test_parse_favorites() {
    let html = r#"<div class="favorites">
        <span class="value">100</span>
    </div>"#;
    let _result = royalroad::parse_favorites(html);
}

#[test]
fn test_parse_ratings() {
    let html = r#"<div class="rating">
        <span class="value">50</span>
    </div>"#;
    let _result = royalroad::parse_ratings(html);
}
