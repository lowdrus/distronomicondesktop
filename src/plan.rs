use crate::{
    asset_select,
    config::{Config, tr},
    install, release, state, verify,
};
use reqwest::blocking::Client;
use std::{path::PathBuf, time::Duration};

pub struct UpdatePlan {
    pub current: Option<String>,
    pub release: release::Release,
    pub asset: release::Asset,
    pub checksum: Option<release::Asset>,
}

pub fn build(config: &Config) -> Result<(Client, UpdatePlan), String> {
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let existing = state::load(&state_path).map_err(|e| e.to_string())?;
    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let fetched = release::fetch_selected(
        &client,
        &config.github_host,
        &config.repo,
        token(config),
        config.allow_prerelease,
        &config.pinned_version,
        &release::Validators::default(),
    )?;
    let release = fetched.release.ok_or_else(|| {
        tr(
            config.language,
            "Nenhuma release disponível.",
            "No release available.",
        )
        .to_string()
    })?;
    let asset = asset_select::select(&release.assets, &config.asset_pattern)?.clone();
    let checksum = if config.skip_verification {
        None
    } else {
        Some(verify::select_asset(&release.assets, &config.checksum_pattern)?.clone())
    };
    let install_root = PathBuf::from(&config.install_root);
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| {
            existing
                .as_ref()
                .map(|s| s.latest_tag.clone())
                .filter(|s| !s.is_empty())
        });
    Ok((
        client,
        UpdatePlan {
            current,
            release,
            asset,
            checksum,
        },
    ))
}

pub fn dry_run(config: &Config) -> Result<String, String> {
    let (_, plan) = build(config)?;
    let current = plan
        .current
        .clone()
        .unwrap_or_else(|| tr(config.language, "nenhuma", "none").to_string());
    let checksum = plan
        .checksum
        .as_ref()
        .map(|a| a.name.as_str())
        .unwrap_or(tr(config.language, "ignorado", "skipped"));
    let pin = if config.pinned_version.trim().is_empty() {
        tr(config.language, "mais recente", "latest")
    } else {
        config.pinned_version.trim()
    };
    Ok(format!(
        "{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}",
        tr(
            config.language,
            "DRY RUN — nenhuma alteração foi feita.",
            "DRY RUN — no changes were made."
        ),
        tr(config.language, "Versão atual", "Current version"),
        current,
        tr(config.language, "Versão alvo", "Target version"),
        plan.release.tag_name,
        tr(config.language, "Versão fixada", "Pinned version"),
        pin,
        tr(config.language, "Asset selecionado", "Selected asset"),
        plan.asset.name,
        tr(config.language, "Checksum", "Checksum"),
        checksum,
        tr(config.language, "Destino", "Destination"),
        PathBuf::from(&config.install_root)
            .join(&config.app_name)
            .display(),
        tr(
            config.language,
            "A atualização só acontecerá ao clicar em Atualizar.",
            "The update will only happen when you click Update."
        )
    ))
}

fn token(config: &Config) -> Option<&str> {
    (!config.github_token.trim().is_empty()).then(|| config.github_token.trim())
}
