use nexus_core::parser::fanfiction;

#[test]
fn test_parse_fanfiction_chapters() {
    let html = r#"<select id="chap_select">
        <option value="1">Chapter 1: The Beginning</option>
        <option value="2">Chapter 2: The Journey</option>
        <option value="3">Chapter 3: The End</option>
    </select>"#;
    let result = fanfiction::parse_fanfiction_chapters(html);
    assert_eq!(result.len(), 3);

    let first = &result[0];
    assert_eq!(first.chapter_number, Some(1));
    assert_eq!(first.title, Some("Chapter 1: The Beginning".to_string()));
    assert_eq!(first.site, "fanfiction");
}

#[test]
fn test_parse_fanfiction_chapters_no_duplicates() {
    let html = r#"<select id="chap_select">
        <option value="1">Chapter 1</option>
        <option value="1">Chapter 1</option>
        <option value="2">Chapter 2</option>
    </select>"#;
    let result = fanfiction::parse_fanfiction_chapters(html);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_tags() {
    let html = r#"<span class="tags">
        <a href="/tags/hp-canon">Harry Potter</a>
        <a href="/tags/dragons">Dragons</a>
    </span>"#;
    let _result = fanfiction::parse_tags(html);
}

#[test]
fn test_parse_genre() {
    let html = r#"<a href="fictionratings.com">Romance</a>"#;
    let _result = fanfiction::parse_genre(html);
}

#[test]
fn test_parse_word_count() {
    let html = r#"<span class="xgray">Words: 100,000 - Chapters: 50</span>"#;
    let result = fanfiction::parse_word_count(html);
    assert_eq!(result, Some(100000));
}

#[test]
fn test_parse_publish_date() {
    let html = r#"<span class="xgray">Published:01-Jan-2020 - Words: 1000</span>"#;
    let _result = fanfiction::parse_publish_date(html);
}

#[test]
fn test_parse_updated_date() {
    let html = r#"<span class="xgray">Updated:15-Jun-2021 - Words: 1000</span>"#;
    let _result = fanfiction::parse_updated_date(html);
}

#[test]
fn test_parse_status_complete() {
    let html = r#"<span class="xgray">Status:Complete - Words: 1000</span>"#;
    let _result = fanfiction::parse_status(html);
}

#[test]
fn test_parse_chapter_count() {
    let html = r#"<span class="xgray">Chapters: 18 - Words: 100,000</span>"#;
    let result = fanfiction::parse_chapter_count(html);
    assert_eq!(result, Some(18));
}

#[test]
fn test_parse_fanfiction_chapter_layout() {
    let html = r#"
    <div id="storytext">
        <p><strong>Chapter 1: Title</strong></p>
        <p>First line<br>Second line</p>
        <p>
            Another paragraph with <em>formatting</em>.
        </p>
    </div>
    "#;

    let chapter = fanfiction::parse_fanfiction_chapter(html, 1);
    let text = chapter.text.unwrap();
    assert_eq!(
        text,
        "Chapter 1: Title\n\nFirst line\nSecond line\n\nAnother paragraph with formatting."
    );
}

#[test]
fn test_is_story_not_found() {
    let html = r#"<div id="content_wrapper_inner" style="padding:0.5em;"><div class="panel_warning"><span class="gui_warning">Story Not Found<hr size="1" noshade="">Story is unavailable for reading. (A)</span></div></div>"#;
    assert!(fanfiction::is_story_not_found(html));
}

#[test]
fn test_is_story_not_found_false() {
    let html = r#"<div id="profile_top"><b>My Story Title</b></div>"#;
    assert!(!fanfiction::is_story_not_found(html));
}
