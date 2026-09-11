use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED,
};
use serde::Deserialize;
use std::cmp::Reverse;

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub html_url: String,
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct Validators {
    pub etag: String,
    pub last_modified: String,
}

pub struct FetchResult {
    pub release: Option<Release>,
    pub validators: Validators,
    pub not_modified: bool,
}

pub fn fetch_latest(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    allow_prerelease: bool,
    previous: &Validators,
) -> Result<FetchResult, String> {
    let host = host.trim_end_matches('/');
    let url = if allow_prerelease {
        format!("{host}/repos/{repo}/releases")
    } else {
        format!("{host}/repos/{repo}/releases/latest")
    };

    let mut request = client
        .get(url)
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    if !previous.etag.is_empty() {
        request = request.header(IF_NONE_MATCH, &previous.etag);
    }
    if !previous.last_modified.is_empty() {
        request = request.header(IF_MODIFIED_SINCE, &previous.last_modified);
    }

    let response = request.send().map_err(|e| e.to_string())?;
    let validators = Validators {
        etag: response
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
        last_modified: response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
    };

    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(FetchResult {
            release: None,
            validators,
            not_modified: true,
        });
    }

    let response = response.error_for_status().map_err(|e| e.to_string())?;
    let release = if allow_prerelease {
        let mut releases: Vec<Release> = response.json().map_err(|e| e.to_string())?;
        releases.retain(|r| !r.draft);
        releases.sort_by_key(|r| Reverse(r.created_at.clone()));
        releases
            .into_iter()
            .next()
            .ok_or_else(|| "No GitHub release found".to_string())?
    } else {
        response.json::<Release>().map_err(|e| e.to_string())?
    };

    Ok(FetchResult {
        release: Some(release),
        validators,
        not_modified: false,
    })
}
