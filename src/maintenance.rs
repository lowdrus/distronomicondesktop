use crate::{
    check,
    config::{Config, tr},
    history::{self, HistoryEntry},
    install,
};
use std::path::PathBuf;

pub fn rollback(config: &Config) -> Result<String, String> {
    let root = PathBuf::from(&config.install_root);
    let before = install::current_tag(&root, &config.app_name)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let result = check::rollback(config);
    let after = install::current_tag(&root, &config.app_name)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let history_path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("history.json");
    let status = if result.is_ok() { "success" } else { "failed" };
    let _ = history::append(
        &history_path,
        HistoryEntry::new("rollback", &before, &after, "", "", status),
    );
    result
}

pub fn history(config: &Config) -> Result<String, String> {
    let path = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("history.json");
    let body = history::format_recent(&path, 50).map_err(|e| e.to_string())?;
    let body = if body.trim().is_empty() {
        tr(
            config.language,
            "Nenhuma entrada no histórico.",
            "No history entries.",
        )
        .to_string()
    } else {
        body
    };
    Ok(format!(
        "{}\n{}",
        tr(config.language, "Histórico recente", "Recent history"),
        body
    ))
}
