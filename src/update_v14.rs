use crate::{
    config::{Config, Language, tr},
    download_resume,
    features_v14,
    history::{self, HistoryEntry},
    install, lock, plan, restart, state, verify,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};

pub fn run(
    config: &Config,
    status: &Arc<Mutex<String>>,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let install_root = PathBuf::from(&config.install_root);
    let state_root = PathBuf::from(&config.state_root);
    let app_state_dir = state_root.join(&config.app_name);
    let state_path = app_state_dir.join("state.json");
    let history_path = app_state_dir.join("history.json");
    let lock_path = app_state_dir.join("lock");
    let _ = fs::create_dir_all(&app_state_dir);

    if state::load(&state_path).is_err() {
        let _ = features_v14::recover_state(config);
    }
    let _guard = lock::acquire(&lock_path, Duration::from_secs(30)).map_err(|e| {
        format!("{}: {e}", tr(config.language, "Não foi possível adquirir o lock", "Could not acquire update lock"))
    })?;

    set_phase(status, config.language, "Testando conectividade e planejando...", "Testing connectivity and planning...");
    features_v14::log(config, "INFO", "Update planning started");
    let (client, plan) = plan::build(config)?;
    let old = plan.current.clone().unwrap_or_default();
    if plan.current.as_deref() == Some(plan.release.tag_name.as_str()) {
        return Ok(format!("{}: {}", tr(config.language, "Já está atualizado", "Already up to date"), plan.release.tag_name));
    }
    if !config.allow_downgrade && !old.is_empty() && features_v14::is_downgrade(&old, &plan.release.tag_name) {
        let error = format!(
            "{}: {} → {}",
            tr(config.language, "Downgrade bloqueado. Habilite a autorização explícita no perfil para continuar", "Downgrade blocked. Enable explicit authorization in the profile to continue"),
            old,
            plan.release.tag_name
        );
        features_v14::log(config, "WARNING", &error);
        return Err(error);
    }

    let required = features_v14::estimated_required_bytes(&plan.asset);
    let available = features_v14::ensure_free_space(&install_root, required)?;
    features_v14::log(
        config,
        "INFO",
        &format!("Space check: required={}, available={}", features_v14::format_size(required), features_v14::format_size(available)),
    );

    set_phase(status, config.language, "Criando backup dos arquivos configuráveis...", "Backing up configurable files...");
    let backup = features_v14::backup_configurable_files(config)?;

    let temp_dir = app_state_dir.join("downloads");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let temp_path = temp_dir.join(format!("{}-{}", safe_name(&plan.release.tag_name), safe_name(&plan.asset.name)));

    set_phase(status, config.language, "Baixando release (retomável e cancelável)...", "Downloading release (resumable and cancellable)...");
    let resumed = match download_resume::download_cancellable(&client, token(config), &plan.asset, &temp_path, cancel) {
        Ok(value) => value,
        Err(error) if error == "DOWNLOAD_CANCELLED" => {
            features_v14::log(config, "WARNING", "Download cancelled by user; active installation was not changed");
            return Err(tr(config.language, "Download cancelado. A instalação atual permaneceu intacta.", "Download cancelled. The current installation was left intact.").into());
        }
        Err(error) => return Err(error),
    };

    if let Some(checksum) = &plan.checksum {
        set_phase(status, config.language, "Verificando SHA-256...", "Verifying SHA-256...");
        verify::verify_sha256(&client, token(config), &plan.asset.name, &temp_path, checksum)?;
    }
    let sha256 = file_sha256(&temp_path)?;

    set_phase(status, config.language, "Instalando de forma segura...", "Installing safely...");
    let installed = install::install_release(
        &install_root,
        &config.app_name,
        &plan.release.tag_name,
        &plan.asset.name,
        &temp_path,
    )?;

    set_phase(status, config.language, "Validando assinatura Authenticode...", "Validating Authenticode signature...");
    if let Err(signature_error) = features_v14::verify_authenticode_tree(config, &installed) {
        rollback_after_failure(config, &install_root, &state_path, &old);
        let _ = fs::remove_dir_all(&installed);
        features_v14::log(config, "ERROR", &signature_error);
        return Err(signature_error);
    }

    if !config.restart_command.trim().is_empty() {
        restart::execute(&config.restart_command).map_err(|e| e.to_string())?;
    }

    if !config.health_check_command.trim().is_empty() {
        set_phase(status, config.language, "Executando verificação de saúde...", "Running health check...");
        if let Err(health_error) = restart::execute(&config.health_check_command) {
            rollback_after_failure(config, &install_root, &state_path, &old);
            let result = if !old.is_empty() { "health-check-failed; automatic-rollback-attempted" } else { "health-check-failed; no-previous-release" };
            let _ = history::append(
                &history_path,
                HistoryEntry::new("update", &old, &plan.release.tag_name, &plan.asset.name, &sha256, result),
            );
            return Err(format!(
                "{}: {health_error}. {}: {}",
                tr(config.language, "A verificação de saúde falhou", "Health check failed"),
                tr(config.language, "Versão anterior", "Previous version"),
                if old.is_empty() { "-" } else { &old }
            ));
        }
    }

    let mut state_value = state::load(&state_path).map_err(|e| e.to_string())?.unwrap_or_default();
    state_value.latest_tag = plan.release.tag_name.clone();
    if !plan.validators.etag.is_empty() { state_value.etag = plan.validators.etag.clone(); }
    if !plan.validators.last_modified.is_empty() { state_value.last_modified = plan.validators.last_modified.clone(); }
    state_value.installed_at_unix = state::now_unix();
    state::save_atomic(&state_path, &state_value).map_err(|e| e.to_string())?;

    let deleted = features_v14::prune_policy(config, &plan.release.tag_name)?;
    let _ = history::append(
        &history_path,
        HistoryEntry::new("update", &old, &plan.release.tag_name, &plan.asset.name, &sha256, "success"),
    );
    let _ = fs::remove_file(&temp_path);

    let mut message = format!(
        "{}: {}\n{}: {}\nSHA-256: {}\n{}: {}",
        tr(config.language, "Atualização concluída", "Update completed"),
        plan.release.tag_name,
        tr(config.language, "Instalada em", "Installed at"),
        installed.display(),
        sha256,
        tr(config.language, "Espaço estimado usado no planejamento", "Estimated space used in planning"),
        features_v14::format_size(required)
    );
    if let Some(path) = backup {
        message.push_str(&format!("\n{}: {}", tr(config.language, "Backup", "Backup"), path.display()));
    }
    if resumed {
        message.push_str(&format!("\n{}", tr(config.language, "Download retomado de uma transferência parcial.", "Download resumed from a partial transfer.")));
    }
    if !deleted.is_empty() {
        message.push_str(&format!("\n{}: {}", tr(config.language, "Versões removidas pela política de retenção", "Releases removed by retention policy"), deleted.join(", ")));
    }
    features_v14::log(config, "INFO", &message);
    features_v14::notify(config, "Distronomicon Desktop", &format!("{} {}", tr(config.language, "Atualização concluída", "Update completed"), plan.release.tag_name));
    Ok(message)
}

fn rollback_after_failure(config: &Config, install_root: &Path, state_path: &Path, old: &str) {
    if old.is_empty() { return; }
    if install::activate_release(install_root, &config.app_name, old).is_ok() {
        let mut previous_state = state::load(state_path).ok().flatten().unwrap_or_default();
        previous_state.latest_tag = old.to_string();
        previous_state.installed_at_unix = state::now_unix();
        let _ = state::save_atomic(state_path, &previous_state);
        if !config.restart_command.trim().is_empty() { let _ = restart::execute(&config.restart_command); }
    }
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
