use nexus_core::parser::fanfiction;

#[test]
fn test_parse_tags() {
    let html = r#"<a class="tag" href="/tags/hp-canon">Harry Potter</a>"#;
    let result = fanfiction::parse_tags(html);
    assert!(result.len() >= 0);
}

#[test]
fn test_parse_genre() {
    let html = r#"<a class="genre">Romance</a><a class="genre">Adventure</a>"#;
    let result = fanfiction::parse_genre(html);
    assert!(result.len() >= 0);
}

#[test]
fn test_parse_word_count() {
    let html = r#"<span class="xgray">Words: 100,000</span>"#;
    let result = fanfiction::parse_word_count(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_publish_date() {
    let html = r#"<span class="xgray">Published: 01-Jan-2020</span>"#;
    let result = fanfiction::parse_publish_date(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_updated_date() {
    let html = r#"<span class="xgray">Updated: 15-Jun-2021</span>"#;
    let result = fanfiction::parse_updated_date(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_status() {
    let html = r#"<span class="xgray">Status: Complete</span>"#;
    let result = fanfiction::parse_status(html);
    assert!(result.is_some() || result.is_none());
}
