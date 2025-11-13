use crate::{network, models::Chapter};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Author};
use crate::parser::royalroad;
use crate::parser::fanfiction;

pub struct RoyalroadSite;

#[async_trait::async_trait]
impl Site for RoyalroadSite{
    fn name(&self) -> &'static str {
        "royalroad"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_id: u64,
        _chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://www.royalroad.com/fiction/{}/chapter/{}", story_id, chapter_id);

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = royalroad::parse_chapter(&html, chapter_id);

        Ok(chapter)

    }


    
    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://www.royalroad.com/fiction/{}", story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = royalroad::parse_chapters(&html);

        Ok(chapters)
        
    }


    async fn fetch_chapters_content(
        &self,
        story_id: u64,
        client: &reqwest::Client,
        ) -> Result<Vec<Chapter>> {
         Err(CoreError::UnsupportedOperation(
            "fetch_stories not supported for fanfiction".into(),
        ))
        
    }




    async fn fetch_author_stories(
        &self,
        author_id: u64,
        _author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
         Err(CoreError::UnsupportedOperation(
            "fetch_author_stories not supported for spacebattles".into(),
        ))
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
          Err(CoreError::UnsupportedOperation(
            "fetch_author_stories not supported for spacebattles".into(),
        ))
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
        let story_name = split.get(5).ok_or(CoreError::InvalidUrl("Story name not found in URL".to_string()))?.to_string();

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
            site: "royalroad".to_string(),
            ..Default::default()

        })
    }


    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author> {
        let url = format!("https://www.royalroad.com/fiction/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = royalroad::parse_author_from_story(&html);

        Ok(author)

         
        }






}
