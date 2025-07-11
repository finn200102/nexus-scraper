use crate::{network, models::Chapter};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Author};
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

    async fn get_story_data_from_url(
        &self,
        url: &str,
        client: &reqwest::Client,
    ) -> Result<Story> {
        let split: Vec<_> = url.split('/').collect();
        let story_id = split.get(4).ok_or(CoreError::InvalidUrl("Story ID not found in URL".to_string()))?.parse::<u64>()
             .map_err(|_| CoreError::InvalidUrl("Failed to parse story id as number".to_string()))?;
        let story_name = split.get(6).ok_or(CoreError::InvalidUrl("Story name not found in URL".to_string()))?.to_string();

        let chapters = self.fetch_chapters(story_id, &client).await?;
        let author_data = self.fetch_author(story_id, &client).await?;
        let author_name = author_data.author_name; 
        let author_id = author_data.author_id;     

        Ok(Story{
            story_name: Some(story_name),
            story_id: Some(story_id),
            chapters: chapters,
            author_name: author_name,
            author_id: author_id,
            site: "fanfiction".to_string(),
        })
    }


    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author> {
        let url = format!("https://www.fanfiction.net/s/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = fanfiction::parse_author_from_story(&html);

        Ok(author)

         
        }






}
