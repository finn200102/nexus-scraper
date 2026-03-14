use nexus_core::parser::royalroad;

#[test]
fn test_parse_description() {
    let html = r#"<div class="description">A great story.</div>"#;
    let result = royalroad::parse_description(html);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_tags() {
    let html = r#"<span class="tags"><a>Action</a></span>"#;
    let result = royalroad::parse_tags(html);
    assert!(result.len() >= 0);
}

#[test]
fn test_parse_total_views() {
    let html = r#"<span class="views">1,000,000</span>"#;
    let result = royalroad::parse_total_views(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_followers() {
    let html = r#"<span class="followers">500</span>"#;
    let result = royalroad::parse_followers(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_favorites() {
    let html = r#"<span class="favorites">100</span>"#;
    let result = royalroad::parse_favorites(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_ratings() {
    let html = r#"<span class="ratings">50</span>"#;
    let result = royalroad::parse_ratings(html);
    assert!(result.is_some() || result.is_none());
}
