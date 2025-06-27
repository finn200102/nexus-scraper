use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    site: String,
    author_id: u64,
    story: String,
    #[arg(default_value = "1")]
    chapter: u32,
}

#[derive(Serialize)]
struct ProxyRequest {
    cmd: String,
    url: String,
    session: String,
    maxTimeout: u32,
}

#[derive(Deserialize, Debug)]
struct ProxyResponse {
    status: String,
    solution: Option<serde_json::Value>,
}


fn create_url(site: &str, author_id: u64, story: String, chapter: u32) -> Option<String> {
    match site {
        "fanfiction" => {
            Some(format!("https://www.fanfiction.net/s/{}/{}/{}", author_id, chapter, story))
        }
        _ => None,
    }    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Cli::parse();
    println!("Site {}", args.site);
    println!("Chapter {}", args.chapter);
    let url = create_url(&args.site, args.author_id, args.story, args.chapter);
    
    let client = Client::new();
    let proxy_url = "http://localhost:8191/v1";

    match url {
        Some(link) => {
            let payload = ProxyRequest {
            cmd: "request.get".to_string(),
            url: link,
            maxTimeout: 60000,
            session: "fiction".into(),
            };

            let res = client
                .post(proxy_url)
                .json(&payload)
                .send()
                .await?
                .json::<ProxyResponse>()
                .await?;

            if let Some(solution) = res.solution {
                if let Some(html) = solution.get("response").and_then(|r| r.as_str()) {
                    let document = Html::parse_document(html);
                    let selector = Selector::parse("div#storytext").unwrap();
                    if let Some(div) = document.select(&selector).next() {
                        let story_text = div.text().collect::<Vec<_>>().join(" ");
                        let filename = format!("chapter{}.html", args.chapter);
                        tokio::fs::write(filename, story_text).await?;
                    }

                    println!("Chapter saved to chapter.html");
                } else {
                    println!("No response field found in solution")
                }
            } else {
                println!("No solution found in response")
            }

        },
        None => println!("Unsuported site: {}", args.site),
    }

     
    Ok(())
   
}
