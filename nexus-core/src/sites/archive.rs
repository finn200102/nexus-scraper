use crate::error::CoreError;
use crate::error::Result;
use crate::models::{Author, Chapter, Stories, Story};
use crate::network;
use crate::parser::archive;
use crate::sites::Site;
pub struct ArchiveSite;

#[async_trait::async_trait]
impl Site for ArchiveSite {
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
        let url = if chapter_id == 0 {
            format!("https://archiveofourown.org/works/{story_id}?view_adult=true")
        } else {
            format!("https://archiveofourown.org/works/{story_id}/chapters/{chapter_id}?view_adult=true")
        };

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = archive::parse_archive_chapter(&html, chapter_id);

        Ok(chapter)
    }

    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://archiveofourown.org/works/{story_id}/navigate?view_adult=true");
        let html = network::fetch_via_proxy(&url, client).await?;
        let mut chapters = archive::parse_archive_chapters(&html);

        if chapters.is_empty() {
            chapters.push(Chapter {
                site: "archive".to_string(),
                chapter_id: Some(0),
                chapter_number: Some(1),
                url: Some(format!(
                    "https://archiveofourown.org/works/{story_id}?view_adult=true"
                )),
                ..Default::default()
            });
            return Ok(chapters);
        }

        for chapter in chapters.iter_mut() {
            if let Some(chapter_id) = chapter.chapter_id {
                chapter.url = Some(format!(
                    "https://archiveofourown.org/works/{story_id}/chapters/{chapter_id}?view_adult=true"
                ));
            }
        }

        for (i, chapter) in chapters.iter_mut().enumerate() {
            chapter.chapter_number = Some((i as u32) + 1)
        }

        Ok(chapters)
    }

    async fn fetch_chapters_content(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let mut chapters = self.fetch_chapters(story_id, client).await?;

        for chapter in &mut chapters {
            if let Some(chapter_id) = chapter.chapter_id {
                let full_chapter = self
                    .fetch_chapter(
                        story_id,
                        chapter_id,
                        chapter.chapter_number.unwrap_or_default(),
                        client,
                    )
                    .await?;
                chapter.text = full_chapter.text;
            }
        }

        Ok(chapters)
    }

    async fn fetch_author_stories(
        &self,
        _author_id: u64,
        author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://archiveofourown.org/users/{author_name}/series");

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = archive::parse_archive_stories(&html, &author_name);

        Ok(stories)
    }

    async fn fetch_stories_by_series(
        &self,
        _medium_name: String,
        _series_name: &str,
        _sortby_id: u32,
        _rating_id: u32,
        _word_count: u32,
        _time_range: u32,
        _num_pages: u32,
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

    async fn get_story_data_from_url(&self, url: &str, client: &reqwest::Client) -> Result<Story> {
        let split: Vec<_> = url.split('/').collect();
        let story_id = split
            .get(4)
            .ok_or(CoreError::InvalidUrl(
                "Story ID not found in URL".to_string(),
            ))?
            .parse::<u64>()
            .map_err(|_| CoreError::InvalidUrl("Failed to parse story id as number".to_string()))?;

        let chapters = self.fetch_chapters(story_id, client).await?;
        // Get html
        let url = format!(
            "https://archiveofourown.org/works/{}?view_adult=true",
            &story_id
        );
        let html = network::fetch_via_proxy(&url, client).await?;
        let story_name = archive::parse_story_title(&html);
        let author_data = archive::parse_author_from_story(&html);
        let author_name = author_data.author_name;
        let author_id = author_data.author_id;
        let description = archive::parse_description(&html);

        let tags = archive::parse_tags(&html);

        let publish_date = archive::parse_publish_date(&html);
        let updated_date = archive::parse_updated_date(&html);
        let word_count = archive::parse_word_count(&html);
        let reviews = archive::parse_reviews(&html);
        let favorites = archive::parse_favorites(&html);
        let follows = archive::parse_follows(&html);
        let views = archive::parse_views(&html);
        let chapter_count = archive::parse_chapter_count(&html).or(Some(chapters.len() as u64));

        let status = match chapter_count {
            Some(count) if count == chapters.len() as u64 => Some("Complete".to_string()),
            Some(_) => Some("In Progress".to_string()),
            None => None,
        };

        Ok(Story {
            story_name,
            story_id: Some(story_id),
            chapters,
            author_name,
            author_id,
            site: "archive".to_string(),
            description: Some(description),
            tags,
            genre: vec![],
            word_count,
            reviews,
            favorites,
            follows,
            publish_date,
            updated_date,
            status,
            views,
            chapter_count,
            url: Some(format!("https://archiveofourown.org/works/{story_id}")),
            ..Default::default()
        })
    }
    async fn fetch_author(&self, story_id: u64, client: &reqwest::Client) -> Result<Author> {
        let url = format!("https://archiveofourown.org/works/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = archive::parse_author_from_story(&html);

        Ok(author)
    }
}
