use crate::network;
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Author, Chapter};
use crate::parser::webnovel;

pub struct WebnovelSite;

#[async_trait::async_trait]
impl Site for WebnovelSite {
    fn name(&self) -> &'static str {
        "webnovel"
    }

    async fn fetch_chapter(
        &self,
        _story_id: u64,
        _chapter_id: u64,
        _chapter_number: u32,
        _client: &reqwest::Client,
    ) -> Result<Chapter> {
        Err(CoreError::UnsupportedOperation("fetch_chapter not supported for webnovel".into()))
    }

    async fn fetch_author_stories(
        &self,
        _author_id: u64,
        _author_name: String,
        _client: &reqwest::Client,
    ) -> Result<Stories> {
        Err(CoreError::UnsupportedOperation("fetch_author_stories not supported for webnovel".into()))
    }

    async fn fetch_stories_by_series(
        &self,
        _medium_name: String,
        _series_name: &str,
        _sortby_id: u32,
        _rating_id: u32,
        _word_count: u32,
        _time_range: u32,
        _num_pages: u32,
        _client: &reqwest::Client,
    ) -> Result<Stories> {
        Err(CoreError::UnsupportedOperation("fetch_stories_by_series not supported for webnovel".into()))
    }

    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://www.webnovel.com/book/{}/catalog", story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = webnovel::parse_catalog(&html);
        Ok(chapters)
    }

    async fn fetch_chapters_content(
        &self,
        _story_id: u64,
        _client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        Err(CoreError::UnsupportedOperation("fetch_chapters_content not supported for webnovel".into()))
    }

    async fn fetch_stories(
        &self,
        _sortby_id: u32,
        _num_pages: u32,
        _client: &reqwest::Client,
    ) -> Result<Vec<Story>> {
        Err(CoreError::UnsupportedOperation("fetch_stories not supported for webnovel".into()))
    }

    async fn get_story_data_from_url(
        &self,
        url: &str,
        client: &reqwest::Client,
    ) -> Result<Story> {
        let html = network::fetch_via_proxy(url, client).await?;

        let story_name = webnovel::parse_story_name(&html);
        let img_url = webnovel::parse_cover(&html);
        let author_data = webnovel::parse_author(&html);
        let author_name = author_data.as_ref().and_then(|a| a.author_name.clone());
        let author_id = author_data.as_ref().and_then(|a| a.author_id);
        let description = webnovel::parse_description(&html);
        let tags = webnovel::parse_tags(&html);
        let chapter_count = webnovel::parse_chapter_count(&html);
        let views = webnovel::parse_views(&html);
        let rating = webnovel::parse_rating(&html);
        let reviews = webnovel::parse_reviews(&html);

        let story_id = extract_story_id(url);

        Ok(Story {
            site: self.name().to_string(),
            story_id,
            story_name,
            img_url,
            author_id,
            author_name,
            description,
            tags,
            chapter_count,
            views,
            rating,
            reviews,
            url: Some(url.to_string()),
            ..Default::default()
        })
    }

    async fn fetch_author(
        &self,
        _story_id: u64,
        _client: &reqwest::Client,
    ) -> Result<Author> {
        Err(CoreError::UnsupportedOperation("fetch_author not supported for webnovel".into()))
    }
}

fn extract_story_id(url: &str) -> Option<u64> {
    let parts: Vec<&str> = url.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "book" {
            if let Some(id_part) = parts.get(i + 1) {
                let id = id_part.split('_').last()?;
                return id.parse::<u64>().ok();
            }
        }
    }
    None
}
