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
        let first_url = format!("https://forums.spacebattles.com/threads/{}/reader/page-1", story_id);
        let first_html = network::fetch_via_proxy(&first_url, client).await?;
        let pages = spacebattles::parse_spacebattles_pages(&first_html) + 1;
        let mut all_chapters = Vec::new();
        // pages+1 TODO find out why
        for page_number in 1..pages {
            let url = format!("https://forums.spacebattles.com/threads/{}/reader/page-{}", story_id, page_number);
            let html = network::fetch_via_proxy(&url, client).await?;
            let chapters = spacebattles::parse_spacebattles_chapters(&html);
            all_chapters.extend(chapters);
        }
        Ok(all_chapters)
        
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

    async fn fetch_stories(
        &self,
        sortby_id: u32,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let sortby_name = sortby_id_to_name(sortby_id);
        let url = format!("https://forums.spacebattles.com/forums/creative-writing.18/?order={}", sortby_name);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = spacebattles::parse_spacebattles_stories(&html);

        Ok(chapters)

    }



}


fn sortby_id_to_name(sortby_id: u32) -> &'static str {
    match sortby_id {
        0 => "last_threadmark",
        _ => "Unknown",
    }
}

