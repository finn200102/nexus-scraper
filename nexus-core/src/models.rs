use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct Chapter {
    pub title: String,
    pub text: String,
    pub chapter_number: u32,
    pub chapter_id: u64,
}
#[derive(Debug, Serialize)]
pub struct Story {
    pub title: String,
    pub author_id: u64,
    pub author_name: String,
    pub story_id: u64,
    pub chapters: Vec<Chapter>,

}

#[derive(Debug, Serialize)]
pub struct Stories {
    pub stories: Vec<Story>,

} 
