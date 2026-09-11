#![windows_subsystem = "windows"]

mod app;
mod asset_select;
mod atomic_file;
mod check;
mod check_v13;
mod config;
mod download_resume;
mod history;
mod install;
mod lock;
mod maintenance;
mod plan;
mod profiles;
mod release;
mod restart;
mod scheduled;
mod scheduler;
mod self_update;
mod state;
mod update;
mod update_v13;
mod verify;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(code) = scheduled::maybe_run(&args) {
        std::process::exit(code);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([840.0, 820.0])
            .with_min_inner_size([680.0, 560.0])
            .with_title("Distronomicon Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Distronomicon Desktop",
        options,
        Box::new(|_cc| Ok(Box::<app::DesktopApp>::default())),
    )
}
