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
    pub prerelease: bool,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub size: u64,
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

pub fn fetch_selected(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    allow_prerelease: bool,
    pinned_version: &str,
    previous: &Validators,
) -> Result<FetchResult, String> {
    fetch_selected_channel(
        client,
        host,
        repo,
        token,
        if allow_prerelease { "nightly" } else { "stable" },
        pinned_version,
        previous,
    )
}

pub fn fetch_selected_channel(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    channel: &str,
    pinned_version: &str,
    previous: &Validators,
) -> Result<FetchResult, String> {
    let pin = pinned_version.trim();
    if !pin.is_empty() && !pin.eq_ignore_ascii_case("latest") {
        return match fetch_tag(client, host, repo, token, pin) {
            Ok(result) => Ok(result),
            Err(first_error) if !pin.starts_with(['v', 'V']) => {
                let prefixed = format!("v{pin}");
                fetch_tag(client, host, repo, token, &prefixed).map_err(|second_error| {
                    format!(
                        "Could not find pinned release '{pin}' or '{prefixed}': {first_error}; {second_error}"
                    )
                })
            }
            Err(error) => Err(error),
        };
    }

    match channel {
        "stable" => {
            let first = fetch_latest(client, host, repo, token, false, previous)?;
            if first.not_modified {
                fetch_latest(client, host, repo, token, false, &Validators::default())
            } else {
                Ok(first)
            }
        }
        "beta" => fetch_from_list(client, host, repo, token, Some(true)),
        "nightly" => fetch_from_list(client, host, repo, token, None),
        _ => Err(format!("Unknown release channel: {channel}")),
    }
}

pub fn fetch_tag(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    tag: &str,
) -> Result<FetchResult, String> {
    let host = host.trim_end_matches('/');
    let url = format!("{host}/repos/{repo}/releases/tags/{}", encode_tag(tag));
    let mut request = client
        .get(url)
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    let response = request
        .send()
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let release = response.json::<Release>().map_err(|e| e.to_string())?;
    Ok(FetchResult {
        release: Some(release),
        validators: Validators::default(),
        not_modified: false,
    })
}

pub fn fetch_latest(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    allow_prerelease: bool,
    previous: &Validators,
) -> Result<FetchResult, String> {
    if allow_prerelease {
        return fetch_from_list(client, host, repo, token, None);
    }
    let host = host.trim_end_matches('/');
    let url = format!("{host}/repos/{repo}/releases/latest");
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
        etag: response.headers().get(ETAG).and_then(|v| v.to_str().ok()).unwrap_or("").to_string(),
        last_modified: response.headers().get(LAST_MODIFIED).and_then(|v| v.to_str().ok()).unwrap_or("").to_string(),
    };
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(FetchResult { release: None, validators, not_modified: true });
    }
    let response = response.error_for_status().map_err(|e| e.to_string())?;
    let release = response.json::<Release>().map_err(|e| e.to_string())?;
    Ok(FetchResult { release: Some(release), validators, not_modified: false })
}

fn fetch_from_list(
    client: &Client,
    host: &str,
    repo: &str,
    token: Option<&str>,
    prerelease_only: Option<bool>,
) -> Result<FetchResult, String> {
    let host = host.trim_end_matches('/');
    let url = format!("{host}/repos/{repo}/releases?per_page=100");
    let mut request = client
        .get(url)
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    let response = request.send().map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?;
    let mut releases: Vec<Release> = response.json().map_err(|e| e.to_string())?;
    releases.retain(|r| !r.draft);
    if let Some(required) = prerelease_only {
        releases.retain(|r| r.prerelease == required);
    }
    releases.sort_by_key(|r| Reverse(r.published_at.clone().or_else(|| r.created_at.clone())));
    let release = releases.into_iter().next().ok_or_else(|| "No GitHub release found for selected channel".to_string())?;
    Ok(FetchResult { release: Some(release), validators: Validators::default(), not_modified: false })
}

fn encode_tag(tag: &str) -> String {
    let mut encoded = String::with_capacity(tag.len());
    for byte in tag.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::encode_tag;

    #[test]
    fn encodes_slashes_in_tag_names() {
        assert_eq!(encode_tag("release/1.3"), "release%2F1.3");
    }
}
