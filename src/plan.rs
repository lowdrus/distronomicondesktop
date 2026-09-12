use crate::{
    asset_select,
    config::{Config, tr},
    features_v14, install, release, state, verify,
};
use reqwest::blocking::Client;
use std::{path::PathBuf, time::Duration};

pub struct UpdatePlan {
    pub current: Option<String>,
    pub release: release::Release,
    pub asset: release::Asset,
    pub checksum: Option<release::Asset>,
    pub validators: release::Validators,
}

pub fn build(config: &Config) -> Result<(Client, UpdatePlan), String> {
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let existing = state::load(&state_path).unwrap_or(None);
    let previous = existing
        .as_ref()
        .map(|value| release::Validators {
            etag: value.etag.clone(),
            last_modified: value.last_modified.clone(),
        })
        .unwrap_or_default();
    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    features_v14::test_connectivity(&client, config)?;
    let fetched = release::fetch_selected_channel(
        &client,
        &config.github_host,
        &config.repo,
        token(config),
        &config.channel,
        &config.pinned_version,
        &previous,
    )?;
    let validators = fetched.validators.clone();
    let release = fetched.release.ok_or_else(|| {
        tr(config.language, "Nenhuma release disponível.", "No release available.").to_string()
    })?;
    let asset = asset_select::select_for_arch(&release.assets, &config.asset_pattern, &config.architecture)?.clone();
    let checksum = if config.skip_verification {
        None
    } else {
        Some(verify::select_asset(&release.assets, &config.checksum_pattern)?.clone())
    };
    let install_root = PathBuf::from(&config.install_root);
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| existing.as_ref().map(|s| s.latest_tag.clone()).filter(|s| !s.is_empty()));
    Ok((client, UpdatePlan { current, release, asset, checksum, validators }))
}

pub fn dry_run(config: &Config) -> Result<String, String> {
    let (_, plan) = build(config)?;
    let current = plan.current.clone().unwrap_or_else(|| tr(config.language, "nenhuma", "none").to_string());
    let checksum = plan.checksum.as_ref().map(|a| a.name.as_str()).unwrap_or(tr(config.language, "ignorado", "skipped"));
    let pin = if config.pinned_version.trim().is_empty() { tr(config.language, "mais recente", "latest") } else { config.pinned_version.trim() };
    let releases_dir = PathBuf::from(&config.install_root).join(&config.app_name).join("releases");
    let prune = install::preview_prune_after_install(&releases_dir, &plan.release.tag_name, config.retain).map_err(|e| e.to_string())?;
    let prune_text = if prune.is_empty() { tr(config.language, "nenhuma", "none").to_string() } else { prune.join(", ") };
    let required = features_v14::estimated_required_bytes(&plan.asset);
    let action = if plan.current.as_deref() == Some(plan.release.tag_name.as_str()) {
        tr(config.language, "Nenhuma instalação necessária; a versão alvo já está ativa.", "No installation is required; the target version is already active.")
    } else {
        tr(config.language, "A atualização só acontecerá ao clicar em Atualizar.", "The update will only happen when you click Update.")
    };
    let downgrade = plan.current.as_deref().is_some_and(|c| features_v14::is_downgrade(c, &plan.release.tag_name));
    Ok(format!(
        "{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}",
        tr(config.language, "PRÉVIA — nenhuma alteração foi feita.", "DRY RUN — no changes were made."),
        tr(config.language, "Versão atual", "Current version"), current,
        tr(config.language, "Versão alvo", "Target version"), plan.release.tag_name,
        tr(config.language, "Canal", "Channel"), config.channel,
        tr(config.language, "Arquitetura", "Architecture"), if config.architecture == "auto" { asset_select::detect_architecture() } else { &config.architecture },
        tr(config.language, "Versão fixada", "Pinned version"), pin,
        tr(config.language, "Asset selecionado", "Selected asset"), plan.asset.name,
        tr(config.language, "Tamanho estimado necessário", "Estimated required size"), features_v14::format_size(required),
        tr(config.language, "Checksum", "Checksum"), checksum,
        tr(config.language, "Possível downgrade", "Possible downgrade"), if downgrade { tr(config.language, "sim", "yes") } else { tr(config.language, "não", "no") },
        tr(config.language, "Versões que serão removidas pela retenção por quantidade", "Releases that count retention will remove"), prune_text,
        action
    ))
}

pub fn release_notes(config: &Config) -> Result<String, String> {
    let (_, plan) = build(config)?;
    let body = if plan.release.body.trim().is_empty() { tr(config.language, "Sem notas de release.", "No release notes.") } else { plan.release.body.trim() };
    Ok(format!("{} — {}\n\n{}", plan.release.tag_name, plan.release.html_url, body))
}

fn token(config: &Config) -> Option<&str> {
    (!config.github_token.trim().is_empty()).then(|| config.github_token.trim())
}
