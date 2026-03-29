use serde::Serialize;
#[derive(Debug, Serialize, Default, Clone)]
pub struct Chapter {
    pub site: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub chapter_number: Option<u32>,
    pub chapter_id: Option<u64>,
    pub url: Option<String>,
}
#[derive(Debug, Serialize, Default)]
pub struct Story {
    pub site: String,
    pub author_id: Option<u64>,
    pub author_name: Option<String>,
    pub story_id: Option<u64>,
    pub story_name: Option<String>,
    pub chapters: Vec<Chapter>,
    pub description: Option<String>,
    pub img_url: Option<String>,
    pub tags: Vec<String>,
    pub genre: Vec<String>,
    pub word_count: Option<u64>,
    pub reviews: Option<u64>,
    pub favorites: Option<u64>,
    pub follows: Option<u64>,
    pub publish_date: Option<String>,
    pub updated_date: Option<String>,
    pub status: Option<String>,
    pub views: Option<u64>,
    pub rating: Option<f64>,
    pub chapter_count: Option<u64>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct Stories {
    pub stories: Vec<Story>,
}

#[derive(Debug, Serialize, Default)]
pub struct Author {
    pub author_name: Option<String>,
    pub author_id: Option<u64>,
}
