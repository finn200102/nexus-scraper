use crate::network;
use crate::error::Result;
use crate::error::CoreError;
use crate::sites::Site;
use crate::models::{Stories, Story, Author, Chapter};
use crate::parser::webnovel;

pub struct WebnovelSite;

async fn fetch_chapters_via_api(
    story_id: u64,
    client: &reqwest::Client,
) -> Result<Vec<Chapter>> {
    let csrf_token = network::fetch_webnovel_csrf_token(client).await?;

    let mut all_chapters = Vec::new();
    let mut page_index: u32 = 0;

    loop {
        let json = network::fetch_webnovel_chapter_list(story_id, page_index, &csrf_token, client).await?;

        let chapters = webnovel::parse_chapter_list_api(&json);

        if chapters.is_empty() {
            break;
        }

        all_chapters.extend(chapters);

        if !webnovel::has_more_chapters(&json) {
            break;
        }

        page_index += 1;
    }

    Ok(all_chapters)
}

#[async_trait::async_trait]
impl Site for WebnovelSite {
    fn name(&self) -> &'static str {
        "webnovel"
    }

    async fn fetch_chapter(
        &self,
        story_id: u64,
        chapter_id: u64,
        chapter_number: u32,
        client: &reqwest::Client,
    ) -> Result<Chapter> {
        let url_new = format!("https://www.webnovel.com/book/{story_id}/{chapter_id}");
        let url_old = format!("https://www.webnovel.com/book/{story_id}_{story_id}/chapter_{chapter_id}");

        let (html, url_used) = match network::fetch_via_proxy_browser(&url_new, client).await {
            Ok(html) if webnovel::parse_chapter_content(&html).is_some() => (html, url_new),
            _ => {
                let html = network::fetch_via_proxy_browser(&url_old, client).await?;
                (html, url_old)
            }
        };
        
        let title = webnovel::parse_chapter_title(&html);
        let text = webnovel::parse_chapter_content(&html);
        
        Ok(Chapter {
            site: self.name().to_string(),
            title,
            text,
            chapter_number: Some(chapter_number),
            chapter_id: Some(chapter_id),
            url: Some(url_used),
        })
    }

    async fn fetch_author_stories(
        &self,
        _author_id: u64,
        _author_name: String,
        _client: &reqwest::Client,
    ) -> Result<Stories> {
        Err(CoreError::UnsupportedOperation("fetch_author_stories not supported for webnovel".into()))
    }

    async fn fetch_stories_by_series(
        &self,
        keywords: String,
        _series_name: &str,
        _sortby_id: u32,
        _rating_id: u32,
        _word_count: u32,
        _time_range: u32,
        num_pages: u32,
        client: &reqwest::Client,
    ) -> Result<Stories> {
        let type_param = match keywords.as_str() {
            "fanfic" | "fanfic-anime-comics" => "fanfic",
            "original" => "original", 
            "comics" => "comics",
            _ => "fanfic",
        };
        
        let mut all_stories = Vec::new();
        let max_pages = if num_pages == 0 { 10 } else { num_pages };

        for page_index in 0..max_pages {
            let url = if page_index == 0 {
                format!("https://www.webnovel.com/search?keywords={}&type={}", 
                    urlencoding::encode(&keywords), 
                    type_param
                )
            } else {
                format!("https://www.webnovel.com/search?keywords={}&type={}&pageIndex={}", 
                    urlencoding::encode(&keywords), 
                    type_param,
                    page_index + 1
                )
            };
            
            let html = network::fetch_via_proxy_browser(&url, client).await?;
            let stories = webnovel::parse_search_results(&html);
            
            if stories.stories.is_empty() {
                break;
            }
            
            all_stories.extend(stories.stories);
        }
        
        Ok(Stories { stories: all_stories })
    }

    async fn fetch_chapters(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        match fetch_chapters_via_api(story_id, client).await {
            Ok(chapters) if !chapters.is_empty() => Ok(chapters),
            _ => {
                let url = format!("https://www.webnovel.com/book/{story_id}/catalog");
                let html = network::fetch_via_proxy_browser(&url, client).await?;
                let chapters = webnovel::parse_catalog(&html);
                Ok(chapters)
            }
        }
    }

    async fn fetch_chapters_content(
        &self,
        story_id: u64,
        client: &reqwest::Client,
    ) -> Result<Vec<Chapter>> {
        let mut chapters = self.fetch_chapters(story_id, client).await?;

        for i in 0..chapters.len() {
            let ch_id = chapters[i].chapter_id;
            let c_num = chapters[i].chapter_number;
            let url = chapters[i].url.clone();

            if chapters[i].text.is_some() {
                continue;
            }

            if let (Some(chapter_id), Some(chapter_number)) = (ch_id, c_num) {
                if let Some(ref chapter_url) = url {
                    if let Ok(full_chapter) = fetch_chapter_content_from_url(
                        chapter_url,
                        chapter_number,
                        Some(chapter_id),
                        client
                    ).await {
                        chapters[i].text = full_chapter.text;
                    }
                } else if let Ok(full_chapter) = self.fetch_chapter(story_id, chapter_id, chapter_number, client).await {
                    chapters[i].text = full_chapter.text;
                    chapters[i].url = full_chapter.url;
                }
            }
        }

        Ok(chapters)
    }

    async fn fetch_stories(
        &self,
        _sortby_id: u32,
        _num_pages: u32,
        _client: &reqwest::Client,
    ) -> Result<Vec<Story>> {
        Err(CoreError::UnsupportedOperation("fetch_stories not supported for webnovel".into()))
    }

    async fn get_story_data_from_url(
        &self,
        url: &str,
        client: &reqwest::Client,
    ) -> Result<Story> {
        let html = network::fetch_via_proxy_browser(url, client).await?;

        if webnovel::is_story_not_found(&html) {
            let story_id = extract_story_id(url);
            return Err(CoreError::StoryNotFound(format!("Story ID {:?} not found", story_id)));
        }

        let story_name = webnovel::parse_story_name(&html);
        let img_url = webnovel::parse_cover(&html);
        let author_data = webnovel::parse_author(&html);
        let author_name = author_data.as_ref().and_then(|a| a.author_name.clone());
        let author_id = author_data.as_ref().and_then(|a| a.author_id);
        let description = webnovel::parse_description(&html);
        let tags = webnovel::parse_tags(&html);
        let mut chapter_count = webnovel::parse_chapter_count(&html);
        let views = webnovel::parse_views(&html);
        let rating = webnovel::parse_rating(&html);
        let reviews = webnovel::parse_reviews(&html);

        let story_id = extract_story_id(url);

        let chapters = if let Some(sid) = story_id {
            self.fetch_chapters(sid, client).await?
        } else {
            vec![]
        };

        if chapter_count.is_none() {
            chapter_count = Some(chapters.len() as u64);
        }

        Ok(Story {
            site: self.name().to_string(),
            story_id,
            story_name,
            img_url,
            author_id,
            author_name,
            description,
            tags,
            chapters,
            chapter_count,
            views,
            rating,
            reviews,
            url: Some(url.to_string()),
            ..Default::default()
        })
    }

    async fn fetch_author(
        &self,
        _story_id: u64,
        _client: &reqwest::Client,
    ) -> Result<Author> {
        Err(CoreError::UnsupportedOperation("fetch_author not supported for webnovel".into()))
    }
}

async fn fetch_chapter_content_from_url(
    url: &str,
    chapter_number: u32,
    chapter_id: Option<u64>,
    client: &reqwest::Client,
) -> Result<Chapter> {
    let html = network::fetch_via_proxy_browser(url, client).await?;
    
    let title = webnovel::parse_chapter_title(&html);
    let text = webnovel::parse_chapter_content(&html);
    
    Ok(Chapter {
        site: "webnovel".to_string(),
        title,
        text,
        chapter_number: Some(chapter_number),
        chapter_id,
        url: Some(url.to_string()),
    })
}

fn extract_story_id(url: &str) -> Option<u64> {
    let parts: Vec<&str> = url.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "book" {
            if let Some(id_part) = parts.get(i + 1) {
                let id = id_part.split('_').next_back()?;
                return id.parse::<u64>().ok();
            }
        }
    }
    None
}
