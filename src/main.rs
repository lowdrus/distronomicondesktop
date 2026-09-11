#![windows_subsystem = "windows"]

use eframe::egui;
use std::process::Command;
use std::sync::{Arc, Mutex};

const RUNNER: &str = r#"
set -e
APP="$1"
REPO="$2"
ACTION="$3"
BIN="$(command -v distronomicon || true)"
if [ -z "$BIN" ] && [ -x "/mnt/h/REPOSITORIOS GITHUB/distronomicon-main/target/release/distronomicon" ]; then
  BIN="/mnt/h/REPOSITORIOS GITHUB/distronomicon-main/target/release/distronomicon"
fi
if [ -z "$BIN" ]; then
  echo "Distronomicon não encontrado no Ubuntu/WSL." >&2
  echo "Instale em /usr/local/bin ou mantenha o binário compilado em /mnt/h/REPOSITORIOS GITHUB/distronomicon-main/target/release/distronomicon" >&2
  exit 127
fi
case "$ACTION" in
  version) exec "$BIN" --app "$APP" version ;;
  check) exec "$BIN" --app "$APP" check --repo "$REPO" ;;
  update) exec "$BIN" --app "$APP" update --repo "$REPO" ;;
  *) echo "Ação inválida" >&2; exit 2 ;;
esac
"#;

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
            output: Arc::new(Mutex::new("Pronto. Informe a aplicação e o repositório.".into())),
        }
    }
}

impl DesktopApp {
    fn execute(&self, action: &str) {
        let app = self.app_name.trim().to_string();
        let repo = self.repo.trim().to_string();
        if app.is_empty() || (action != "version" && !repo.contains('/')) {
            if let Ok(mut text) = self.output.lock() {
                *text = "Preencha o nome da aplicação e use o repositório no formato owner/repository.".into();
            }
            return;
        }

        let action = action.to_string();
        let output = Arc::clone(&self.output);
        std::thread::spawn(move || {
            if let Ok(mut text) = output.lock() {
                *text = format!("Executando {action} no Ubuntu/WSL...\n\n");
            }

            let result = Command::new("wsl.exe")
                .args(["-d", "Ubuntu", "bash", "-lc", RUNNER, "_", &app, &repo, &action])
                .output();

            let final_text = match result {
                Ok(result) => {
                    let mut text = String::new();
                    text.push_str(&String::from_utf8_lossy(&result.stdout));
                    text.push_str(&String::from_utf8_lossy(&result.stderr));
                    if text.trim().is_empty() {
                        text = if result.status.success() { "Concluído com sucesso.".into() } else { format!("Falha com código {:?}.", result.status.code()) };
                    }
                    text
                }
                Err(err) => format!("Não foi possível iniciar o WSL/Ubuntu: {err}"),
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

            ui.label("Repositório GitHub (owner/repository)");
            ui.text_edit_singleline(&mut self.repo);
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                if ui.button("Verificar").clicked() { self.execute("check"); }
                if ui.button("Versão").clicked() { self.execute("version"); }
                if ui.button("Atualizar").clicked() { self.execute("update"); }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.label("Saída");

            let mut text = self.output.lock().map(|s| s.clone()).unwrap_or_default();
            ui.add(egui::TextEdit::multiline(&mut text)
                .desired_rows(14)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .interactive(false));

            ui.add_space(8.0);
            ui.small("Requer WSL 2 + Ubuntu. O app procura distronomicon no PATH e também no diretório usado neste projeto.");
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 520.0])
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
