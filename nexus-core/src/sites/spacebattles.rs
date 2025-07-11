use crate::{network};
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Story, Stories, Chapter, Author};
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
        chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url = format!("https://forums.spacebattles.com/posts/{}", &chapter_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let chapter = spacebattles::parse_spacebattles_chapter(&html, chapter_id, chapter_number);

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


        // add chapter number to chapter_number
        for (i, chapter) in all_chapters.iter_mut().enumerate() {
            chapter.chapter_number = Some((i as u32) + 1)
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
        num_pages: u32,
        client: &reqwest::Client,
        
    ) -> Result<Vec<Story>> {
        let sortby_name = sortby_id_to_name(sortby_id);
        let mut all_stories = Vec::new();
        for page_number in 1..num_pages {
            let url = format!("https://forums.spacebattles.com/forums/creative-writing.18/page-{}?order={}", page_number, sortby_name);
            let html = network::fetch_via_proxy(&url, client).await?;
            let stories = spacebattles::parse_spacebattles_stories(&html);
            all_stories.extend(stories);
        }
        Ok(all_stories)

    }


     async fn get_story_data_from_url(
            &self,
            url: &str,
            client: &reqwest::Client,
        ) -> Result<Story> {
        let (story_name, story_id) = { 
        let trimmed_url = url.trim_end_matches('/').to_string();
        let last_segment = trimmed_url.rsplit('/').next().unwrap_or_default();
        let (name_part, id_part) = last_segment.rsplit_once('.').unwrap_or((last_segment, ""));
        (name_part.to_string(), id_part.to_string())
    };
        let story_id: u64 = story_id.parse()
        .map_err(|e| CoreError::Parse(format!("Failed to parse story_id '{}': {}", story_id, e)))?;

        let chapters = self.fetch_chapters(story_id, &client).await?;
        let author_data = self.fetch_author(story_id, &client).await?;
        let author_name = author_data.author_name; 
        let author_id = author_data.author_id;     

        Ok(Story{
            story_name: Some(story_name),
            story_id: Some(story_id),
            chapters: chapters,
            author_name: author_name,
            author_id: author_id,
            site: "spacebattles".to_string(),
        })

     }

    async fn fetch_author(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Author> {
        let url = format!("https://forums.spacebattles.com/threads/{}", &story_id);
        let html = network::fetch_via_proxy(&url, client).await?;
        let author = spacebattles::parse_author_from_story(&html);

        Ok(author)

        }


}


fn sortby_id_to_name(sortby_id: u32) -> &'static str {
    match sortby_id {
        0 => "last_threadmark",
        _ => "Unknown",
    }
}

