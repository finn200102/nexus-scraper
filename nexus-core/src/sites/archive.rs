use crate::{network, parser, models::Chapter};
use crate::error::Result;
use crate::sites::Site;
use crate::models::Stories;

pub struct ArchiveSite;

#[async_trait::async_trait]
impl Site for ArchiveSite{
    fn name(&self) -> &'static str {
        "archive"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
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
        _author_id: u64,
        author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://archiveofourown.org/users/{}/series", author_name);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = parser::parse_archive_stories(&html, &author_name);

        Ok(stories)
    }

}
