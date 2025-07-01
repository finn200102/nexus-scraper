use crate::{network, models::Chapter};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story};
use crate::parser::fanfiction;

pub struct FanFictionSite;

#[async_trait::async_trait]
impl Site for FanFictionSite {
    fn name(&self) -> &'static str {
        "fanfiction"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
        _chapter_id: u64,
        chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://www.fanfiction.net/s/{}/{}", story_id, chapter_number);

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = fanfiction::parse_fanfiction_chapter(&html, chapter_number);

        Ok(chapter)

    }


    
    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://www.fanfiction.net/s/{}", story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = fanfiction::parse_fanfiction_chapters(&html);

        Ok(chapters)
        
    }



    async fn fetch_author_stories(
        &self,
        author_id: u64,
        _author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://www.fanfiction.net/u/{}", author_id);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = fanfiction::parse_fanfiction_stories(&html, author_id);

        Ok(stories)
    }



    async fn fetch_stories_by_series(
        &self,
        medium_name: String,
        series_name: &str,
        sortby_id: u32,
        rating_id: u32,
        word_count: u32,
        time_range: u32,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://www.fanfiction.net/{}/{}/?&srt={}&r={}&len={}&t={}", medium_name, series_name, sortby_id, rating_id, word_count, time_range);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = fanfiction::parse_fanfiction_stories_by_series(&html);

        Ok(stories)
    }


    async fn fetch_stories(
        &self,
        sortby_id: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<Story>> {
         Err(CoreError::UnsupportedOperation(
            "fetch_stories not supported for fanfiction".into(),
        ))
        
    }


}
