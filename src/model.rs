use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub html_url: String,
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub latest_tag: String,
    pub etag: String,
    pub last_modified: String,
    pub installed_at_unix: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub repo: String,
    pub asset_filter: String,
    pub checksum_filter: String,
    pub install_root: PathBuf,
    pub allow_prerelease: bool,
    pub github_token: Option<String>,
    pub retain: usize,
    pub skip_verification: bool,
}
