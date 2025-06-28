use clap::{Parser, Subcommand};
use nexus_core::error::Result;
use nexus_core::sites::get_site;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    FetchChapter {
        #[arg(long)]
        site: String,
        #[arg(long)]
        story_id: u64,
        #[arg(long, default_value = "0")]
        chapter_id: u64,
        #[arg(long, default_value = "1")]
        chapter_number: u32,
    },

    FetchChapters {
        #[arg(long)]
        site: String,
        #[arg(long)]
        story_id: u64,
    },

    FetchAuthorStories {
        #[arg(long)]
        site: String,
        #[arg(long, default_value = "0")]
        author_id: u64,
        #[arg(long, default_value = "UNKNOWN")]
        author_name: String,
    }
}


async fn handle_fetch_chapter(
    site: String,
    story_id: u64,
    chapter_id: u64,
    chapter_number: u32,
    client: &reqwest::Client,
) -> Result<()> {

    let site = get_site(&site)?;
    let chapter = site.fetch_chapter(story_id, chapter_id, chapter_number, &client).await?;
    // TODO: check if the chapter_number was give or if chapter_number must be extracted from
    // chapter
    let filename = format!("chapter{}.html", chapter_number);
    tokio::fs::write(&filename, chapter.text).await?;
    println!("Saved to {}", filename);
    Ok(())

}

async fn handle_fetch_chapters(
    site: String,
    story_id: u64,
    client: &reqwest::Client,
) -> Result<()> {
    let site = get_site(&site)?;
    let chapters = site.fetch_chapters(story_id, &client).await?;
    let filename = format!("chapters{}.json", story_id);
    let json = serde_json::to_string_pretty(&chapters)?;
    tokio::fs::write(&filename, json).await?;
    println!("Saved to {}", filename);
    Ok(())

}
   
async fn handle_fetch_author_stories(
    site: String,
    author_id: u64,
    author_name: String,
    client: &reqwest::Client,
) -> Result<()> {

    let site = get_site(&site)?;
    let stories = site.fetch_author_stories(author_id, author_name, &client).await?;
    let filename = format!("author_{}_stories.json", author_id);
    let json = serde_json::to_string_pretty(&stories)?;
    tokio::fs::write(&filename, json).await?;
    println!("Saved to {}", filename);
    Ok(())

}
 
        

#[tokio::main]
async fn main() -> Result<()> { 
    let args = Cli::parse();
    
    let client = reqwest::Client::new();

    match args.command {
        Commands::FetchChapter {
            site,
            story_id,
            chapter_id,
            chapter_number,
        } => {
            handle_fetch_chapter(site, story_id, chapter_id, chapter_number, &client).await?;
        }
        Commands::FetchAuthorStories {
            site,
            author_id,
            author_name,
        } => {
            handle_fetch_author_stories(site, author_id, author_name, &client).await?;
        }
        Commands::FetchChapters {
            site,
            story_id,
        } => {
            handle_fetch_chapters(site, story_id, &client).await?;
        }
    }
    Ok(())
}


