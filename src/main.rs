#![windows_subsystem = "windows"]

mod app_v14;
mod asset_select;
mod atomic_file;
mod check;
mod check_v13;
mod config;
mod download_resume;
mod features_v14;
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
mod update_v14;
mod verify;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(code) = scheduled::maybe_run(&args) {
        std::process::exit(code);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 760.0])
            .with_min_inner_size([760.0, 580.0])
            .with_title("Distronomicon Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Distronomicon Desktop",
        options,
        Box::new(|_cc| Ok(Box::<app_v14::DesktopApp>::default())),
    )
}

#[cfg(test)]
mod documentation_screenshot {
    use super::*;
    use egui_kittest::Harness;

    #[test]
    #[ignore = "run explicitly in CI to generate the README screenshot"]
    fn render_real_ui_for_documentation() {
        let mut harness = Harness::new_eframe(|_cc| app_v14::DesktopApp::default());
        harness.set_size(egui::vec2(920.0, 760.0));
        harness.run_steps(2);
        harness.remove_cursor();
        harness.snapshot("distronomicon-desktop-real");
    }
}
