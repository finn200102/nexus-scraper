use crate::{network};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Chapter, Author};
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


    async fn fetch_stories_by_series(&self,
        _medium_name: String,
        _series_name: &str,
        _sortby_id: u32,
        _rating_id: u32,
        _word_count: u32,
        _time_range: u32,
        client: &reqwest::Client,
        ) -> Result<Stories> {
        Err(CoreError::UnsupportedOperation(
            "fetch_stories_by_series not supported for archive".into(),
        ))
    }




    async fn fetch_stories(
        &self,
        sortby_id: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<Story>> {
         Err(CoreError::UnsupportedOperation(
            "fetch_stories not supported for archive".into(),
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

        let chapters = self.fetch_chapters(story_id, &client).await?;
        let author_data = self.fetch_author(story_id, &client).await?;
        let author_name = author_data.author_name; 
        let author_id = author_data.author_id;     

        Ok(Story{
            story_id: Some(story_id),
            chapters: chapters,
            author_name: author_name,
            author_id: author_id,
            site: "archive".to_string(),
            ..Default::default()
        })

         
        }
    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author> {
        let url = format!("https://archiveofourown.org/works/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = archive::parse_author_from_story(&html);

        Ok(author)

   
        }

}

