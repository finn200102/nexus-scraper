use crate::{network, models::Chapter};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Author};
use crate::parser::fanfiction;

pub struct FanFictionSite;

#[async_trait::async_trait]
impl Site for FanFictionSite {
    fn name(&self) -> &'static str {
        "fanfiction"
    }
    /// Fetches a specific chapter from fanfiction.net
    ///
    /// This function contructs the chapter url from the `story_id` and from
    /// the `chapter_number`, fetches the HTML via proxy and parses it into
    /// a [`Chapter`] object.
    ///
    async fn fetch_chapter(
        &self,
        story_id: u64,
        _chapter_id: u64,
        chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{chapter_number}");

        let html = network::fetch_via_proxy(&url, client).await?;

        let chapter = fanfiction::parse_fanfiction_chapter(&html, chapter_number);

        Ok(chapter)

    }


    
    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let url = format!("https://www.fanfiction.net/s/{story_id}");
        let html = network::fetch_via_proxy(&url, client).await?;
        let mut chapters = fanfiction::parse_fanfiction_chapters(&html);
        
        for chapter in chapters.iter_mut() {
            if let Some(chapter_number) = chapter.chapter_number {
                chapter.url = Some(format!("https://www.fanfiction.net/s/{story_id}/{chapter_number}"));
            }
        }

        Ok(chapters)
        
    }

    async fn fetch_chapters_content(
        &self,
        story_id: u64,
        client: &reqwest::Client,
        ) -> Result<Vec<Chapter>> {
        let url = format!("https://www.fanfiction.net/s/{story_id}");
        let html = network::fetch_via_proxy(&url, client).await?;
        let mut chapters = fanfiction::parse_fanfiction_chapters(&html);

        for chapter in &mut chapters {
            let chapter_number = chapter.chapter_number;
            let full_chapter = self.fetch_chapter(story_id, 0, chapter_number.unwrap(), client).await?;

            chapter.text = full_chapter.text;
            
        }

        Ok(chapters)



    }

   



    async fn fetch_author_stories(
        &self,
        author_id: u64,
        _author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://www.fanfiction.net/u/{author_id}");

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = fanfiction::parse_fanfiction_stories(&html, author_id);

        Ok(stories)
    }



    async fn fetch_stories_by_series(
        &self,
        medium_name: String,
        series_name: &str,
        sortby_id: u32,
        rating_id: u32,
        word_count: u32,
        time_range: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let mut all_stories = Vec::new();

        for page in 1..=num_pages.max(1) {
            // URL format: https://www.fanfiction.net/{medium_name}/{series_name}/?srt={}&r={}&len={}&p={}
            let url = format!("https://www.fanfiction.net/{medium_name}/{series_name}/?srt={sortby_id}&r={rating_id}&len={word_count}&t={time_range}&p={page}");

            let html = network::fetch_via_proxy(&url, client).await?;

            let stories = fanfiction::parse_fanfiction_stories_by_series(&html);
            all_stories.extend(stories.stories);
        }

        Ok(Stories { stories: all_stories })
    }


    async fn fetch_stories(
        &self,
        sortby_id: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Vec<Story>> {
         Err(CoreError::UnsupportedOperation(
            "fetch_stories not supported for fanfiction".into(),
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

        let chapters = self.fetch_chapters(story_id, client).await?;
       // Get html
        let url = format!("https://www.fanfiction.net/s/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author_data = fanfiction::parse_author_from_story(&html);
        let author_name = author_data.author_name; 
        let author_id = author_data.author_id;
        let description = fanfiction::parse_description(&html);
        let story_name = fanfiction::parse_story_name(&html);

        let img_url = fanfiction::parse_cover(&html);
        let genre = fanfiction::parse_genre(&html);
        let tags = fanfiction::parse_tags(&html);
        let word_count = fanfiction::parse_word_count(&html);
        let reviews = fanfiction::parse_reviews(&html);
        let favorites = fanfiction::parse_favorites(&html);
        let follows = fanfiction::parse_follows(&html);
        let publish_date = fanfiction::parse_publish_date(&html);
        let updated_date = fanfiction::parse_updated_date(&html);
        let status = fanfiction::parse_status(&html);
        let chapter_count = fanfiction::parse_chapter_count(&html);
        


        Ok(Story{
            story_name: Some(story_name),
            story_id: Some(story_id),
            chapters,
            author_name,
            author_id,
            site: "fanfiction".to_string(),
            description: Some(description),
            img_url,
            tags,
            genre,
            word_count,
            reviews,
            favorites,
            follows,
            publish_date,
            updated_date,
            status,
            views: None,
            rating: None,
            chapter_count,
            url: Some(format!("https://www.fanfiction.net/s/{story_id}/")),

        })
    }




    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author> {
        let url = format!("https://www.fanfiction.net/s/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = fanfiction::parse_author_from_story(&html);

        Ok(author)

         
        }






}
