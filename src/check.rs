use crate::{
    config::{Config, tr},
    install, lock, release, restart, state,
};
use reqwest::blocking::Client;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn version(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| {
            state::load(&state_path)
                .ok()
                .flatten()
                .map(|s| s.latest_tag)
                .filter(|s| !s.is_empty())
        });
    Ok(match current {
        Some(tag) => format!(
            "{}: {tag}",
            tr(config.language, "Versão instalada", "Installed version")
        ),
        None => tr(
            config.language,
            "Nenhuma versão instalada.",
            "No version installed.",
        )
        .into(),
    })
}

pub fn rollback(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| {
            tr(
                config.language,
                "Nenhuma versão instalada para rollback.",
                "No installed version available for rollback.",
            )
            .to_string()
        })?;
    let releases =
        install::list_releases(&install_root, &config.app_name).map_err(|e| e.to_string())?;
    let previous = releases
        .into_iter()
        .find(|tag| tag != &current)
        .ok_or_else(|| {
            tr(
                config.language,
                "Nenhuma versão anterior disponível.",
                "No previous version is available.",
            )
            .to_string()
        })?;

    install::activate_release(&install_root, &config.app_name, &previous)
        .map_err(|e| e.to_string())?;
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let mut value = state::load(&state_path)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    value.latest_tag = previous.clone();
    value.installed_at_unix = state::now_unix();
    state::save_atomic(&state_path, &value).map_err(|e| e.to_string())?;

    if !config.restart_command.is_empty() {
        restart::execute(&config.restart_command).map_err(|e| {
            format!(
                "{}: {e}",
                tr(
                    config.language,
                    "Rollback concluído, mas o reinício falhou",
                    "Rollback completed, but restart failed"
                )
            )
        })?;
    }

    Ok(format!(
        "{}: {current} → {previous}",
        tr(config.language, "Rollback concluído", "Rollback completed")
    ))
}

pub fn doctor(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let app_root = install_root.join(&config.app_name);
    let releases_dir = app_root.join("releases");
    let bin_dir = app_root.join("bin");
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let current =
        install::current_tag(&install_root, &config.app_name).map_err(|e| e.to_string())?;
    let releases =
        install::list_releases(&install_root, &config.app_name).map_err(|e| e.to_string())?;
    let state_value = state::load(&state_path).map_err(|e| e.to_string())?;
    let mut problems = Vec::new();

    match &current {
        Some(tag) if !releases_dir.join(tag).is_dir() => problems.push(tr(
            config.language,
            "current.txt aponta para uma release ausente",
            "current.txt points to a missing release",
        )),
        None => problems.push(tr(
            config.language,
            "Nenhuma versão atual registrada",
            "No current version is recorded",
        )),
        _ => {}
    }
    if current.is_some() && !bin_dir.is_dir() {
        problems.push(tr(
            config.language,
            "Diretório bin ausente",
            "bin directory is missing",
        ));
    }
    if bin_dir.is_dir()
        && fs::read_dir(&bin_dir)
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
    {
        problems.push(tr(
            config.language,
            "Diretório bin está vazio",
            "bin directory is empty",
        ));
    }
    if let (Some(current_tag), Some(saved)) = (&current, &state_value)
        && !saved.latest_tag.is_empty()
        && saved.latest_tag != *current_tag
    {
        problems.push(tr(
            config.language,
            "state.json e current.txt estão divergentes",
            "state.json and current.txt disagree",
        ));
    }

    if problems.is_empty() {
        Ok(format!(
            "{}\n{}: {}\n{}: {}",
            tr(config.language, "Diagnóstico OK.", "Doctor OK."),
            tr(config.language, "Versão atual", "Current version"),
            current.unwrap_or_else(|| "-".into()),
            tr(config.language, "Releases instaladas", "Installed releases"),
            releases.len()
        ))
    } else {
        Ok(format!(
            "{}\n- {}",
            tr(
                config.language,
                "Diagnóstico encontrou problemas:",
                "Doctor found problems:"
            ),
            problems.join("\n- ")
        ))
    }
}

pub fn unlock(config: &Config) -> Result<String, String> {
    let lock_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("lock");
    lock::force_unlock(&lock_path).map_err(|e| e.to_string())?;
    Ok(tr(config.language, "Lock removido.", "Lock removed.").into())
}

pub fn check(config: &Config) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("state.json");
    let existing = state::load(&state_path).map_err(|e| e.to_string())?;
    let previous = existing
        .as_ref()
        .map(|s| release::Validators {
            etag: s.etag.clone(),
            last_modified: s.last_modified.clone(),
        })
        .unwrap_or_default();

    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

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

    if let Some(ref current_tag) = current {
        save_validators(
            &state_path,
            existing.clone(),
            current_tag,
            &previous,
            &fetched.validators,
        )?;
    }
    if fetched.not_modified {
        return Ok(current
            .map(|tag| format!("{}: {tag}", tr(config.language, "Atualizado", "Up to date")))
            .unwrap_or_else(|| {
                tr(
                    config.language,
                    "Nenhuma versão instalada.",
                    "No version installed.",
                )
                .into()
            }));
    }
    let latest = fetched.release.ok_or_else(|| {
        tr(
            config.language,
            "Nenhuma release disponível.",
            "No release available.",
        )
        .to_string()
    })?;
    Ok(match current {
        Some(current) if current == latest.tag_name => format!(
            "{}: {current}",
            tr(config.language, "Atualizado", "Up to date")
        ),
        Some(current) => format!(
            "{}: {} → {}\n{}",
            tr(
                config.language,
                "Atualização disponível",
                "Update available"
            ),
            current,
            latest.tag_name,
            latest.html_url
        ),
        None => format!(
            "{}: {}\n{}",
            tr(
                config.language,
                "Instalação disponível",
                "Install available"
            ),
            latest.tag_name,
            latest.html_url
        ),
    })
}

fn save_validators(
    state_path: &Path,
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
    if value.installed_at_unix == 0 {
        value.installed_at_unix = state::now_unix();
    }
    state::save_atomic(state_path, &value).map_err(|e| e.to_string())
}

fn token(config: &Config) -> Option<&str> {
    (!config.github_token.trim().is_empty()).then(|| config.github_token.trim())
}
