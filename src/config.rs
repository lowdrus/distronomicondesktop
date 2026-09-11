#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    PtBr,
    En,
}

impl Language {
    pub fn label(self) -> &'static str {
        match self {
            Self::PtBr => "PT-BR",
            Self::En => "EN",
        }
    }
}

pub fn tr(language: Language, pt: &'static str, en: &'static str) -> &'static str {
    match language {
        Language::PtBr => pt,
        Language::En => en,
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    Check,
    Update,
    Version,
    Rollback,
    Doctor,
    Unlock,
}

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

pub fn normalize_repo(value: &str) -> Result<String, String> {
    let mut repo = value.trim().trim_end_matches('/').to_string();
    for prefix in ["https://github.com/", "http://github.com/", "github.com/"] {
        if let Some(rest) = repo.strip_prefix(prefix) {
            repo = rest.to_string();
            break;
        }
    }
    if let Some(stripped) = repo.strip_suffix(".git") {
        repo = stripped.to_string();
    }
    let parts = repo.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("invalid repository".into());
    }
    if parts.iter().any(|part| part.contains(['?', '#', '\\'])) {
        return Err("invalid repository".into());
    }
    Ok(format!("{}/{}", parts[0], parts[1]))
}

fn valid_windows_app_name(app: &str) -> bool {
    if app.is_empty()
        || app.contains("..")
        || app.ends_with([' ', '.'])
        || app.chars().any(|c| c < ' ' || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
    {
        return false;
    }

    let stem = app.split('.').next().unwrap_or("").to_ascii_uppercase();
    !matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL"
            | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9"
            | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    )
}

pub fn validate(config: &Config, action: Action) -> Result<(), String> {
    if !valid_windows_app_name(&config.app_name) {
        return Err(tr(
            config.language,
            "Nome da aplicação inválido para Windows. Evite caracteres < > : \" / \\ | ? *, '..', nomes reservados como CON/AUX/NUL/COM1/LPT1 e nomes terminados em ponto ou espaço.",
            "Invalid Windows application name. Avoid < > : \" / \\ | ? *, '..', reserved names such as CON/AUX/NUL/COM1/LPT1, and names ending in a dot or space.",
        )
        .into());
    }
    if matches!(action, Action::Check | Action::Update) {
        if normalize_repo(&config.repo).is_err() {
            return Err(tr(
                config.language,
                "Use owner/repository ou uma URL completa do GitHub, como https://github.com/owner/repository.",
                "Use owner/repository or a full GitHub URL such as https://github.com/owner/repository.",
            )
            .into());
        }
        if config.github_host.is_empty() {
            return Err(tr(
                config.language,
                "Informe o host da API do GitHub.",
                "Enter the GitHub API host.",
            )
            .into());
        }
    }
    if matches!(action, Action::Update)
        && !config.skip_verification
        && config.checksum_pattern.is_empty()
    {
        return Err(tr(
            config.language,
            "Informe o padrão do checksum ou habilite 'Pular verificação'.",
            "Enter a checksum pattern or enable 'Skip verification'.",
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_owner_repo() {
        assert_eq!(
            normalize_repo("lowdrus/distronomicondesktop").unwrap(),
            "lowdrus/distronomicondesktop"
        );
    }

    #[test]
    fn normalizes_full_github_url() {
        assert_eq!(
            normalize_repo("https://github.com/lowdrus/distronomicondesktop").unwrap(),
            "lowdrus/distronomicondesktop"
        );
    }

    #[test]
    fn normalizes_git_suffix_and_trailing_slash() {
        assert_eq!(
            normalize_repo("https://github.com/lowdrus/distronomicondesktop.git/").unwrap(),
            "lowdrus/distronomicondesktop"
        );
    }

    #[test]
    fn rejects_extra_path_segments() {
        assert!(
            normalize_repo("https://github.com/lowdrus/distronomicondesktop/releases").is_err()
        );
    }

    #[test]
    fn accepts_normal_windows_app_name() {
        assert!(valid_windows_app_name("distronomicondesktop"));
        assert!(valid_windows_app_name("meu-app"));
    }

    #[test]
    fn rejects_reserved_windows_app_names() {
        for name in ["CON", "con.txt", "AUX", "NUL", "COM1", "LPT9"] {
            assert!(!valid_windows_app_name(name), "{name}");
        }
    }

    #[test]
    fn rejects_invalid_windows_filename_characters_and_endings() {
        for name in ["app:one", "app?", "app*", "app.", "app ", "../app"] {
            assert!(!valid_windows_app_name(name), "{name}");
        }
    }
}
