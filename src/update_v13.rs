use crate::{
    config::{Config, Language, tr},
    download_resume, history::{self, HistoryEntry}, install, lock, plan, restart, state, verify,
};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::Duration};

pub fn run(config: &Config, status: &Arc<Mutex<String>>) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_root = PathBuf::from(&config.state_root);
    let app_state_dir = state_root.join(&config.app_name);
    let state_path = app_state_dir.join("state.json");
    let history_path = app_state_dir.join("history.json");
    let lock_path = app_state_dir.join("lock");
    let _guard = lock::acquire(&lock_path, Duration::from_secs(30)).map_err(|e| format!("{}: {e}", tr(config.language, "Não foi possível adquirir o lock", "Could not acquire update lock")))?;

    set_phase(status, config.language, "Planejando atualização...", "Planning update...");
    let (client, plan) = plan::build(config)?;
    let old = plan.current.clone().unwrap_or_default();
    if plan.current.as_deref() == Some(plan.release.tag_name.as_str()) {
        return Ok(format!("{}: {}", tr(config.language, "Já está atualizado", "Already up to date"), plan.release.tag_name));
    }

    let temp_dir = app_state_dir.join("downloads");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let temp_path = temp_dir.join(format!("{}-{}", safe_name(&plan.release.tag_name), safe_name(&plan.asset.name)));

    set_phase(status, config.language, "Baixando release (retomável)...", "Downloading release (resumable)...");
    let resumed = download_resume::download(&client, token(config), &plan.asset, &temp_path)?;

    if let Some(checksum) = &plan.checksum {
        set_phase(status, config.language, "Verificando SHA-256...", "Verifying SHA-256...");
        verify::verify_sha256(&client, token(config), &plan.asset.name, &temp_path, checksum)?;
    }
    let sha256 = file_sha256(&temp_path)?;

    set_phase(status, config.language, "Instalando de forma segura...", "Installing safely...");
    let installed = install::install_release(&install_root, &config.app_name, &plan.release.tag_name, &plan.asset.name, &temp_path)?;

    if !config.restart_command.trim().is_empty() {
        restart::execute(&config.restart_command).map_err(|e| e.to_string())?;
    }

    if !config.health_check_command.trim().is_empty() {
        set_phase(status, config.language, "Executando health check...", "Running health check...");
        if let Err(health_error) = restart::execute(&config.health_check_command) {
            if !old.is_empty() && old != plan.release.tag_name {
                let rollback_result = install::activate_release(&install_root, &config.app_name, &old);
                let mut previous_state = state::load(&state_path).map_err(|e| e.to_string())?.unwrap_or_default();
                previous_state.latest_tag = old.clone();
                previous_state.installed_at_unix = state::now_unix();
                let _ = state::save_atomic(&state_path, &previous_state);
                let result = if rollback_result.is_ok() { "health-check-failed; automatic-rollback-ok" } else { "health-check-failed; automatic-rollback-failed" };
                let _ = history::append(&history_path, HistoryEntry::new("update", &old, &plan.release.tag_name, &plan.asset.name, &sha256, result));
                return Err(format!("{}: {health_error}. {}: {old}", tr(config.language, "Health check falhou", "Health check failed"), tr(config.language, "Rollback automático aplicado", "Automatic rollback applied")));
            }
            return Err(format!("{}: {health_error}", tr(config.language, "Health check falhou e não há versão anterior para rollback", "Health check failed and there is no previous version to roll back to")));
        }
    }

    let mut state_value = state::load(&state_path).map_err(|e| e.to_string())?.unwrap_or_default();
    state_value.latest_tag = plan.release.tag_name.clone();
    state_value.installed_at_unix = state::now_unix();
    state::save_atomic(&state_path, &state_value).map_err(|e| e.to_string())?;

    let (deleted, failed) = install::prune_old_releases(&install_root.join(&config.app_name).join("releases"), &plan.release.tag_name, config.retain).map_err(|e| e.to_string())?;
    let _ = history::append(&history_path, HistoryEntry::new("update", &old, &plan.release.tag_name, &plan.asset.name, &sha256, "success"));
    let _ = fs::remove_file(&temp_path);

    let mut message = format!(
        "{}: {}\n{}: {}\nSHA-256: {}",
        tr(config.language, "Atualização concluída", "Update completed"), plan.release.tag_name,
        tr(config.language, "Instalada em", "Installed at"), installed.display(), sha256
    );
    if resumed { message.push_str(&format!("\n{}", tr(config.language, "Download retomado de uma transferência parcial.", "Download resumed from a partial transfer."))); }
    if !deleted.is_empty() { message.push_str(&format!("\n{}: {}", tr(config.language, "Versões antigas removidas", "Old releases removed"), deleted.join(", "))); }
    if !failed.is_empty() { message.push_str(&format!("\n{}: {}", tr(config.language, "Aviso de limpeza", "Cleanup warning"), failed.iter().map(|(t,e)| format!("{t}: {e}")).collect::<Vec<_>>().join("; "))); }
    Ok(message)
}

fn token(config: &Config) -> Option<&str> {
    (!config.github_token.trim().is_empty()).then(|| config.github_token.trim())
}

fn safe_name(value: &str) -> String {
    value.chars().map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c }).collect()
}

fn set_phase(status: &Arc<Mutex<String>>, language: Language, pt: &'static str, en: &'static str) {
    if let Ok(mut text) = status.lock() { *text = tr(language, pt, en).into(); }
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
