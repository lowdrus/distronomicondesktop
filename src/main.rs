#![windows_subsystem = "windows"]

use eframe::egui;
use std::process::Command;
use std::sync::{Arc, Mutex};

struct DesktopApp {
    app_name: String,
    repo: String,
    output: Arc<Mutex<String>>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self {
            app_name: "meu-app".into(),
            repo: "owner/repository".into(),
            output: Arc::new(Mutex::new("Pronto.".into())),
        }
    }
}

impl DesktopApp {
    fn execute(&self, action: &str) {
        let app = self.app_name.trim().to_string();
        let repo = self.repo.trim().to_string();
        let command = match action {
            "version" => format!("distronomicon --app {} version", app),
            "check" => format!("distronomicon --app {} check --repo {}", app, repo),
            "update" => format!("distronomicon --app {} update --repo {}", app, repo),
            _ => return,
        };

        let output = Arc::clone(&self.output);
        std::thread::spawn(move || {
            if let Ok(mut text) = output.lock() {
                *text = format!("Executando:\n{}\n\n", command);
            }

            let result = Command::new("wsl.exe")
                .args(["-d", "Ubuntu", "bash", "-lc", &command])
                .output();

            let final_text = match result {
                Ok(result) => format!(
                    "{}{}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                ),
                Err(err) => format!("Erro ao iniciar WSL/Ubuntu: {err}"),
            };

            if let Ok(mut text) = output.lock() {
                text.push_str(&final_text);
            }
        });
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Distronomicon Desktop");
            ui.label("Interface portátil para Windows usando Ubuntu/WSL.");
            ui.add_space(12.0);

            ui.label("Aplicação");
            ui.text_edit_singleline(&mut self.app_name);
            ui.add_space(8.0);

            ui.label("Repositório GitHub (owner/repo)");
            ui.text_edit_singleline(&mut self.repo);
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                if ui.button("Verificar").clicked() {
                    self.execute("check");
                }
                if ui.button("Versão").clicked() {
                    self.execute("version");
                }
                if ui.button("Atualizar").clicked() {
                    self.execute("update");
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.label("Saída");

            let mut text = self.output.lock().map(|s| s.clone()).unwrap_or_default();
            ui.add(
                egui::TextEdit::multiline(&mut text)
                    .desired_rows(14)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .interactive(false),
            );

            ui.add_space(8.0);
            ui.small("Requer WSL + Ubuntu e o comando distronomicon instalado no Ubuntu.");
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 500.0])
            .with_min_inner_size([520.0, 400.0])
            .with_title("Distronomicon Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Distronomicon Desktop",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::<DesktopApp>::default())
        }),
    )
}
