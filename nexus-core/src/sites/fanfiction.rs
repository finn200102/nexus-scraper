use crate::{network, parser, models::Chapter};
use crate::error::Result;
use crate::sites::Site;

pub struct FanFictionSite;

#[async_trait::async_trait]
impl Site for FanFictionSite {
    fn name(&self) -> &'static str {
        "fanfiction"
    }

    async fn fetch_chapter(
        &self,
        author_id: u64,
        story_name: &str,
        chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://www.fanfiction.net/s/{}/{}/{}", author_id, chapter_number, story_name);

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = parser::parse_fanfiction_chapter(&html);

        Ok(chapter)

    }
}
