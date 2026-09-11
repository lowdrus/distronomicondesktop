use crate::release::Asset;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{fs::File, io::{self, Read}, path::Path};

pub fn select_asset<'a>(assets: &'a [Asset], pattern: &str) -> Result<&'a Asset, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid asset pattern: {e}"))?;
    assets.iter().find(|a| regex.is_match(&a.name))
        .ok_or_else(|| format!("No release asset matches pattern: {pattern}"))
}

fn request_asset(client: &Client, token: Option<&str>, asset: &Asset) -> reqwest::blocking::RequestBuilder {
    let mut request = client.get(&asset.url).header(ACCEPT, "application/octet-stream");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    request
}

pub fn download_asset_to_file(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
    destination: &Path,
) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut response = request_asset(client, token, asset)
        .send().map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?;
    let mut file = File::create(destination).map_err(|e| e.to_string())?;
    io::copy(&mut response, &mut file).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn verify_sha256(
    client: &Client,
    token: Option<&str>,
    asset_name: &str,
    downloaded_path: &Path,
    checksum_asset: &Asset,
) -> Result<(), String> {
    let text = request_asset(client, token, checksum_asset)
        .send().map_err(|e| e.to_string())?
        .error_for_status().map_err(|e| e.to_string())?
        .text().map_err(|e| e.to_string())?;

    let expected = parse_checksum(&text, asset_name)?;
    let mut file = File::open(downloaded_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
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
        let name = rest.trim_start_matches(|c| c == ' ' || c == '*').trim();
        if name == wanted { return Ok(hash.to_string()); }
    }
    Err(format!("Checksum entry not found for {wanted}"))
}
