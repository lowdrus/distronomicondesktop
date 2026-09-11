use crate::{
    check, check_v13,
    config::{self, Action, Config, Language, tr},
    maintenance, plan, profiles::{self, Profile}, scheduler, self_update, update_v13,
};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
};

pub struct DesktopApp {
    language: Language,
    dark_mode: bool,
    base: PathBuf,
    profiles: Vec<Profile>,
    selected_profile: String,
    profile_name: String,
    app_name: String,
    repo: String,
    asset_pattern: String,
    checksum_pattern: String,
    install_root: String,
    state_root: String,
    github_host: String,
    github_token: String,
    allow_prerelease: bool,
    skip_verification: bool,
    retain: usize,
    restart_command: String,
    health_check_command: String,
    pinned_version: String,
    auto_mode: String,
    interval_minutes: u32,
    status: Arc<Mutex<String>>,
    busy: Arc<AtomicBool>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        let base = std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)).or_else(|| std::env::current_dir().ok()).unwrap_or_else(|| PathBuf::from("."));
        let profiles = profiles::load(&profiles::default_path(&base)).unwrap_or_default();
        Self {
            language: Language::PtBr,
            dark_mode: true,
            base: base.clone(),
            profiles,
            selected_profile: String::new(),
            profile_name: String::new(),
            app_name: "meu-app".into(),
            repo: "owner/repository".into(),
            asset_pattern: r"(?i).*\.(zip|exe)$".into(),
            checksum_pattern: r"(?i)^(SHA256SUMS|checksums?(\.txt)?|.*sha256.*)$".into(),
            install_root: base.join("managed").display().to_string(),
            state_root: base.join(".distronomicon").display().to_string(),
            github_host: "https://api.github.com".into(),
            github_token: String::new(),
            allow_prerelease: false,
            skip_verification: false,
            retain: 3,
            restart_command: String::new(),
            health_check_command: String::new(),
            pinned_version: String::new(),
            auto_mode: "check".into(),
            interval_minutes: 60,
            status: Arc::new(Mutex::new("Pronto. Configure a aplicação e o repositório.".into())),
            busy: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl DesktopApp {
    fn config(&self) -> Config {
        let repo_input = self.repo.trim();
        let repo = config::normalize_repo(repo_input).unwrap_or_else(|_| repo_input.to_string());
        Config {
            language: self.language,
            app_name: self.app_name.trim().to_string(),
            repo,
            asset_pattern: self.asset_pattern.trim().to_string(),
            checksum_pattern: self.checksum_pattern.trim().to_string(),
            install_root: self.install_root.trim().to_string(),
            state_root: self.state_root.trim().to_string(),
            github_host: self.github_host.trim().to_string(),
            github_token: self.github_token.clone(),
            allow_prerelease: self.allow_prerelease,
            skip_verification: self.skip_verification,
            retain: self.retain,
            restart_command: self.restart_command.trim().to_string(),
            health_check_command: self.health_check_command.trim().to_string(),
            pinned_version: self.pinned_version.trim().to_string(),
        }
    }

    fn profile(&self) -> Profile {
        Profile {
            name: self.profile_name.trim().to_string(),
            app_name: self.app_name.trim().to_string(), repo: self.repo.trim().to_string(),
            asset_pattern: self.asset_pattern.clone(), checksum_pattern: self.checksum_pattern.clone(),
            install_root: self.install_root.clone(), state_root: self.state_root.clone(), github_host: self.github_host.clone(),
            allow_prerelease: self.allow_prerelease, skip_verification: self.skip_verification, retain: self.retain,
            restart_command: self.restart_command.clone(), health_check_command: self.health_check_command.clone(), pinned_version: self.pinned_version.clone(),
            auto_mode: self.auto_mode.clone(), interval_minutes: self.interval_minutes,
        }
    }

    fn apply_profile(&mut self, p: &Profile) {
        self.profile_name = p.name.clone(); self.app_name = p.app_name.clone(); self.repo = p.repo.clone();
        self.asset_pattern = p.asset_pattern.clone(); self.checksum_pattern = p.checksum_pattern.clone();
        self.install_root = p.install_root.clone(); self.state_root = p.state_root.clone(); self.github_host = p.github_host.clone();
        self.allow_prerelease = p.allow_prerelease; self.skip_verification = p.skip_verification; self.retain = p.retain;
        self.restart_command = p.restart_command.clone(); self.health_check_command = p.health_check_command.clone(); self.pinned_version = p.pinned_version.clone();
        self.auto_mode = if p.auto_mode.is_empty() { "check".into() } else { p.auto_mode.clone() };
        self.interval_minutes = p.interval_minutes.max(1);
    }

    fn set_status(&self, text: impl Into<String>) { if let Ok(mut status) = self.status.lock() { *status = text.into(); } }

    fn save_profile(&mut self) {
        if self.profile_name.trim().is_empty() { self.set_status(tr(self.language, "Informe um nome para o perfil.", "Enter a profile name.")); return; }
        let profile = self.profile();
        profiles::upsert(&mut self.profiles, profile.clone());
        match profiles::save(&profiles::default_path(&self.base), &self.profiles) {
            Ok(()) => { self.selected_profile = profile.name; self.set_status(tr(self.language, "Perfil salvo.", "Profile saved.")); }
            Err(e) => self.set_status(format!("{}: {e}", tr(self.language, "Erro ao salvar perfil", "Failed to save profile"))),
        }
    }

    fn delete_profile(&mut self) {
        let name = if self.selected_profile.is_empty() { self.profile_name.clone() } else { self.selected_profile.clone() };
        profiles::remove(&mut self.profiles, &name);
        match profiles::save(&profiles::default_path(&self.base), &self.profiles) {
            Ok(()) => { self.selected_profile.clear(); self.set_status(tr(self.language, "Perfil removido.", "Profile removed.")); }
            Err(e) => self.set_status(format!("{}: {e}", tr(self.language, "Erro ao remover perfil", "Failed to delete profile"))),
        }
    }

    fn schedule(&self) {
        if self.profile_name.trim().is_empty() { self.set_status(tr(self.language, "Salve o perfil antes de agendar.", "Save the profile before scheduling.")); return; }
        match std::env::current_exe().map_err(|e| e.to_string()).and_then(|exe| scheduler::install(&self.profile_name, &exe, &self.auto_mode, self.interval_minutes).map_err(|e| e.to_string())) {
            Ok(()) => self.set_status(tr(self.language, "Automação do Windows ativada.", "Windows automation enabled.")),
            Err(e) => self.set_status(format!("{}: {e}", tr(self.language, "Falha ao criar agendamento", "Failed to create schedule"))),
        }
    }

    fn unschedule(&self) {
        match scheduler::uninstall(&self.profile_name) {
            Ok(()) => self.set_status(tr(self.language, "Automação removida.", "Automation removed.")),
            Err(e) => self.set_status(format!("{}: {e}", tr(self.language, "Falha ao remover agendamento", "Failed to remove schedule"))),
        }
    }

    fn run(&self, action: Action) {
        if self.busy.swap(true, Ordering::SeqCst) { return; }
        let config = self.config();
        if !matches!(action, Action::SelfUpdate) {
            if let Err(message) = config::validate(&config, action) { self.set_status(message); self.busy.store(false, Ordering::SeqCst); return; }
        }
        let status = Arc::clone(&self.status); let busy = Arc::clone(&self.busy);
        thread::spawn(move || {
            let language = config.language;
            let result = match action {
                Action::Check => check_v13::check(&config),
                Action::Update => update_v13::run(&config, &status),
                Action::DryRun => plan::dry_run(&config),
                Action::Version => check::version(&config),
                Action::Rollback => maintenance::rollback(&config),
                Action::Doctor => check::doctor(&config),
                Action::History => maintenance::history(&config),
                Action::Unlock => check::unlock(&config),
                Action::SelfUpdate => self_update::run(),
            };
            if let Ok(mut text) = status.lock() { *text = match result { Ok(message) => message, Err(error) => format!("{}: {error}", tr(language, "Erro", "Error")) }; }
            busy.store(false, Ordering::SeqCst);
        });
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);
        let language = self.language; let is_busy = self.busy.load(Ordering::SeqCst);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Distronomicon Desktop");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.dark_mode, "◐").on_hover_text(tr(language, "Alternar modo claro/escuro", "Toggle light/dark mode"));
                    egui::ComboBox::from_id_salt("language").selected_text(self.language.label()).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.language, Language::PtBr, "PT-BR"); ui.selectable_value(&mut self.language, Language::En, "EN");
                    });
                });
            });
            ui.label(tr(language, "Gerenciador de releases portátil e nativo para Windows.", "Portable native release manager for Windows."));
            ui.small(tr(language, "Sem WSL, terminal ou instalação.", "No WSL, terminal, or installation required."));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label(tr(language, "Perfil", "Profile"));
                let mut chosen: Option<Profile> = None;
                egui::ComboBox::from_id_salt("profile").selected_text(if self.selected_profile.is_empty() { tr(language, "Selecione...", "Select...") } else { &self.selected_profile }).show_ui(ui, |ui| {
                    for p in &self.profiles { if ui.selectable_label(self.selected_profile == p.name, &p.name).clicked() { chosen = Some(p.clone()); } }
                });
                if let Some(p) = chosen { self.selected_profile = p.name.clone(); self.apply_profile(&p); }
            });
            ui.horizontal(|ui| { ui.label(tr(language, "Nome do perfil", "Profile name")); ui.text_edit_singleline(&mut self.profile_name); if ui.button(tr(language, "Salvar perfil", "Save profile")).clicked() { self.save_profile(); } if ui.button(tr(language, "Excluir", "Delete")).clicked() { self.delete_profile(); } });
            ui.add_space(8.0);

            ui.label(tr(language, "Aplicação", "Application")); ui.text_edit_singleline(&mut self.app_name);
            ui.small(tr(language, "Nome local que identifica o programa gerenciado.", "Local name used to identify the managed program."));
            ui.label(tr(language, "Repositório GitHub", "GitHub repository")); ui.text_edit_singleline(&mut self.repo);
            ui.small(tr(language, "Aceita owner/repository ou qualquer URL completa válida do GitHub.", "Accepts owner/repository or any valid full GitHub repository URL."));
            ui.add_space(10.0);

            ui.add_enabled_ui(!is_busy, |ui| { ui.horizontal_wrapped(|ui| {
                if ui.button(tr(language, "Verificar", "Check")).clicked() { self.run(Action::Check); }
                if ui.button(tr(language, "Prévia", "Dry Run")).clicked() { self.run(Action::DryRun); }
                if ui.button(tr(language, "Atualizar", "Update")).clicked() { self.run(Action::Update); }
                if ui.button(tr(language, "Versão", "Version")).clicked() { self.run(Action::Version); }
                if ui.button(tr(language, "Reverter", "Rollback")).clicked() { self.run(Action::Rollback); }
                if ui.button(tr(language, "Diagnóstico", "Doctor")).clicked() { self.run(Action::Doctor); }
                if ui.button(tr(language, "Histórico", "History")).clicked() { self.run(Action::History); }
            }); });
            if is_busy { ui.horizontal(|ui| { ui.spinner(); ui.label(tr(language, "Processando...", "Working...")); }); }

            egui::CollapsingHeader::new(tr(language, "Opções avançadas", "Advanced options")).default_open(false).show(ui, |ui| {
                ui.label(tr(language, "Fixar versão (vazio = mais recente)", "Pin version (blank = latest)")); ui.text_edit_singleline(&mut self.pinned_version);
                ui.label(tr(language, "Health check após atualizar", "Post-update health check")); ui.text_edit_singleline(&mut self.health_check_command);
                ui.small(tr(language, "Se falhar, a versão anterior é reativada automaticamente.", "If it fails, the previous version is automatically restored."));
                ui.label(tr(language, "Pasta de instalação", "Install folder")); ui.text_edit_singleline(&mut self.install_root);
                ui.label(tr(language, "Pasta de estado", "State folder")); ui.text_edit_singleline(&mut self.state_root);
                ui.label(tr(language, "Padrão do asset (Regex)", "Asset pattern (Regex)")); ui.text_edit_singleline(&mut self.asset_pattern);
                ui.label(tr(language, "Padrão do checksum (Regex)", "Checksum pattern (Regex)")); ui.text_edit_singleline(&mut self.checksum_pattern);
                ui.checkbox(&mut self.skip_verification, tr(language, "Pular verificação SHA-256 (não recomendado)", "Skip SHA-256 verification (not recommended)"));
                ui.checkbox(&mut self.allow_prerelease, tr(language, "Permitir pré-lançamentos", "Allow prereleases"));
                ui.horizontal(|ui| { ui.label(tr(language, "Manter versões recentes", "Keep recent releases")); ui.add(egui::DragValue::new(&mut self.retain).range(0..=20)); });
                ui.label(tr(language, "Host da API do GitHub", "GitHub API host")); ui.text_edit_singleline(&mut self.github_host);
                ui.label(tr(language, "Token do GitHub (opcional)", "GitHub token (optional)")); ui.add(egui::TextEdit::singleline(&mut self.github_token).password(true));
                ui.label(tr(language, "Comando após atualização/reversão", "Post-update/rollback command")); ui.text_edit_singleline(&mut self.restart_command);
                ui.separator();
                ui.label(tr(language, "Automação do Windows", "Windows automation"));
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("auto_mode").selected_text(if self.auto_mode == "update" { tr(language, "Verificar e atualizar", "Check and update") } else { tr(language, "Somente verificar", "Check only") }).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.auto_mode, "check".into(), tr(language, "Somente verificar", "Check only"));
                        ui.selectable_value(&mut self.auto_mode, "update".into(), tr(language, "Verificar e atualizar", "Check and update"));
                    });
                    ui.label(tr(language, "a cada", "every")); ui.add(egui::DragValue::new(&mut self.interval_minutes).range(1..=1440)); ui.label(tr(language, "minutos", "minutes"));
                });
                ui.horizontal(|ui| { if ui.button(tr(language, "Ativar automação", "Enable automation")).clicked() { self.schedule(); } if ui.button(tr(language, "Remover automação", "Remove automation")).clicked() { self.unschedule(); } });
                ui.separator();
                ui.add_enabled_ui(!is_busy, |ui| { if ui.button(tr(language, "Atualizar o próprio Distronomicon Desktop", "Update Distronomicon Desktop itself")).clicked() { self.run(Action::SelfUpdate); } });
                ui.add_enabled_ui(!is_busy, |ui| { if ui.button(tr(language, "Forçar desbloqueio", "Force unlock")).clicked() { self.run(Action::Unlock); } });
                ui.small(tr(language, "Use o desbloqueio apenas se uma atualização interrompida deixou o lock preso.", "Use force unlock only if an interrupted update left a stale lock."));
            });

            ui.separator(); ui.label(tr(language, "Situação", "Status"));
            let mut text = self.status.lock().map(|s| s.clone()).unwrap_or_default();
            ui.add(egui::TextEdit::multiline(&mut text).desired_rows(10).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace).interactive(false));
        });
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut visuals = if dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() };
    let golden_gate = egui::Color32::from_rgb(205, 151, 77);
    visuals.selection.bg_fill = golden_gate; visuals.hyperlink_color = golden_gate; ctx.set_visuals(visuals);
}
