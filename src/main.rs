#![windows_subsystem = "windows"]

mod check;
mod config;
mod install;
mod lock;
mod release;
mod restart;
mod state;
mod update;
mod verify;

use config::{tr, Action, Config, Language};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread,
    time::Duration,
};

struct DesktopApp {
    language: Language,
    dark_mode: bool,
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
    status: Arc<Mutex<String>>,
    busy: Arc<AtomicBool>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        let base = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(PathBuf::from))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            language: Language::PtBr,
            dark_mode: true,
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
            status: Arc::new(Mutex::new("Pronto. Configure a aplicação e o repositório.".into())),
            busy: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl DesktopApp {
    fn config(&self) -> Config {
        Config {
            language: self.language,
            app_name: self.app_name.trim().to_string(),
            repo: self.repo.trim().to_string(),
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
        }
    }

    fn set_status(&self, text: impl Into<String>) {
        if let Ok(mut status) = self.status.lock() { *status = text.into(); }
    }

    fn run(&self, action: Action) {
        if self.busy.swap(true, Ordering::SeqCst) { return; }
        let config = self.config();
        if let Err(message) = config::validate(&config, action) {
            self.set_status(message);
            self.busy.store(false, Ordering::SeqCst);
            return;
        }

        let status = Arc::clone(&self.status);
        let busy = Arc::clone(&self.busy);
        thread::spawn(move || {
            let language = config.language;
            if matches!(action, Action::Check)
                && let Ok(mut text) = status.lock() {
                *text = tr(language, "Consultando releases no GitHub...", "Checking GitHub releases...").into();
            }

            let result = match action {
                Action::Check => check::check(&config),
                Action::Update => update::run(&config, &status),
                Action::Version => check::version(&config),
                Action::Rollback => check::rollback(&config),
                Action::Doctor => check::doctor(&config),
                Action::Unlock => check::unlock(&config),
            };

            if let Ok(mut text) = status.lock() {
                *text = match result {
                    Ok(message) => message,
                    Err(error) => format!("{}: {error}", tr(language, "Erro", "Error")),
                };
            }
            busy.store(false, Ordering::SeqCst);
        });
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);
        let language = self.language;
        let is_busy = self.busy.load(Ordering::SeqCst);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Distronomicon Desktop");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.dark_mode, "◐").on_hover_text(tr(
                        language,
                        "Alternar modo claro/escuro",
                        "Toggle light/dark mode",
                    ));
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(self.language.label())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.language, Language::PtBr, "PT-BR");
                            ui.selectable_value(&mut self.language, Language::En, "EN");
                        });
                });
            });

            ui.label(tr(language,
                "Gerenciador de releases portátil e nativo para Windows.",
                "Portable native release manager for Windows."));
            ui.small(tr(language,
                "Sem WSL, terminal ou instalação.",
                "No WSL, terminal, or installation required."));
            ui.add_space(14.0);

            ui.label(tr(language, "Aplicação", "Application"));
            ui.text_edit_singleline(&mut self.app_name);
            ui.small(tr(
                language,
                "Nome local que você escolhe para identificar o programa gerenciado. Ex.: meu-app, servidor, cinetwitch.",
                "A local name you choose to identify the managed program. Examples: my-app, server, cinetwitch.",
            ));
            ui.add_space(8.0);

            ui.label(tr(language, "Repositório GitHub", "GitHub repository"));
            ui.text_edit_singleline(&mut self.repo);
            ui.small("owner/repository");
            ui.add_space(14.0);

            ui.add_enabled_ui(!is_busy, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(tr(language, "Verificar", "Check")).clicked() { self.run(Action::Check); }
                    if ui.button(tr(language, "Atualizar", "Update")).clicked() { self.run(Action::Update); }
                    if ui.button(tr(language, "Versão", "Version")).clicked() { self.run(Action::Version); }
                    if ui.button("Rollback").clicked() { self.run(Action::Rollback); }
                    if ui.button(tr(language, "Diagnóstico", "Doctor")).clicked() { self.run(Action::Doctor); }
                });
            });

            if is_busy {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tr(language, "Processando...", "Working..."));
                });
            }

            ui.add_space(14.0);
            egui::CollapsingHeader::new(tr(language, "Opções avançadas", "Advanced options"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.label(tr(language, "Pasta de instalação", "Install folder"));
                    ui.text_edit_singleline(&mut self.install_root);
                    ui.add_space(6.0);

                    ui.label(tr(language, "Pasta de estado", "State folder"));
                    ui.text_edit_singleline(&mut self.state_root);
                    ui.add_space(6.0);

                    ui.label(tr(language, "Padrão do asset (Regex)", "Asset pattern (Regex)"));
                    ui.text_edit_singleline(&mut self.asset_pattern);
                    ui.add_space(6.0);

                    ui.label(tr(language, "Padrão do checksum (Regex)", "Checksum pattern (Regex)"));
                    ui.text_edit_singleline(&mut self.checksum_pattern);
                    ui.add_space(6.0);

                    ui.checkbox(&mut self.skip_verification, tr(language,
                        "Pular verificação SHA-256 (não recomendado)",
                        "Skip SHA-256 verification (not recommended)"));
                    ui.checkbox(&mut self.allow_prerelease, tr(language,
                        "Permitir prereleases",
                        "Allow prereleases"));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(tr(language, "Manter releases recentes", "Keep recent releases"));
                        ui.add(egui::DragValue::new(&mut self.retain).range(0..=20));
                    });
                    ui.small(tr(language,
                        "A versão atual nunca é removida.",
                        "The current version is never removed."));
                    ui.add_space(6.0);

                    ui.label("GitHub API host");
                    ui.text_edit_singleline(&mut self.github_host);
                    ui.add_space(6.0);

                    ui.label(tr(language, "GitHub token (opcional)", "GitHub token (optional)"));
                    ui.add(egui::TextEdit::singleline(&mut self.github_token).password(true));
                    ui.small(tr(language,
                        "Para repositórios privados ou maior limite de API.",
                        "For private repositories or higher API limits."));
                    ui.add_space(6.0);

                    ui.label(tr(language,
                        "Comando após atualização/rollback (opcional)",
                        "Post-update/rollback command (optional)"));
                    ui.text_edit_singleline(&mut self.restart_command);
                    ui.add_space(8.0);

                    ui.add_enabled_ui(!is_busy, |ui| {
                        if ui.button(tr(language, "Forçar desbloqueio", "Force unlock")).clicked() {
                            self.run(Action::Unlock);
                        }
                    });
                    ui.small(tr(
                        language,
                        "Use somente se uma atualização anterior foi interrompida e deixou o lock preso. Não é necessário no uso normal.",
                        "Use only if a previous update was interrupted and left the lock stuck. It is not needed during normal use.",
                    ));
                });

            ui.add_space(14.0);
            ui.separator();
            ui.label("Status");
            let mut text = self.status.lock().map(|s| s.clone()).unwrap_or_default();
            ui.add(egui::TextEdit::multiline(&mut text)
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .interactive(false));
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn apply_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut visuals = if dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() };
    let golden_gate = egui::Color32::from_rgb(205, 151, 77);
    visuals.selection.bg_fill = golden_gate;
    visuals.hyperlink_color = golden_gate;
    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 700.0])
            .with_min_inner_size([580.0, 520.0])
            .with_title("Distronomicon Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Distronomicon Desktop",
        options,
        Box::new(|_cc| Ok(Box::<DesktopApp>::default())),
    )
}
