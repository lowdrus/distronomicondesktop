use crate::{
    asset_select,
    config::{Language, current_language, tr},
    download_resume, release, verify,
};
use reqwest::blocking::Client;
use std::{fs, io, path::PathBuf, process::Command, time::Duration};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run() -> Result<String, String> {
    run_language(current_language())
}

pub fn run_language(language: Language) -> Result<String, String> {
    let client = Client::builder()
        .user_agent(concat!("DistronomiconDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let fetched = release::fetch_latest(
        &client,
        "https://api.github.com",
        "lowdrus/distronomicondesktop",
        None,
        false,
        &release::Validators::default(),
    )?;
    let latest = fetched.release.ok_or_else(|| {
        tr(
            language,
            "Nenhuma versão de autoatualização está disponível.",
            "No self-update release is available.",
        )
        .to_string()
    })?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if compare_versions(&latest.tag_name, &current) <= 0 {
        return Ok(format!(
            "{}: {current}",
            tr(
                language,
                "O Distronomicon Desktop já está na versão mais recente",
                "Distronomicon Desktop is already up to date"
            )
        ));
    }

    let asset = asset_select::select(&latest.assets, r"(?i)^DistronomiconDesktop\.exe$")?.clone();
    let checksum = verify::select_asset(&latest.assets, r"(?i)^SHA256SUMS\.txt$")?.clone();
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = exe.parent().ok_or_else(|| {
        tr(
            language,
            "Não foi possível localizar a pasta do executável.",
            "Could not locate the executable directory.",
        )
        .to_string()
    })?;
    let work = base.join(".distronomicon").join("self-update");
    fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let new_exe = work.join("DistronomiconDesktop.new.exe");
    download_resume::download(&client, None, &asset, &new_exe)?;
    verify::verify_sha256(&client, None, &asset.name, &new_exe, &checksum)?;

    let script = work.join("apply-update.cmd");
    let body = format!(
        "@echo off\r\nping 127.0.0.1 -n 3 >nul\r\nmove /Y \"{}\" \"{}\" >nul\r\nstart \"\" \"{}\"\r\ndel \"%~f0\"\r\n",
        new_exe.display(),
        exe.display(),
        exe.display()
    );
    fs::write(&script, body).map_err(|e| e.to_string())?;
    spawn_hidden(&script).map_err(|e| e.to_string())?;
    std::process::exit(0);
}

fn compare_versions(a: &str, b: &str) -> i32 {
    let parse = |value: &str| {
        value
            .trim_start_matches(['v', 'V'])
            .split(|c: char| !c.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .take(4)
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let left = parse(a);
    let right = parse(b);
    for index in 0..left.len().max(right.len()) {
        let l = *left.get(index).unwrap_or(&0);
        let r = *right.get(index).unwrap_or(&0);
        if l > r {
            return 1;
        }
        if l < r {
            return -1;
        }
    }
    0
}

fn spawn_hidden(script: &PathBuf) -> io::Result<()> {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/C", &script.display().to_string()]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::compare_versions;

    #[test]
    fn compares_semver_like_tags() {
        assert_eq!(compare_versions("v1.3.0", "v1.2.1"), 1);
        assert_eq!(compare_versions("v1.2.1", "v1.2.1"), 0);
        assert_eq!(compare_versions("v1.2.0", "v1.2.1"), -1);
    }
}
