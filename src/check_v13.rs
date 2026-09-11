use crate::{config::{Config, tr}, plan};

pub fn check(config: &Config) -> Result<String, String> {
    let (_, plan) = plan::build(config)?;
    Ok(match plan.current {
        Some(current) if current == plan.release.tag_name => format!("{}: {}", tr(config.language, "Atualizado", "Up to date"), current),
        Some(current) => format!("{}: {} → {}\n{}", tr(config.language, "Atualização disponível", "Update available"), current, plan.release.tag_name, plan.release.html_url),
        None => format!("{}: {}\n{}", tr(config.language, "Instalação disponível", "Install available"), plan.release.tag_name, plan.release.html_url),
    })
}
