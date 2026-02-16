use crate::error::{CoreError, Result};
use crate::models::Story;
use crate::sites::get_site;
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct ResolvedStoryUrl {
    pub site: String,
    pub normalized_url: String,
}

pub fn resolve_story_url(url: &str, site_override: Option<&str>) -> Result<ResolvedStoryUrl> {
    let parsed = Url::parse(url)
        .map_err(|e| CoreError::InvalidUrl(format!("Invalid URL '{}': {}", url, e)))?;

    let site = match site_override {
        Some(site) => site.to_string(),
        None => detect_site(&parsed)?,
    };

    let normalized_url = normalize_story_url(&site, &parsed)?;

    Ok(ResolvedStoryUrl { site, normalized_url })
}

pub async fn fetch_story_data_from_url(
    url: &str,
    site_override: Option<&str>,
    client: &reqwest::Client,
) -> Result<Story> {
    let resolved = resolve_story_url(url, site_override)?;
    let site = get_site(&resolved.site)?;
    site.get_story_data_from_url(&resolved.normalized_url, client).await
}

fn detect_site(url: &Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or_else(|| CoreError::InvalidUrl("URL has no host".into()))?
        .to_lowercase();

    if host.contains("fanfiction.net") {
        return Ok("fanfiction".into());
    }
    if host.contains("royalroad.com") {
        return Ok("royalroad".into());
    }
    if host.contains("archiveofourown.org") {
        return Ok("archive".into());
    }
    if host.contains("spacebattles.com") {
        return Ok("spacebattles".into());
    }

    Err(CoreError::InvalidUrl(format!(
        "Unsupported host '{}'. Supported: fanfiction.net, royalroad.com, archiveofourown.org, spacebattles.com",
        host
    )))
}

fn normalize_story_url(site: &str, url: &Url) -> Result<String> {
    let path = url.path().trim_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    match site {
        "fanfiction" => {
            // Accept /s/{story_id}/... and normalize to chapter-1 URL.
            if segments.first() != Some(&"s") {
                return Err(CoreError::InvalidUrl(
                    "FanFiction URL must look like /s/{story_id}/...".into(),
                ));
            }
            let story_id = segments
                .get(1)
                .ok_or_else(|| CoreError::InvalidUrl("Missing FanFiction story_id".into()))?;
            parse_u64(story_id, "fanfiction story_id")?;
            Ok(format!("https://www.fanfiction.net/s/{}/1", story_id))
        }
        "royalroad" => {
            // Accept /fiction/{story_id}/{slug?}
            if segments.first() != Some(&"fiction") {
                return Err(CoreError::InvalidUrl(
                    "RoyalRoad URL must look like /fiction/{story_id}/{slug}".into(),
                ));
            }
            let story_id = segments
                .get(1)
                .ok_or_else(|| CoreError::InvalidUrl("Missing RoyalRoad story_id".into()))?;
            parse_u64(story_id, "royalroad story_id")?;
            let slug = segments.get(2).copied().unwrap_or("story");
            Ok(format!("https://www.royalroad.com/fiction/{}/{}", story_id, slug))
        }
        "archive" => {
            // Accept /works/{story_id}/...
            if segments.first() != Some(&"works") {
                return Err(CoreError::InvalidUrl(
                    "Archive URL must look like /works/{story_id}".into(),
                ));
            }
            let story_id = segments
                .get(1)
                .ok_or_else(|| CoreError::InvalidUrl("Missing Archive work id".into()))?;
            parse_u64(story_id, "archive work id")?;
            Ok(format!("https://archiveofourown.org/works/{}", story_id))
        }
        "spacebattles" => {
            // Accept /threads/{slug.id}/...
            if segments.first() != Some(&"threads") {
                return Err(CoreError::InvalidUrl(
                    "SpaceBattles URL must look like /threads/{slug.id}".into(),
                ));
            }
            let story_segment = segments
                .get(1)
                .ok_or_else(|| CoreError::InvalidUrl("Missing SpaceBattles thread segment".into()))?;
            let (_slug, id) = split_slug_id(story_segment)?;
            Ok(format!("https://forums.spacebattles.com/threads/{}.{}", _slug, id))
        }
        other => Err(CoreError::UnknownSite(other.to_string())),
    }
}

fn parse_u64(v: &str, label: &str) -> Result<u64> {
    v.parse::<u64>()
        .map_err(|_| CoreError::InvalidUrl(format!("Invalid {}: '{}'", label, v)))
}

fn split_slug_id(input: &str) -> Result<(&str, u64)> {
    let (slug, id_str) = input.rsplit_once('.').ok_or_else(|| {
        CoreError::InvalidUrl(format!(
            "Expected slug.id format for SpaceBattles thread segment, got '{}'",
            input
        ))
    })?;
    let id = parse_u64(id_str, "spacebattles thread id")?;
    Ok((slug, id))
}
