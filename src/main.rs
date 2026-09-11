#![windows_subsystem = "windows"]

use eframe::egui;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{fs, io::Write, path::PathBuf, sync::{Arc, Mutex}, thread, time::Duration};

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

struct DesktopApp {
    repo: String,
    asset_filter: String,
    install_dir: String,
    status: Arc<Mutex<String>>,
}

impl Default for DesktopApp {
    fn default() -> Self {
        let default_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("apps");
        Self {
            repo: "owner/repository".into(),
            asset_filter: ".zip".into(),
            install_dir: default_dir.display().to_string(),
            status: Arc::new(Mutex::new("Pronto. Informe o repositório e clique em Verificar.".into())),
        }
    }
}

impl DesktopApp {
    fn set_status(&self, text: impl Into<String>) {
        if let Ok(mut s) = self.status.lock() { *s = text.into(); }
    }

    fn run_check(&self, do_update: bool) {
        let repo = self.repo.trim().to_string();
        let filter = self.asset_filter.trim().to_string();
        let install_dir = self.install_dir.trim().to_string();
        let status = Arc::clone(&self.status);

        if !repo.contains('/') {
            self.set_status("Use o formato owner/repository.");
            return;
        }

        thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                if let Ok(mut s) = status.lock() { *s = "Consultando a release mais recente no GitHub...".into(); }

                let client = Client::builder()
                    .user_agent("DistronomiconDesktop/1.0")
                    .timeout(Duration::from_secs(30))
                    .build()
                    .map_err(|e| e.to_string())?;

                let api = format!("https://api.github.com/repos/{repo}/releases/latest");
                let release: Release = client.get(api).send().map_err(|e| e.to_string())?
                    .error_for_status().map_err(|e| e.to_string())?
                    .json().map_err(|e| e.to_string())?;

                let asset = release.assets.iter().find(|a| filter.is_empty() || a.name.to_lowercase().contains(&filter.to_lowercase()));

                if !do_update {
                    return Ok(match asset {
                        Some(a) => format!("Última versão: {}\nArquivo: {}\n{}", release.tag_name, a.name, release.html_url),
                        None => format!("Última versão: {}\nNenhum arquivo corresponde ao filtro '{}'.\n{}", release.tag_name, filter, release.html_url),
                    });
                }

                let asset = asset.ok_or_else(|| format!("Nenhum arquivo da release corresponde ao filtro '{}'.", filter))?;
                if let Ok(mut s) = status.lock() { *s = format!("Baixando {}...", asset.name); }

                let bytes = client.get(&asset.browser_download_url).send().map_err(|e| e.to_string())?
                    .error_for_status().map_err(|e| e.to_string())?
                    .bytes().map_err(|e| e.to_string())?;

                let base = PathBuf::from(&install_dir).join(repo.replace('/', "_"));
                fs::create_dir_all(&base).map_err(|e| e.to_string())?;
                let target = base.join(&asset.name);
                let mut file = fs::File::create(&target).map_err(|e| e.to_string())?;
                file.write_all(&bytes).map_err(|e| e.to_string())?;

                if asset.name.to_lowercase().ends_with(".zip") {
                    let reader = std::io::Cursor::new(bytes);
                    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
                    let extract_dir = base.join(release.tag_name.trim_start_matches('v'));
                    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
                    archive.extract(&extract_dir).map_err(|e| e.to_string())?;
                    Ok(format!("Atualização concluída.\nVersão: {}\nInstalada em: {}", release.tag_name, extract_dir.display()))
                } else {
                    Ok(format!("Download concluído.\nVersão: {}\nArquivo salvo em: {}", release.tag_name, target.display()))
                }
            })();

            if let Ok(mut s) = status.lock() {
                *s = match result { Ok(v) => v, Err(e) => format!("Erro: {e}") };
            }
        });
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Distronomicon Desktop");
            ui.label("Atualizador portátil nativo para Windows. Sem terminal, WSL ou instalação.");
            ui.add_space(14.0);

            ui.label("Repositório GitHub");
            ui.text_edit_singleline(&mut self.repo);
            ui.add_space(8.0);

            ui.label("Filtro do arquivo da release");
            ui.text_edit_singleline(&mut self.asset_filter);
            ui.small("Ex.: .zip, windows, win64, x64");
            ui.add_space(8.0);

            ui.label("Pasta de destino");
            ui.text_edit_singleline(&mut self.install_dir);
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                if ui.button("Verificar atualização").clicked() { self.run_check(false); }
                if ui.button("Baixar / Atualizar").clicked() { self.run_check(true); }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.label("Status");
            let mut text = self.status.lock().map(|s| s.clone()).unwrap_or_default();
            ui.add(egui::TextEdit::multiline(&mut text)
                .desired_rows(12)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace)
                .interactive(false));
            ui.add_space(8.0);
            ui.small("Funciona com releases públicas do GitHub e salva tudo dentro da pasta escolhida.");
        });
        ctx.request_repaint_after(Duration::from_millis(250));
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([620.0, 500.0])
            .with_min_inner_size([520.0, 420.0])
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
