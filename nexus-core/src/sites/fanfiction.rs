use crate::{network, parser, models::Chapter};
use crate::error::Result;
use crate::sites::Site;
use crate::models::Stories;

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

        let chapter = parser::parse_fanfiction_chapter(&html);

        Ok(chapter)

    }


    async fn fetch_author_stories(
        &self,
        author_id: u64,
        _author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://www.fanfiction.net/u/{}", author_id);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = parser::parse_fanfiction_stories(&html, author_id);

        Ok(stories)
    }

}
