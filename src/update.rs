use crate::{
    config::{Config, Language, tr},
    install, lock, release, restart, state, verify,
};
use reqwest::blocking::Client;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

pub fn run(config: &Config, status: &Arc<Mutex<String>>) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_root = PathBuf::from(&config.state_root);
    let app_state_dir = state_root.join(&config.app_name);
    let state_path = app_state_dir.join("state.json");
    let lock_path = app_state_dir.join("lock");
    let _guard = lock::acquire(&lock_path, Duration::from_secs(30)).map_err(|e| {
        format!(
            "{}: {e}",
            tr(
                config.language,
                "Não foi possível adquirir o lock",
                "Could not acquire update lock"
            )
        )
    })?;

    set_phase(
        status,
        config.language,
        "Consultando releases no GitHub...",
        "Checking GitHub releases...",
    );
    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    let existing = state::load(&state_path).map_err(|e| e.to_string())?;
    let previous = existing
        .as_ref()
        .map(|s| release::Validators {
            etag: s.etag.clone(),
            last_modified: s.last_modified.clone(),
        })
        .unwrap_or_default();
    let fetched = release::fetch_latest(
        &client,
        &config.github_host,
        &config.repo,
        token(config),
        config.allow_prerelease,
        &previous,
    )?;
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| {
            existing
                .as_ref()
                .map(|s| s.latest_tag.clone())
                .filter(|s| !s.is_empty())
        });

    if fetched.not_modified {
        return current
            .map(|tag| {
                format!(
                    "{}: {tag}",
                    tr(config.language, "Já está atualizado", "Already up to date")
                )
            })
            .ok_or_else(|| {
                tr(
                    config.language,
                    "O GitHub não retornou uma release instalável.",
                    "GitHub did not return an installable release.",
                )
                .into()
            });
    }

    let latest = fetched.release.ok_or_else(|| {
        tr(
            config.language,
            "Nenhuma release disponível.",
            "No release available.",
        )
        .to_string()
    })?;
    if current.as_deref() == Some(latest.tag_name.as_str()) {
        save_state(
            &state_path,
            existing,
            &latest.tag_name,
            &previous,
            &fetched.validators,
        )?;
        return Ok(format!(
            "{}: {}",
            tr(config.language, "Já está atualizado", "Already up to date"),
            latest.tag_name
        ));
    }

    let asset = verify::select_asset(&latest.assets, &config.asset_pattern)?.clone();
    let temp_dir = app_state_dir.join("downloads");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let temp_path = temp_dir.join(format!(
        "{}-{}",
        safe_name(&latest.tag_name),
        safe_name(&asset.name)
    ));

    let result = (|| -> Result<String, String> {
        set_phase(
            status,
            config.language,
            "Baixando release...",
            "Downloading release...",
        );
        verify::download_asset_to_file(&client, token(config), &asset, &temp_path)?;

        if !config.skip_verification {
            set_phase(
                status,
                config.language,
                "Verificando SHA-256...",
                "Verifying SHA-256...",
            );
            let checksum = verify::select_asset(&latest.assets, &config.checksum_pattern)?.clone();
            verify::verify_sha256(&client, token(config), &asset.name, &temp_path, &checksum)?;
        }

        set_phase(
            status,
            config.language,
            "Instalando de forma segura...",
            "Installing safely...",
        );
        let installed = install::install_release(
            &install_root,
            &config.app_name,
            &latest.tag_name,
            &asset.name,
            &temp_path,
        )?;

        let restart_error = if config.restart_command.is_empty() {
            None
        } else {
            restart::execute(&config.restart_command)
                .err()
                .map(|e| e.to_string())
        };

        let deleted = install::prune_old_releases(
            &install_root.join(&config.app_name).join("releases"),
            &latest.tag_name,
            config.retain,
        )
        .map_err(|e| e.to_string())?;

        save_state(
            &state_path,
            existing,
            &latest.tag_name,
            &previous,
            &fetched.validators,
        )?;

        let mut message = format!(
            "{}: {}\n{}: {}",
            tr(config.language, "Atualização concluída", "Update completed"),
            latest.tag_name,
            tr(config.language, "Instalada em", "Installed at"),
            installed.display()
        );
        if !deleted.is_empty() {
            message.push_str(&format!(
                "\n{}: {}",
                tr(
                    config.language,
                    "Versões antigas removidas",
                    "Old releases removed"
                ),
                deleted.join(", ")
            ));
        }
        if let Some(error) = restart_error {
            message.push_str(&format!(
                "\n{}: {error}",
                tr(
                    config.language,
                    "Aviso: o comando de reinício falhou",
                    "Warning: restart command failed"
                )
            ));
        }
        Ok(message)
    })();
    let _ = fs::remove_file(&temp_path);
    result
}

fn save_state(
    path: &std::path::Path,
    existing: Option<state::State>,
    current_tag: &str,
    previous: &release::Validators,
    fresh: &release::Validators,
) -> Result<(), String> {
    let mut value = existing.unwrap_or_default();
    value.latest_tag = current_tag.to_string();
    value.etag = if fresh.etag.is_empty() {
        previous.etag.clone()
    } else {
        fresh.etag.clone()
    };
    value.last_modified = if fresh.last_modified.is_empty() {
        previous.last_modified.clone()
    } else {
        fresh.last_modified.clone()
    };
    value.installed_at_unix = state::now_unix();
    state::save_atomic(path, &value).map_err(|e| e.to_string())
}

fn token(config: &Config) -> Option<&str> {
    (!config.github_token.trim().is_empty()).then(|| config.github_token.trim())
}
fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect()
}
fn set_phase(status: &Arc<Mutex<String>>, language: Language, pt: &'static str, en: &'static str) {
    if let Ok(mut text) = status.lock() {
        *text = tr(language, pt, en).into();
    }
}
