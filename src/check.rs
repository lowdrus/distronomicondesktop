use crate::{config::{tr, Config}, install, lock, release, state};
use reqwest::blocking::Client;
use std::{path::{Path, PathBuf}, time::Duration};

pub fn version(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_path = PathBuf::from(&config.state_root).join(&config.app_name).join("state.json");
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| state::load(&state_path).ok().flatten().map(|s| s.latest_tag).filter(|s| !s.is_empty()));
    Ok(match current {
        Some(tag) => format!("{}: {tag}", tr(config.language, "Versão instalada", "Installed version")),
        None => tr(config.language, "Nenhuma versão instalada.", "No version installed.").into(),
    })
}

pub fn unlock(config: &Config) -> Result<String, String> {
    let lock_path = PathBuf::from(&config.state_root).join(&config.app_name).join("lock");
    lock::force_unlock(&lock_path).map_err(|e| e.to_string())?;
    Ok(tr(config.language, "Lock removido.", "Lock removed.").into())
}

pub fn check(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_path = PathBuf::from(&config.state_root).join(&config.app_name).join("state.json");
    let existing = state::load(&state_path).map_err(|e| e.to_string())?;
    let previous = existing.as_ref().map(|s| release::Validators { etag: s.etag.clone(), last_modified: s.last_modified.clone() }).unwrap_or_default();

    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build().map_err(|e| e.to_string())?;

    let fetched = release::fetch_latest(&client, &config.github_host, &config.repo, token(config), config.allow_prerelease, &previous)?;
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| existing.as_ref().map(|s| s.latest_tag.clone()).filter(|s| !s.is_empty()));

    if let Some(ref current_tag) = current {
        save_validators(&state_path, existing.clone(), current_tag, &previous, &fetched.validators)?;
    }
    if fetched.not_modified {
        return Ok(current.map(|tag| format!("{}: {tag}", tr(config.language, "Atualizado", "Up to date")))
            .unwrap_or_else(|| tr(config.language, "Nenhuma versão instalada.", "No version installed.").into()));
    }
    let latest = fetched.release.ok_or_else(|| tr(config.language, "Nenhuma release disponível.", "No release available.").to_string())?;
    Ok(match current {
        Some(current) if current == latest.tag_name => format!("{}: {current}", tr(config.language, "Atualizado", "Up to date")),
        Some(current) => format!("{}: {} → {}\n{}", tr(config.language, "Atualização disponível", "Update available"), current, latest.tag_name, latest.html_url),
        None => format!("{}: {}\n{}", tr(config.language, "Instalação disponível", "Install available"), latest.tag_name, latest.html_url),
    })
}

fn save_validators(state_path: &Path, existing: Option<state::State>, current_tag: &str, previous: &release::Validators, fresh: &release::Validators) -> Result<(), String> {
    let mut value = existing.unwrap_or_default();
    value.latest_tag = current_tag.to_string();
    value.etag = if fresh.etag.is_empty() { previous.etag.clone() } else { fresh.etag.clone() };
    value.last_modified = if fresh.last_modified.is_empty() { previous.last_modified.clone() } else { fresh.last_modified.clone() };
    if value.installed_at_unix == 0 { value.installed_at_unix = state::now_unix(); }
    state::save_atomic(state_path, &value).map_err(|e| e.to_string())
}

fn token(config: &Config) -> Option<&str> { (!config.github_token.trim().is_empty()).then(|| config.github_token.trim()) }
