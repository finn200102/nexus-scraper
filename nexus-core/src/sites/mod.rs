use crate::models::{Chapter, Stories};
use crate::error::Result;

pub mod fanfiction;
#[async_trait::async_trait]
pub trait Site {
    fn name(&self) -> &'static str;

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_number: u32,
        client: &reqwest::Client,
        ) -> Result<Chapter>;


    async fn fetch_author_stories(
        &self,
        author_id: u64,
        client: &reqwest::Client,
    ) -> Result<Stories>;


}



pub fn get_site(name: &str) -> Result<Box<dyn Site>> {
    match name {
        "fanfiction" => Ok(Box::new(fanfiction::FanFictionSite)),
        _ => Err(crate::error::CoreError::UnknownSite(name.into())),
    }
}

