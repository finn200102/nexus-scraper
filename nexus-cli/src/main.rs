use clap::Parser;
use nexus_core::error::Result;
use nexus_core::sites::get_site;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(long)]
    site: String,
    #[arg(long)]
    author_id: u64,
    #[arg(long)]
    story_name: String,
    #[arg(long, default_value = "1")]
    chapter_number: u32,
}

#[tokio::main]
async fn main() -> Result<()> { 
    let args = Cli::parse();
    
    let client = reqwest::Client::new();

    let site = get_site(&args.site)?;
    let chapter = site.fetch_chapter(args.author_id, &args.story_name, args.chapter_number, &client).await?;
    let filename = format!("chapter{}.html", args.chapter_number);
    tokio::fs::write(&filename, chapter.text).await?;
    println!("Saved to {}", filename);
    Ok(())
   
}
