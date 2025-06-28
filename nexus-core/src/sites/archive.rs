use crate::{network, models::Chapter};
use crate::error::Result;
use crate::sites::Site;
use crate::models::Stories;
use crate::parser::archive;
pub struct ArchiveSite;

#[async_trait::async_trait]
impl Site for ArchiveSite{
    fn name(&self) -> &'static str {
        "archive"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_id: u64,
        _chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://archiveofourown.org/works/{}/chapters/{}", story_id, chapter_id);

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = archive::parse_archive_chapter(&html, chapter_id);

        Ok(chapter)

    }


    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://archiveofourown.org/works/{}/navigate", story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapters = archive::parse_archive_chapters(&html);

        Ok(chapters)
        
    }


    async fn fetch_author_stories(
        &self,
        _author_id: u64,
        author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://archiveofourown.org/users/{}/series", author_name);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = archive::parse_archive_stories(&html, &author_name);

        Ok(stories)
    }

}
