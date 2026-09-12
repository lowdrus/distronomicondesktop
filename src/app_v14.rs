use crate::{
    check, check_v13,
    config::{self, Action, Config, Language, tr},
    features_v14, maintenance, plan,
    profiles::{self, Profile},
    scheduler, self_update, update_v14,
};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
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
    skip_verification: bool,
    retain: usize,
    restart_command: String,
    health_check_command: String,
    pinned_version: String,
    auto_mode: String,
    interval_minutes: u32,
    channel: String,
    architecture: String,
    backup_paths: String,
    verify_authenticode: bool,
    max_disk_mb: u64,
    retention_days: u64,
    allow_downgrade: bool,
    notifications: bool,
    settings_open: bool,
    log_open: bool,
    status: Arc<Mutex<String>>,
    busy: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        let base = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
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
            asset_pattern: r"(?i).*\.(zip|exe|tgz|tbz2|txz)$|.*\.tar\.(gz|bz2|xz|zst)$".into(),
            checksum_pattern: r"(?i)^(SHA256SUMS|checksums?(\.txt)?|.*sha256.*)$".into(),
            install_root: base.join("managed").display().to_string(),
            state_root: base.join(".distronomicon").display().to_string(),
            github_host: "https://api.github.com".into(),
            github_token: String::new(),
            skip_verification: false,
            retain: 3,
            restart_command: String::new(),
            health_check_command: String::new(),
            pinned_version: String::new(),
            auto_mode: "check".into(),
            interval_minutes: 60,
            channel: "stable".into(),
            architecture: "auto".into(),
            backup_paths: String::new(),
            verify_authenticode: true,
            max_disk_mb: 0,
            retention_days: 0,
            allow_downgrade: false,
            notifications: true,
            settings_open: false,
            log_open: false,
            status: Arc::new(Mutex::new(
                "Pronto. Configure a aplicação e o repositório.".into(),
            )),
            busy: Arc::new(AtomicBool::new(false)),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl DesktopApp {
    fn config(&self) -> Config {
        let repo_input = self.repo.trim();
        let repo = config::normalize_repo(repo_input).unwrap_or_else(|_| repo_input.to_string());
        Config {
            language: self.language,
            app_name: config::normalize_app_name(&self.app_name, &repo),
            repo,
            asset_pattern: self.asset_pattern.trim().into(),
            checksum_pattern: self.checksum_pattern.trim().into(),
            install_root: self.install_root.trim().into(),
            state_root: self.state_root.trim().into(),
            github_host: self.github_host.trim().into(),
            github_token: self.github_token.clone(),
            allow_prerelease: self.channel != "stable",
            skip_verification: self.skip_verification,
            retain: self.retain,
            restart_command: self.restart_command.trim().into(),
            health_check_command: self.health_check_command.trim().into(),
            pinned_version: self.pinned_version.trim().into(),
            channel: self.channel.clone(),
            architecture: self.architecture.clone(),
            backup_paths: self.backup_paths.clone(),
            verify_authenticode: self.verify_authenticode,
            max_disk_mb: self.max_disk_mb,
            retention_days: self.retention_days,
            allow_downgrade: self.allow_downgrade,
            notifications: self.notifications,
        }
    }

    fn profile(&self) -> Profile {
        Profile {
            name: self.profile_name.trim().into(),
            app_name: config::normalize_app_name(&self.app_name, &self.repo),
            repo: self.repo.trim().into(),
            asset_pattern: self.asset_pattern.clone(),
            checksum_pattern: self.checksum_pattern.clone(),
            install_root: self.install_root.clone(),
            state_root: self.state_root.clone(),
            github_host: self.github_host.clone(),
            allow_prerelease: self.channel != "stable",
            skip_verification: self.skip_verification,
            retain: self.retain,
            restart_command: self.restart_command.clone(),
            health_check_command: self.health_check_command.clone(),
            pinned_version: self.pinned_version.clone(),
            auto_mode: self.auto_mode.clone(),
            interval_minutes: self.interval_minutes,
            channel: self.channel.clone(),
            architecture: self.architecture.clone(),
            backup_paths: self.backup_paths.clone(),
            verify_authenticode: self.verify_authenticode,
            max_disk_mb: self.max_disk_mb,
            retention_days: self.retention_days,
            allow_downgrade: self.allow_downgrade,
            notifications: self.notifications,
        }
    }

    fn apply_profile(&mut self, p: &Profile) {
        self.profile_name = p.name.clone();
        self.app_name = p.app_name.clone();
        self.repo = p.repo.clone();
        self.asset_pattern = p.asset_pattern.clone();
        self.checksum_pattern = p.checksum_pattern.clone();
        self.install_root = if p.install_root.is_empty() {
            self.base.join("managed").display().to_string()
        } else {
            p.install_root.clone()
        };
        self.state_root = if p.state_root.is_empty() {
            self.base.join(".distronomicon").display().to_string()
        } else {
            p.state_root.clone()
        };
        self.github_host = if p.github_host.is_empty() {
            "https://api.github.com".into()
        } else {
            p.github_host.clone()
        };
        self.skip_verification = p.skip_verification;
        self.retain = p.retain;
        self.restart_command = p.restart_command.clone();
        self.health_check_command = p.health_check_command.clone();
        self.pinned_version = p.pinned_version.clone();
        self.auto_mode = if p.auto_mode.is_empty() {
            "check".into()
        } else {
            p.auto_mode.clone()
        };
        self.interval_minutes = p.interval_minutes.max(1);
        self.channel = p.effective_channel();
        self.architecture = if p.architecture.is_empty() {
            "auto".into()
        } else {
            p.architecture.clone()
        };
        self.backup_paths = p.backup_paths.clone();
        self.verify_authenticode = p.verify_authenticode;
        self.max_disk_mb = p.max_disk_mb;
        self.retention_days = p.retention_days;
        self.allow_downgrade = p.allow_downgrade;
        self.notifications = p.notifications;
    }

    fn set_status(&self, text: impl Into<String>) {
        if let Ok(mut s) = self.status.lock() {
            *s = text.into();
        }
    }

    fn save_profile(&mut self) {
        if self.profile_name.trim().is_empty() {
            self.set_status(tr(
                self.language,
                "Informe um nome para o perfil.",
                "Enter a profile name.",
            ));
            return;
        }
        let p = self.profile();
        profiles::upsert(&mut self.profiles, p.clone());
        match profiles::save(&profiles::default_path(&self.base), &self.profiles) {
            Ok(()) => {
                self.selected_profile = p.name;
                self.app_name = p.app_name;
                self.set_status(tr(self.language, "Perfil salvo.", "Profile saved."));
            }
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Erro ao salvar perfil",
                    "Failed to save profile"
                )
            )),
        }
    }
    fn delete_profile(&mut self) {
        let n = if self.selected_profile.is_empty() {
            self.profile_name.clone()
        } else {
            self.selected_profile.clone()
        };
        profiles::remove(&mut self.profiles, &n);
        match profiles::save(&profiles::default_path(&self.base), &self.profiles) {
            Ok(()) => {
                self.selected_profile.clear();
                self.set_status(tr(self.language, "Perfil removido.", "Profile removed."));
            }
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Erro ao remover perfil",
                    "Failed to delete profile"
                )
            )),
        }
    }
    fn export_profiles(&self) {
        match profiles::export_profiles(&self.base, &self.profiles) {
            Ok(p) => self.set_status(format!(
                "{}: {}",
                tr(self.language, "Perfis exportados", "Profiles exported"),
                p.display()
            )),
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Falha ao exportar perfis",
                    "Failed to export profiles"
                )
            )),
        }
    }
    fn import_profiles(&mut self) {
        match profiles::import_profiles(&self.base) {
            Ok(v) => {
                for p in v {
                    profiles::upsert(&mut self.profiles, p);
                }
                match profiles::save(&profiles::default_path(&self.base), &self.profiles) {
                    Ok(()) => self.set_status(tr(
                        self.language,
                        "Perfis importados com sucesso.",
                        "Profiles imported successfully.",
                    )),
                    Err(e) => self.set_status(format!(
                        "{}: {e}",
                        tr(
                            self.language,
                            "Falha ao salvar perfis importados",
                            "Failed to save imported profiles"
                        )
                    )),
                }
            }
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Falha ao importar perfis",
                    "Failed to import profiles"
                )
            )),
        }
    }
    fn schedule(&self) {
        if self.profile_name.trim().is_empty() {
            self.set_status(tr(
                self.language,
                "Salve o perfil antes de agendar.",
                "Save the profile before scheduling.",
            ));
            return;
        }
        match std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|exe| {
                scheduler::install(
                    &self.profile_name,
                    &exe,
                    &self.auto_mode,
                    self.interval_minutes,
                )
                .map_err(|e| e.to_string())
            }) {
            Ok(()) => self.set_status(tr(
                self.language,
                "Automação do Windows ativada.",
                "Windows automation enabled.",
            )),
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Falha ao criar agendamento",
                    "Failed to create schedule"
                )
            )),
        }
    }
    fn unschedule(&self) {
        match scheduler::uninstall(&self.profile_name) {
            Ok(()) => self.set_status(tr(
                self.language,
                "Automação removida.",
                "Automation removed.",
            )),
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Falha ao remover agendamento",
                    "Failed to remove schedule"
                )
            )),
        }
    }

    fn open_current_folder(&self) {
        match features_v14::current_release_path(&self.config())
            .and_then(|p| features_v14::open_path(&p))
        {
            Ok(()) => {}
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Não foi possível abrir a pasta",
                    "Could not open folder"
                )
            )),
        }
    }
    fn open_release_page(&self) {
        let c = self.config();
        let url = features_v14::current_release_url(&c).or_else(|_| {
            let repo = config::normalize_repo(&c.repo).map_err(|e| e.to_string())?;
            Ok::<String, String>(format!("https://github.com/{repo}/releases"))
        });
        match url.and_then(|url| features_v14::open_url(&url)) {
            Ok(()) => {}
            Err(e) => self.set_status(format!(
                "{}: {e}",
                tr(
                    self.language,
                    "Não foi possível abrir a página",
                    "Could not open page"
                )
            )),
        }
    }

    fn run(&self, action: Action) {
        if self.busy.swap(true, Ordering::SeqCst) {
            return;
        }
        self.cancel.store(false, Ordering::SeqCst);
        let c = self.config();
        if !matches!(action, Action::SelfUpdate | Action::Recovery) {
            if let Err(m) = config::validate(&c, action) {
                self.set_status(m);
                self.busy.store(false, Ordering::SeqCst);
                return;
            }
        }
        let status = Arc::clone(&self.status);
        let busy = Arc::clone(&self.busy);
        let cancel = Arc::clone(&self.cancel);
        thread::spawn(move || {
            let language = c.language;
            let r = match action {
                Action::Check => check_v13::check(&c),
                Action::Update => update_v14::run(&c, &status, &cancel),
                Action::DryRun => plan::dry_run(&c),
                Action::Version => check::version(&c),
                Action::Rollback => maintenance::rollback(&c),
                Action::Doctor => check::doctor(&c),
                Action::History => maintenance::history(&c),
                Action::Unlock => check::unlock(&c),
                Action::SelfUpdate => self_update::run(),
                Action::ReleaseNotes => plan::release_notes(&c),
                Action::Recovery => features_v14::recover_state(&c),
            };
            if let Ok(mut t) = status.lock() {
                *t = match r {
                    Ok(m) => m,
                    Err(e) => format!("{}: {e}", tr(language, "Erro", "Error")),
                };
            }
            busy.store(false, Ordering::SeqCst);
        });
    }

    fn settings_window(&mut self, ctx: &egui::Context, is_busy: bool) {
        let language = self.language;
        let mut open = self.settings_open;
        egui::Window::new(tr(language, "Configurações", "Settings"))
            .open(&mut open)
            .default_width(590.0)
            .default_height(650.0)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.heading(tr(language, "Interface", "Interface"));
                ui.checkbox(
                    &mut self.dark_mode,
                    tr(language, "Tema escuro", "Dark theme"),
                );
                ui.checkbox(
                    &mut self.notifications,
                    tr(
                        language,
                        "Notificações nativas do Windows",
                        "Native Windows notifications",
                    ),
                );
                ui.separator();
                ui.heading(tr(language, "Canal e plataforma", "Channel and platform"));
                ui.horizontal(|ui| {
                    ui.label(tr(language, "Canal", "Channel"));
                    egui::ComboBox::from_id_salt("channel")
                        .selected_text(&self.channel)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.channel, "stable".into(), "Stable");
                            ui.selectable_value(&mut self.channel, "beta".into(), "Beta");
                            ui.selectable_value(&mut self.channel, "nightly".into(), "Nightly");
                        });
                    ui.label(tr(language, "Arquitetura", "Architecture"));
                    egui::ComboBox::from_id_salt("arch")
                        .selected_text(&self.architecture)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.architecture, "auto".into(), "Auto");
                            ui.selectable_value(&mut self.architecture, "x64".into(), "x64");
                            ui.selectable_value(&mut self.architecture, "arm64".into(), "ARM64");
                            ui.selectable_value(&mut self.architecture, "x86".into(), "x86");
                        });
                });
                ui.label(tr(
                    language,
                    "Fixar versão (vazio = canal selecionado)",
                    "Pin version (blank = selected channel)",
                ));
                ui.text_edit_singleline(&mut self.pinned_version);
                ui.checkbox(
                    &mut self.allow_downgrade,
                    tr(
                        language,
                        "Permitir downgrade explicitamente",
                        "Explicitly allow downgrade",
                    ),
                );
                ui.checkbox(
                    &mut self.verify_authenticode,
                    tr(
                        language,
                        "Validar Authenticode quando houver assinatura",
                        "Validate Authenticode when a signature is present",
                    ),
                );
                ui.separator();
                ui.heading(tr(language, "Backup e retenção", "Backup and retention"));
                ui.label(tr(
                    language,
                    "Arquivos/pastas configuráveis para backup antes do update (; ou nova linha)",
                    "Configurable files/folders to back up before update (; or new lines)",
                ));
                ui.add(egui::TextEdit::multiline(&mut self.backup_paths).desired_rows(3));
                ui.horizontal(|ui| {
                    ui.label(tr(language, "Manter versões", "Keep releases"));
                    ui.add(egui::DragValue::new(&mut self.retain).range(0..=50));
                    ui.label(tr(language, "Dias", "Days"));
                    ui.add(egui::DragValue::new(&mut self.retention_days).range(0..=3650));
                });
                ui.horizontal(|ui| {
                    ui.label(tr(
                        language,
                        "Limite das releases em MB (0 desativa)",
                        "Release disk limit MB (0 disables)",
                    ));
                    ui.add(egui::DragValue::new(&mut self.max_disk_mb).range(0..=1_000_000));
                });
                ui.separator();
                ui.heading(tr(language, "Automação do Windows", "Windows automation"));
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("auto_mode")
                        .selected_text(if self.auto_mode == "update" {
                            tr(language, "Verificar e atualizar", "Check and update")
                        } else {
                            tr(language, "Somente verificar", "Check only")
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.auto_mode,
                                "check".into(),
                                tr(language, "Somente verificar", "Check only"),
                            );
                            ui.selectable_value(
                                &mut self.auto_mode,
                                "update".into(),
                                tr(language, "Verificar e atualizar", "Check and update"),
                            );
                        });
                    ui.label(tr(language, "a cada", "every"));
                    ui.add(egui::DragValue::new(&mut self.interval_minutes).range(1..=1440));
                    ui.label(tr(language, "minutos", "minutes"));
                });
                ui.horizontal(|ui| {
                    if ui
                        .button(tr(language, "Ativar automação", "Enable automation"))
                        .clicked()
                    {
                        self.schedule();
                    }
                    if ui
                        .button(tr(language, "Remover automação", "Remove automation"))
                        .clicked()
                    {
                        self.unschedule();
                    }
                });
                ui.separator();
                ui.heading(tr(language, "Avançado", "Advanced"));
                ui.label(tr(language, "Pasta de instalação", "Install folder"));
                ui.text_edit_singleline(&mut self.install_root);
                ui.label(tr(language, "Pasta de estado", "State folder"));
                ui.text_edit_singleline(&mut self.state_root);
                ui.label(tr(
                    language,
                    "Padrão do asset (Regex)",
                    "Asset pattern (Regex)",
                ));
                ui.text_edit_singleline(&mut self.asset_pattern);
                ui.label(tr(
                    language,
                    "Padrão do checksum (Regex)",
                    "Checksum pattern (Regex)",
                ));
                ui.text_edit_singleline(&mut self.checksum_pattern);
                ui.checkbox(
                    &mut self.skip_verification,
                    tr(
                        language,
                        "Pular SHA-256 (não recomendado)",
                        "Skip SHA-256 (not recommended)",
                    ),
                );
                ui.label(tr(language, "Host da API do GitHub", "GitHub API host"));
                ui.text_edit_singleline(&mut self.github_host);
                ui.label(tr(
                    language,
                    "Token do GitHub (opcional)",
                    "GitHub token (optional)",
                ));
                ui.add(egui::TextEdit::singleline(&mut self.github_token).password(true));
                ui.label(tr(
                    language,
                    "Comando após update/rollback",
                    "Post-update/rollback command",
                ));
                ui.text_edit_singleline(&mut self.restart_command);
                ui.label(tr(
                    language,
                    "Health check após update",
                    "Post-update health check",
                ));
                ui.text_edit_singleline(&mut self.health_check_command);
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(tr(language, "Exportar perfis", "Export profiles"))
                        .clicked()
                    {
                        self.export_profiles();
                    }
                    if ui
                        .button(tr(language, "Importar perfis", "Import profiles"))
                        .clicked()
                    {
                        self.import_profiles();
                    }
                    if ui
                        .button(tr(language, "Modo de recuperação", "Recovery mode"))
                        .clicked()
                    {
                        self.run(Action::Recovery);
                    }
                    if ui
                        .button(tr(language, "Forçar desbloqueio", "Force unlock"))
                        .clicked()
                    {
                        self.run(Action::Unlock);
                    }
                    if ui
                        .add_enabled(
                            !is_busy,
                            egui::Button::new(tr(
                                language,
                                "Atualizar o Distronomicon Desktop",
                                "Update Distronomicon Desktop",
                            )),
                        )
                        .clicked()
                    {
                        self.run(Action::SelfUpdate);
                    }
                });
            });
        self.settings_open = open;
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);
        let language = self.language;
        let is_busy = self.busy.load(Ordering::SeqCst);
        if config::normalize_repo(&self.app_name).is_ok() {
            self.app_name = config::normalize_app_name(&self.app_name, &self.repo);
        }
        egui::CentralPanel::default().show(ctx,|ui|{ui.horizontal(|ui|{ui.heading("Distronomicon Desktop");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{egui::ComboBox::from_id_salt("language").selected_text(self.language.label()).show_ui(ui,|ui|{ui.selectable_value(&mut self.language,Language::PtBr,"PT-BR");ui.selectable_value(&mut self.language,Language::En,"EN");});if ui.button(tr(language,"⚙ Configurações","⚙ Settings")).clicked(){self.settings_open=true;}});});ui.label(tr(language,"Gerenciador de releases portátil e nativo para Windows.","Portable native release manager for Windows."));ui.small(tr(language,"Sem WSL, terminal ou instalação.","No WSL, terminal, or installation required."));ui.add_space(10.0);
            ui.horizontal(|ui|{ui.label(tr(language,"Perfil","Profile"));let mut chosen=None;egui::ComboBox::from_id_salt("profile").selected_text(if self.selected_profile.is_empty(){tr(language,"Selecione...","Select...")}else{&self.selected_profile}).show_ui(ui,|ui|{for p in &self.profiles{if ui.selectable_label(self.selected_profile==p.name,&p.name).clicked(){chosen=Some(p.clone());}}});if let Some(p)=chosen{self.selected_profile=p.name.clone();self.apply_profile(&p);}});ui.horizontal(|ui|{ui.label(tr(language,"Nome do perfil","Profile name"));ui.text_edit_singleline(&mut self.profile_name);if ui.button(tr(language,"Salvar perfil","Save profile")).clicked(){self.save_profile();}if ui.button(tr(language,"Excluir","Delete")).clicked(){self.delete_profile();}});ui.add_space(8.0);
            ui.label(tr(language,"Aplicação","Application"));ui.text_edit_singleline(&mut self.app_name);ui.small(tr(language,"Nome local simples. owner/repository colado aqui por engano é corrigido automaticamente.","Simple local name. owner/repository pasted here by mistake is corrected automatically."));ui.label(tr(language,"Repositório GitHub","GitHub repository"));ui.text_edit_singleline(&mut self.repo);ui.small(tr(language,"Aceita owner/repository ou URL completa do GitHub.","Accepts owner/repository or a full GitHub URL."));ui.horizontal(|ui|{ui.small(format!("{}: {}",tr(language,"Canal","Channel"),self.channel));ui.separator();ui.small(format!("{}: {}",tr(language,"Arquitetura","Architecture"),self.architecture));});ui.add_space(10.0);
            ui.add_enabled_ui(!is_busy,|ui|{ui.horizontal_wrapped(|ui|{if ui.button(tr(language,"Verificar","Check")).clicked(){self.run(Action::Check);}if ui.button(tr(language,"Prévia","Dry Run")).clicked(){self.run(Action::DryRun);}if ui.button(tr(language,"Notas da release","Release notes")).clicked(){self.run(Action::ReleaseNotes);}if ui.button(tr(language,"Atualizar","Update")).clicked(){self.run(Action::Update);}if ui.button(tr(language,"Versão","Version")).clicked(){self.run(Action::Version);}if ui.button(tr(language,"Reverter","Rollback")).clicked(){self.run(Action::Rollback);}if ui.button(tr(language,"Diagnóstico","Doctor")).clicked(){self.run(Action::Doctor);}if ui.button(tr(language,"Histórico","History")).clicked(){self.run(Action::History);}});});
            ui.horizontal_wrapped(|ui|{if ui.button(tr(language,"Abrir pasta da versão atual","Open current version folder")).clicked(){self.open_current_folder();}if ui.button(tr(language,"Abrir página da release","Open release page")).clicked(){self.open_release_page();}if ui.button(tr(language,"Log técnico","Technical log")).clicked(){self.log_open=true;}if is_busy&&ui.button(tr(language,"Cancelar download","Cancel download")).clicked(){self.cancel.store(true,Ordering::SeqCst);self.set_status(tr(language,"Cancelamento solicitado. A instalação atual será preservada.","Cancellation requested. The current installation will be preserved."));}});if is_busy{ui.horizontal(|ui|{ui.spinner();ui.label(tr(language,"Processando...","Working..."));});}
            ui.separator();ui.label(tr(language,"Situação","Status"));let mut text=self.status.lock().map(|s|s.clone()).unwrap_or_default();ui.add(egui::TextEdit::multiline(&mut text).desired_rows(12).desired_width(f32::INFINITY).font(egui::TextStyle::Monospace).interactive(false));});
        if self.settings_open {
            self.settings_window(ctx, is_busy);
        }
        if self.log_open {
            let mut open = self.log_open;
            let mut log = features_v14::read_log(&self.config());
            egui::Window::new(tr(
                language,
                "Log técnico — Info / Warning / Error",
                "Technical log — Info / Warning / Error",
            ))
            .open(&mut open)
            .default_width(720.0)
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut log)
                        .desired_rows(24)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .interactive(false),
                );
            });
            self.log_open = open;
        }
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut v = if dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let gold = egui::Color32::from_rgb(205, 151, 77);
    v.selection.bg_fill = gold;
    v.hyperlink_color = gold;
    ctx.set_visuals(v);
}
