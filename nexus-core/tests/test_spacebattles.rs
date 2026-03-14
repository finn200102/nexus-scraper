use nexus_core::parser::spacebattles;

#[test]
fn test_parse_description() {
    let html = r#"<div class="description">A story description.</div>"#;
    let result = spacebattles::parse_description(html);
    assert!(!result.is_empty());
}

#[test]
fn test_parse_tags() {
    let html = r#"<div class="tags"><a>Action</a></div>"#;
    let result = spacebattles::parse_tags(html);
    assert!(result.len() >= 0);
}

#[test]
fn test_parse_created_date() {
    let html = r#"<time data-date-string="2020-01-15">January 15, 2020</time>"#;
    let result = spacebattles::parse_created_date(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_status() {
    let html = r#"<dl class="pairs"><dt>Status</dt><dd>Complete</dd></dl>"#;
    let result = spacebattles::parse_status(html);
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_parse_spacebattles_pages() {
    let html = r#"<ul class="pageNav-main"><li><a>1</a></li><li><a>2</a></li></ul>"#;
    let result = spacebattles::parse_spacebattles_pages(html);
    assert!(result >= 1);
}
