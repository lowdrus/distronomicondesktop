use crate::release::Asset;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use regex::Regex;
use sha2::{Digest, Sha256};

pub fn select_asset<'a>(assets: &'a [Asset], pattern: &str) -> Result<&'a Asset, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid asset pattern: {e}"))?;
    assets.iter().find(|a| regex.is_match(&a.name))
        .ok_or_else(|| format!("No release asset matches pattern: {pattern}"))
}

pub fn download_asset(client: &Client, token: Option<&str>, asset: &Asset) -> Result<Vec<u8>, String> {
    let mut request = client.get(&asset.url).header(ACCEPT, "application/octet-stream");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    let response = request.send().map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?;
    let bytes = response.bytes().map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

pub fn verify_sha256(
    client: &Client,
    token: Option<&str>,
    asset_name: &str,
    bytes: &[u8],
    checksum_asset: &Asset,
) -> Result<(), String> {
    let checksum_bytes = download_asset(client, token, checksum_asset)?;
    let text = String::from_utf8(checksum_bytes)
        .map_err(|_| "checksum asset is not UTF-8 text".to_string())?;

    let expected = parse_checksum(&text, asset_name)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());

    if actual.eq_ignore_ascii_case(&expected) {
        Ok(())
    } else {
        Err(format!("SHA-256 mismatch for {asset_name}: expected {expected}, got {actual}"))
    }
}

fn parse_checksum(text: &str, wanted: &str) -> Result<String, String> {
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if line.len() < 66 { continue; }
        let (hash, rest) = line.split_at(64);
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        let name = rest.trim_start_matches([' ', '*']).trim();
        if name == wanted { return Ok(hash.to_string()); }
    }
    Err(format!("Checksum entry not found for {wanted}"))
}
