use crate::{asset_select, download_resume, release, verify};
use reqwest::blocking::Client;
use std::{fs, io, path::PathBuf, process::Command, time::Duration};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run() -> Result<String, String> {
    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build().map_err(|e| e.to_string())?;
    let fetched = release::fetch_latest(&client, "https://api.github.com", "lowdrus/distronomicondesktop", None, false, &release::Validators::default())?;
    let latest = fetched.release.ok_or_else(|| "No self-update release available".to_string())?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if latest.tag_name == current { return Ok(format!("Already running latest version: {current}")); }

    let asset = asset_select::select(&latest.assets, r"(?i)^DistronomiconDesktop\.exe$")?.clone();
    let checksum = verify::select_asset(&latest.assets, r"(?i)^SHA256SUMS\.txt$")?.clone();
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = exe.parent().ok_or_else(|| "Executable has no parent directory".to_string())?;
    let work = base.join(".distronomicon").join("self-update");
    fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let new_exe = work.join("DistronomiconDesktop.new.exe");
    download_resume::download(&client, None, &asset, &new_exe)?;
    verify::verify_sha256(&client, None, &asset.name, &new_exe, &checksum)?;

    let script = work.join("apply-update.cmd");
    let body = format!(
        "@echo off\r\nping 127.0.0.1 -n 3 >nul\r\nmove /Y \"{}\" \"{}\" >nul\r\nstart \"\" \"{}\"\r\ndel \"%~f0\"\r\n",
        new_exe.display(), exe.display(), exe.display()
    );
    fs::write(&script, body).map_err(|e| e.to_string())?;
    spawn_hidden(&script).map_err(|e| e.to_string())?;
    std::process::exit(0);
}

fn spawn_hidden(script: &PathBuf) -> io::Result<()> {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/C", &script.display().to_string()]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.spawn()?;
    Ok(())
}
