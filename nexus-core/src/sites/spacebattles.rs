use crate::{network, models::Chapter};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::Stories;
use crate::parser::spacebattles;

pub struct SpacebattlesSite;

#[async_trait::async_trait]
impl Site for SpacebattlesSite {
    fn name(&self) -> &'static str {
        "spacebattles"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_id: u64,
        _chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://forums.spacebattles.com/posts/{}", &chapter_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapter = spacebattles::parse_spacebattles_chapter(&html, chapter_id);

        Ok(chapter)
 
    }


    
    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://forums.spacebattles.com/threads/{}/reader/page-{}", story_id, 1);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = spacebattles::parse_spacebattles_chapters(&html);

        Ok(chapters)
        
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
            "fetch_stories_by_series not supported for spacebattles".into(),
        ))
 
   }


}
