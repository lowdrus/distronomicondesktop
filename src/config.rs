#[derive(Clone, Copy, PartialEq)]
pub enum Language { PtBr, En }

impl Language {
    pub fn label(self) -> &'static str { match self { Self::PtBr => "PT-BR", Self::En => "EN" } }
}

pub fn tr(language: Language, pt: &'static str, en: &'static str) -> &'static str {
    match language { Language::PtBr => pt, Language::En => en }
}

#[derive(Clone, Copy)]
pub enum Action { Check, Update, Version, Unlock }

#[derive(Clone)]
pub struct Config {
    pub language: Language,
    pub app_name: String,
    pub repo: String,
    pub asset_pattern: String,
    pub checksum_pattern: String,
    pub install_root: String,
    pub state_root: String,
    pub github_host: String,
    pub github_token: String,
    pub allow_prerelease: bool,
    pub skip_verification: bool,
    pub retain: usize,
    pub restart_command: String,
}

pub fn validate(config: &Config, action: Action) -> Result<(), String> {
    let app = &config.app_name;
    if app.is_empty() || app.contains('/') || app.contains('\\') || app.contains("..") || app.contains('\0') {
        return Err(tr(config.language,
            "Nome da aplicação inválido. Não use /, \\, .. ou caracteres nulos.",
            "Invalid application name. Do not use /, \\, .., or null characters.").into());
    }
    if matches!(action, Action::Check | Action::Update) {
        let parts = config.repo.split('/').collect::<Vec<_>>();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(tr(config.language,
                "Use o repositório no formato owner/repository.",
                "Use the repository format owner/repository.").into());
        }
        if config.github_host.is_empty() {
            return Err(tr(config.language, "Informe o host da API do GitHub.", "Enter the GitHub API host.").into());
        }
    }
    if matches!(action, Action::Update) && !config.skip_verification && config.checksum_pattern.is_empty() {
        return Err(tr(config.language,
            "Informe o padrão do checksum ou habilite 'Pular verificação'.",
            "Enter a checksum pattern or enable 'Skip verification'.").into());
    }
    Ok(())
}
