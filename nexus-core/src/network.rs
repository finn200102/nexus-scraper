use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::{Result, CoreError};
use chrono::NaiveDate;


pub fn detect_site_from_url(url: &str) -> Result<&'static str> {
    let url_lower = url.to_lowercase();
    if url_lower.contains("fanfiction.net") {
        Ok("fanfiction")
    } else if url_lower.contains("archiveofourown.org") {
        Ok("archive")
    } else if url_lower.contains("royalroad.com") {
        Ok("royalroad")
    } else if url_lower.contains("spacebattles.com") {
        Ok("spacebattles")
    } else if url_lower.contains("webnovel.com") {
        Ok("webnovel")
    } else {
        Err(CoreError::InvalidUrl(
            "Could not detect site from URL. Please specify site manually.\nKnown sites: fanfiction, archive, royalroad, spacebattles, webnovel".into()
        ))
    }
}

pub fn parse_date(date_str: &str) -> Option<String> {
    let date_str = date_str.trim();
    
    if date_str.is_empty() {
        return None;
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%b %d, %Y") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d-%b-%Y") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d %B %Y") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
        return Some(date.format("%Y-%m-%d").to_string());
    }
    
    None
}


#[derive(Serialize, Default)]
struct ProxyOptions {
    cmd: String,
    url: String,
    session: String,
    #[serde(rename = "maxTimeout", skip_serializing_if = "Option::is_none")]
    max_timeout: Option<u32>,
    #[serde(rename = "returnRaw", skip_serializing_if = "Option::is_none")]
    return_raw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    browser: Option<BrowserConfig>,
}

#[derive(Serialize, Default)]
struct BrowserConfig {
    platform: String,
    browser: String,
    device: String,
}

#[derive(Deserialize)]
struct ProxyResponse {
    solution: Option<serde_json::Value>,
}

pub async fn fetch_via_proxy(url: &str, client: &Client) -> Result<String> {
    fetch_via_proxy_with_options(url, client, None).await
}

pub async fn fetch_via_proxy_browser(url: &str, client: &Client) -> Result<String> {
    let browser_config = Some(BrowserConfig {
        platform: "windows".to_string(),
        browser: "chrome".to_string(),
        device: "desktop".to_string(),
    });
    fetch_via_proxy_with_options(url, client, browser_config).await
}

async fn fetch_via_proxy_with_options(url: &str, client: &Client, browser: Option<BrowserConfig>) -> Result<String> {
    let proxy_url = std::env::var("FLARESOLVERR_URL")
        .unwrap_or_else(|_| "http://localhost:8191/v1".to_string());
    let session = format!("fiction_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let mut payload = ProxyOptions {
        cmd: "request.get".to_string(),
        url: url.to_string(),
        session,
        max_timeout: Some(60000),
        return_raw: Some(true),
        browser,
    };

    let res = client
        .post(proxy_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?
        .json::<ProxyResponse>()
        .await?;

    let html = res.solution
        .and_then(|sol| sol.get("response").and_then(|r| r.as_str().map(String::from)))
        .ok_or_else(|| CoreError::Parse("Missing HTML in proxy".into()))?;

    Ok(html)
}

