use crate::{network, models::Chapter};
use crate::error::Result;
use crate::sites::Site;
use crate::models::Stories;
use crate::parser::fanfiction;

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

        let chapter = fanfiction::parse_fanfiction_chapter(&html, chapter_number);

        Ok(chapter)

    }


    
    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        unimplemented!("fetch_chapters is not yet implemented for FanFictionSite");
        //let url = format!("https://archiveofourown.org/works/{}/navigate", story_id);
        //let html = network::fetch_via_proxy(&url, client).await?;
        //let chapters = fanfiction::parse_fanfiction_chapters(&html);

        //Ok(chapters)
        
    }



    async fn fetch_author_stories(
        &self,
        author_id: u64,
        _author_name: String,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let url = format!("https://www.fanfiction.net/u/{}", author_id);

        let html = network::fetch_via_proxy(&url, client).await?;

        let stories = fanfiction::parse_fanfiction_stories(&html, author_id);

        Ok(stories)
    }

}
