use serde::Serialize;
#[derive(Debug, Serialize, Default)]
pub struct Chapter {
    pub site: String,
    pub title: Option<String>,
    pub text: Option<String>,
    pub chapter_number: Option<u32>,
    pub chapter_id: Option<u64>,
}
#[derive(Debug, Serialize, Default)]
pub struct Story {
    pub site: String,
    pub title: Option<String>,
    pub author_id: Option<u64>,
    pub author_name: Option<String>,
    pub story_id: Option<u64>,
    pub story_name: Option<String>,
    pub chapters: Vec<Chapter>,

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
