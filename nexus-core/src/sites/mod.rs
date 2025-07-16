use crate::models::{Chapter, Stories, Story, Author};
use crate::error::Result;
use std::sync::Arc;

pub mod fanfiction;
pub mod archive;
pub mod spacebattles;
pub mod royalroad;
#[async_trait::async_trait]
pub trait Site {
    fn name(&self) -> &'static str;

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_id: u64,
        chapter_number: u32,
        client: &reqwest::Client,
        ) -> Result<Chapter>;


    async fn fetch_author_stories(
        &self,
        author_id: u64,
        author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories>;

    async fn fetch_stories_by_series(
        &self,
        medium_name: String,
        series_name: &str,
        sortby_id: u32,
        rating_id: u32,
        word_count: u32,
        time_range: u32,
        client: &reqwest::Client,
    ) -> Result<Stories>;

    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>>;



    async fn fetch_stories(
        &self,
        sortby_id: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<Story>>;

    async fn get_story_data_from_url(
        &self,
        url: &str,
        client: &reqwest::Client,
    ) -> Result<Story>;

    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author>;

}



pub fn get_site(name: &str) -> Result<Arc<dyn Site + Send + Sync>> {
    match name {
        "fanfiction" => Ok(Arc::new(fanfiction::FanFictionSite)),
        "archive" => Ok(Arc::new(archive::ArchiveSite)),
        "spacebattles" => Ok(Arc::new(spacebattles::SpacebattlesSite)),
        "royalroad" => Ok(Arc::new(royalroad::RoyalroadSite)),
        _ => Err(crate::error::CoreError::UnknownSite(name.into())),
    }
}

