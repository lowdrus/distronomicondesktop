use crate::release::Asset;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, RANGE};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

fn request(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
    offset: u64,
) -> reqwest::blocking::RequestBuilder {
    let mut req = client
        .get(&asset.url)
        .header(ACCEPT, "application/octet-stream");
    if offset > 0 {
        req = req.header(RANGE, format!("bytes={offset}-"));
    }
    if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
        req = req.header(AUTHORIZATION, format!("Bearer {}", token.trim()));
    }
    req
}

fn send(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
    offset: u64,
) -> Result<reqwest::blocking::Response, String> {
    let mut delay = Duration::from_millis(500);
    let mut last = String::new();
    for attempt in 0..=3 {
        match request(client, token, asset, offset).send() {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response) if response.status().is_client_error() => {
                return Err(format!("HTTP {} for {}", response.status(), asset.name));
            }
            Ok(response) => last = format!("HTTP {} for {}", response.status(), asset.name),
            Err(error) => last = error.to_string(),
        }
        if attempt < 3 {
            thread::sleep(delay);
            delay = (delay * 2).min(Duration::from_secs(4));
        }
    }
    Err(last)
}

pub fn download(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
    destination: &Path,
) -> Result<bool, String> {
    let cancel = AtomicBool::new(false);
    download_cancellable(client, token, asset, destination, &cancel)
}

pub fn download_cancellable(
    client: &Client,
    token: Option<&str>,
    asset: &Asset,
    destination: &Path,
    cancel: &AtomicBool,
) -> Result<bool, String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let offset = destination.metadata().map(|m| m.len()).unwrap_or(0);
    let mut response = match send(client, token, asset, offset) {
        Ok(response) => response,
        Err(_) if offset > 0 => {
            let _ = std::fs::remove_file(destination);
            send(client, token, asset, 0)?
        }
        Err(error) => return Err(error),
    };
    let resumed = offset > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    let mut file = if resumed {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(destination)
            .map_err(|e| e.to_string())?
    } else {
        File::create(destination).map_err(|e| e.to_string())?
    };
    let mut buffer = [0u8; 128 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            file.sync_all().map_err(|e| e.to_string())?;
            return Err("DOWNLOAD_CANCELLED".into());
        }
        let count = response.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        file.write_all(&buffer[..count])
            .map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    Ok(resumed)
}
