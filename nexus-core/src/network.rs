use reqwest::{Client, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use crate::error::{Result, CoreError};
use chrono::NaiveDate;
use tokio::time::{sleep, Duration};


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

#[derive(Serialize, Default, Clone)]
struct BrowserConfig {
    platform: String,
    browser: String,
    device: String,
}

#[derive(Deserialize)]
struct ProxyResponse {
    solution: Option<serde_json::Value>,
    status: Option<String>,
    message: Option<String>,
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

    match fetch_direct(url, client).await {
        Ok(html) => return Ok(html),
        Err(err) => {
            if std::env::var("NEXUS_PROXY_DEBUG").is_ok() {
                eprintln!("Direct fetch failed: {}", err);
            }
        }
    }
    fn extract_html(value: &serde_json::Value) -> Option<String> {
        if let Some(s) = value.as_str() {
            return Some(s.to_string());
        }

        if let Some(response) = value.get("response") {
            if let Some(body) = response.get("body").and_then(|b| b.as_str()) {
                return Some(body.to_string());
            }
            if let Some(content) = response.get("content").and_then(|c| c.as_str()) {
                return Some(content.to_string());
            }
            if let Some(raw) = response.as_str() {
                return Some(raw.to_string());
            }
        }

        if let Some(page_content) = value.get("pageContent").and_then(|p| p.as_str()) {
            return Some(page_content.to_string());
        }

        if let Some(html) = value.get("data").and_then(|d| d.as_str()) {
            return Some(html.to_string());
        }

        None
    }

    fn build_proxy_error(prefix: &str, status: &Option<String>, message: &Option<String>) -> CoreError {
        let mut msg = prefix.to_string();
        if let Some(status) = status {
            msg.push_str(&format!(" (status: {})", status));
        }
        if let Some(message) = message {
            msg.push_str(&format!(": {}", message));
        }
        CoreError::Parse(msg)
    }

    let mut last_error: Option<CoreError> = None;

    for attempt in 0..3 {
        let session = format!(
            "fiction_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            attempt
        );

        let payload = ProxyOptions {
            cmd: "request.get".to_string(),
            url: url.to_string(),
            session,
            max_timeout: Some(90000),
            return_raw: Some(true),
            browser: browser.clone(),
        };

        let res = client
            .post(&proxy_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json::<ProxyResponse>()
            .await?;

        let ProxyResponse { solution, status, message } = res;

        let result = (|| {
            let solution = solution.ok_or_else(|| build_proxy_error("Missing solution from proxy", &status, &message))?;

            let html = extract_html(&solution)
                .ok_or_else(|| build_proxy_error("Missing HTML in proxy response", &status, &message))?;

            Ok(html)
        })();

        match result {
            Ok(html) => return Ok(html),
            Err(err) => {
                if attempt == 2 || !matches!(err, CoreError::Parse(_)) {
                    return Err(err);
                }
                last_error = Some(err);
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| CoreError::Parse("Unknown proxy error".into())))
}

async fn fetch_direct(url: &str, client: &Client) -> Result<String> {
    let html = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(html)
}

pub async fn fetch_webnovel_csrf_token(_client: &Client) -> Result<String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| CoreError::Parse(format!("Failed to build client: {}", e)))?;

    let response = client
        .get("https://www.webnovel.com")
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await?
        .error_for_status()?;

    let cookies = response.headers().get("set-cookie");
    if let Some(cookie_header) = cookies {
        let cookie_str = cookie_header.to_str().unwrap_or("");
        let re = regex::Regex::new(r#"_csrfToken=([^;]+)"#).map_err(|_| CoreError::Parse("Failed to compile CSRF regex".into()))?;
        if let Some(caps) = re.captures(cookie_str) {
            if let Some(token) = caps.get(1) {
                return Ok(token.as_str().to_string());
            }
        }
    }

    let html = response.text().await?;

    let re = regex::Regex::new(r#"_csrfToken\s*=\s*([^;"']+)"#).map_err(|_| CoreError::Parse("Failed to compile CSRF regex".into()))?;
    if let Some(caps) = re.captures(&html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }

    Err(CoreError::Parse("Could not find _csrfToken".into()))
}

pub async fn fetch_webnovel_chapter_list(
    book_id: u64,
    page_index: u32,
    csrf_token: &str,
    client: &Client,
) -> Result<String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let url = format!(
        "https://www.webnovel.com/go/pcm/chapter/get-chapter-list?_csrfToken={}&bookId={}&pageIndex={}&_={}",
        urlencoding::encode(csrf_token),
        book_id,
        page_index,
        timestamp
    );

    let response = client
        .get(&url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", format!("https://www.webnovel.com/book/{}/catalog", book_id))
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_site_fanfiction() {
        let url = "https://www.fanfiction.net/s/1234567";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "fanfiction");
    }

    #[test]
    fn test_detect_site_archive() {
        let url = "https://archiveofourown.org/works/123456";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "archive");
    }

    #[test]
    fn test_detect_site_royalroad() {
        let url = "https://www.royalroad.com/fiction/12345";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "royalroad");
    }

    #[test]
    fn test_detect_site_spacebattles() {
        let url = "https://forums.spacebattles.com/threads/123456";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "spacebattles");
    }

    #[test]
    fn test_detect_site_webnovel() {
        let url = "https://www.webnovel.com/book/story_12345";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "webnovel");
    }

    #[test]
    fn test_detect_site_case_insensitive() {
        let url = "https://WWW.FANFICTION.NET/s/1234567";
        let result = detect_site_from_url(url).unwrap();
        assert_eq!(result, "fanfiction");
    }

    #[test]
    fn test_detect_site_invalid() {
        let url = "https://unknown-site.com/story/123";
        let result = detect_site_from_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_iso() {
        let result = parse_date("2024-01-15").unwrap();
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_date_fanfiction() {
        let result = parse_date("05-Jun-2012").unwrap();
        assert_eq!(result, "2012-06-05");
    }

    #[test]
    fn test_parse_date_royalroad_short() {
        let result = parse_date("Sep 02, 2010").unwrap();
        assert_eq!(result, "2010-09-02");
    }

    #[test]
    fn test_parse_date_royalroad_long() {
        let result = parse_date("January 15, 2024").unwrap();
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_date_different_order() {
        let result = parse_date("15 January 2024").unwrap();
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_date_slash_format() {
        let result = parse_date("2024/01/15").unwrap();
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_date_already_normalized() {
        let result = parse_date("2024-01-15").unwrap();
        assert_eq!(result, "2024-01-15");
    }

    #[test]
    fn test_parse_date_empty() {
        let result = parse_date("");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_date_whitespace() {
        let result = parse_date("  2024-01-15  ");
        assert_eq!(result, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_parse_date_invalid() {
        let result = parse_date("not-a-date");
        assert!(result.is_none());
    }
}
