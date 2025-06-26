use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    site: String,
    author_id: u64,
    story: String,
    #[arg(default_value = "1")]
    chapter: u32,
}



fn create_url(site: &str, author_id: u64, story: String, chapter: u32) -> Option<String> {
    match site {
        "fanfiction" => {
            Some(format!("https://www.fanfiction.net/s/{}/{}/{}", author_id, chapter, story))
        }
        _ => None,
    }    
}

fn main() {
    let args = Cli::parse();
    println!("Site {}", args.site);
    println!("Chapter {}", args.chapter);
    let url = create_url(&args.site, args.author_id, args.story, args.chapter);
    
    match url {
        Some(link) => println!("Url: {}", link),
        None => println!("Unsuported site: {}", args.site),
    }
}
