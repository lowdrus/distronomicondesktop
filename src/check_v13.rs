use crate::{
    config::{Config, tr},
    features_v14, plan, state,
};
use std::path::PathBuf;

pub fn check(config: &Config) -> Result<String, String> {
    let state_path = PathBuf::from(&config.state_root).join(&config.app_name).join("state.json");
    if state::load(&state_path).is_err() {
        let _ = features_v14::recover_state(config);
    }
    let (_, plan) = plan::build(config)?;
    if let Some(current) = &plan.current {
        let mut value = state::load(&state_path).map_err(|e| e.to_string())?.unwrap_or_default();
        value.latest_tag = current.clone();
        if !plan.validators.etag.is_empty() { value.etag = plan.validators.etag.clone(); }
        if !plan.validators.last_modified.is_empty() { value.last_modified = plan.validators.last_modified.clone(); }
        if value.installed_at_unix == 0 { value.installed_at_unix = state::now_unix(); }
        state::save_atomic(&state_path, &value).map_err(|e| e.to_string())?;
    }

    let message = match plan.current {
        Some(current) if current == plan.release.tag_name => format!("{}: {}", tr(config.language, "Atualizado", "Up to date"), current),
        Some(current) => {
            let text = format!(
                "{}: {} → {}\n{}",
                tr(config.language, "Atualização disponível", "Update available"),
                current,
                plan.release.tag_name,
                plan.release.html_url
            );
            features_v14::notify(config, "Distronomicon Desktop", &text.replace('\n', " "));
            text
        }
        None => {
            let text = format!(
                "{}: {}\n{}",
                tr(config.language, "Instalação disponível", "Install available"),
                plan.release.tag_name,
                plan.release.html_url
            );
            features_v14::notify(config, "Distronomicon Desktop", &text.replace('\n', " "));
            text
        }
    };
    features_v14::log(config, "INFO", &message);
    Ok(message)
}
