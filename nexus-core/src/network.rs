use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::{Result, CoreError};


#[derive(Serialize)]
struct ProxyRequest {
    cmd: String,
    url: String,
    session: String,
    max_timeout: u32,
}

#[derive(Deserialize)]
struct ProxyResponse {
    solution: Option<serde_json::Value>,
}

pub async fn fetch_via_proxy(url: &str, client: &Client) -> Result<String> {
    let proxy_url = "http://localhost:8191/v1";
    let payload = ProxyRequest {
        cmd: "request.get".to_string(),
        url: url.to_string(),
        max_timeout: 60000,
        session: "fiction".into(),
        };

    let res = client
        .post(proxy_url)
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

