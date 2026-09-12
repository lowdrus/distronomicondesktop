use crate::release::Asset;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path, thread, time::Duration};

pub fn select_asset<'a>(assets: &'a [Asset], pattern: &str) -> Result<&'a Asset, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("Invalid asset pattern: {e}"))?;
    assets
        .iter()
        .find(|a| regex.is_match(&a.name))
        .ok_or_else(|| format!("No release asset matches pattern: {pattern}"))
}

fn request_asset(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
) -> reqwest::blocking::RequestBuilder {
    let mut request = client
        .get(&asset.url)
        .header(ACCEPT, "application/octet-stream");
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    request
}

fn send_with_retry(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
) -> Result<reqwest::blocking::Response, String> {
    let mut delay = Duration::from_millis(500);
    let mut last_error = String::new();
    for attempt in 0..=3 {
        match request_asset(client, token, asset).send() {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response) if response.status().is_client_error() => {
                return Err(format!("HTTP {} for {}", response.status(), asset.name));
            }
            Ok(response) => last_error = format!("HTTP {} for {}", response.status(), asset.name),
            Err(error) => last_error = error.to_string(),
        }
        if attempt < 3 {
            thread::sleep(delay);
            delay = (delay * 2).min(Duration::from_secs(4));
        }
    }
    Err(last_error)
}

pub fn verify_sha256(
    client: &Client,
    token: Option<&str>,
    asset_name: &str,
    downloaded_path: &Path,
    checksum_asset: &Asset,
) -> Result<(), String> {
    let text = send_with_retry(client, token, checksum_asset)?
        .text()
        .map_err(|e| e.to_string())?;
    let expected = parse_checksum(&text, asset_name)?;
    let mut file = File::open(downloaded_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual.eq_ignore_ascii_case(&expected) {
        Ok(())
    } else {
        Err(format!(
            "SHA-256 mismatch for {asset_name}: expected {expected}, got {actual}"
        ))
    }
}

fn parse_checksum(text: &str, wanted: &str) -> Result<String, String> {
    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r').trim_start();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.len() < 66 {
            return Err(format!("Invalid checksum line: {line}"));
        }

        let (hash, rest) = line.split_at(64);
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Invalid SHA-256 value: {hash}"));
        }

        let filename = if let Some(name) = rest.strip_prefix("  ") {
            name
        } else if let Some(name) = rest.strip_prefix(" *") {
            name
        } else {
            return Err(format!("Invalid checksum separator: {rest}"));
        };
        if filename.is_empty() {
            return Err("Checksum filename is empty".into());
        }
        if filename == wanted {
            return Ok(hash.to_string());
        }
    }
    Err(format!("Checksum entry not found for {wanted}"))
}

#[cfg(test)]
mod tests {
    use super::parse_checksum;

    #[test]
    fn parses_two_space_format() {
        let text = format!("{}  app.zip", "a".repeat(64));
        assert_eq!(parse_checksum(&text, "app.zip").unwrap(), "a".repeat(64));
    }

    #[test]
    fn parses_asterisk_format() {
        let text = format!("{} *app.exe", "b".repeat(64));
        assert_eq!(parse_checksum(&text, "app.exe").unwrap(), "b".repeat(64));
    }

    #[test]
    fn accepts_comments_whitespace_and_crlf() {
        let text = format!("  # comment\r\n{}  app.zip\r\n", "c".repeat(64));
        assert_eq!(parse_checksum(&text, "app.zip").unwrap(), "c".repeat(64));
    }

    #[test]
    fn rejects_short_or_invalid_lines() {
        assert!(parse_checksum("abc  app.zip", "app.zip").is_err());
        let invalid = format!("{} app.zip", "d".repeat(64));
        assert!(parse_checksum(&invalid, "app.zip").is_err());
    }

    #[test]
    fn reports_missing_asset() {
        let text = format!("{}  other.zip", "e".repeat(64));
        assert!(parse_checksum(&text, "app.zip").is_err());
    }
}
