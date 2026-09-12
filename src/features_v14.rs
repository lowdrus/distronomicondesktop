use crate::{config::{Config, tr}, install, release, state};
use std::{
    cmp::Ordering,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn log(config: &Config, level: &str, message: &str) {
    let dir = PathBuf::from(&config.state_root).join(&config.app_name);
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("technical.log");
    let line = format!("{} [{}] {}\n", state::now_unix(), level, message.replace('\n', " | "));
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let _ = file.write_all(line.as_bytes());
    }
}

pub fn read_log(config: &Config) -> String {
    fs::read_to_string(PathBuf::from(&config.state_root).join(&config.app_name).join("technical.log"))
        .unwrap_or_default()
}

pub fn test_connectivity(client: &reqwest::blocking::Client, config: &Config) -> Result<(), String> {
    let host = config.github_host.trim_end_matches('/');
    let url = format!("{host}/repos/{}", config.repo);
    let mut request = client.get(url).timeout(Duration::from_secs(12));
    if !config.github_token.trim().is_empty() {
        request = request.bearer_auth(config.github_token.trim());
    }
    request
        .send()
        .map_err(|e| format!("GitHub connectivity test failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub connectivity test failed: {e}"))?;
    Ok(())
}

pub fn estimated_required_bytes(asset: &release::Asset) -> u64 {
    if asset.size == 0 { 256 * 1024 * 1024 } else { asset.size.saturating_mul(3) }
}

pub fn ensure_free_space(path: &Path, required: u64) -> Result<u64, String> {
    fs::create_dir_all(path).map_err(|e| e.to_string())?;
    let available = fs2::available_space(path).map_err(|e| e.to_string())?;
    if available < required {
        return Err(format!(
            "Insufficient disk space: required about {} MB, available {} MB",
            required / 1024 / 1024,
            available / 1024 / 1024
        ));
    }
    Ok(available)
}

pub fn backup_configurable_files(config: &Config) -> Result<Option<PathBuf>, String> {
    let items = config
        .backup_paths
        .split([';', '\n', '\r'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if items.is_empty() {
        return Ok(None);
    }
    let stamp = state::now_unix();
    let backup_root = PathBuf::from(&config.state_root)
        .join(&config.app_name)
        .join("backups")
        .join(stamp.to_string());
    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    let bin = PathBuf::from(&config.install_root).join(&config.app_name).join("bin");
    for item in items {
        let source = PathBuf::from(item);
        let source = if source.is_absolute() { source } else { bin.join(source) };
        if !source.exists() {
            log(config, "WARNING", &format!("Backup item not found: {}", source.display()));
            continue;
        }
        let name = source.file_name().ok_or_else(|| "Invalid backup path".to_string())?;
        let destination = backup_root.join(name);
        copy_any(&source, &destination).map_err(|e| e.to_string())?;
    }
    log(config, "INFO", &format!("Backup created at {}", backup_root.display()));
    Ok(Some(backup_root))
}

fn copy_any(source: &Path, destination: &Path) -> io::Result<()> {
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_any(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::copy(source, destination)?;
    }
    Ok(())
}

pub fn recover_state(config: &Config) -> Result<String, String> {
    let state_path = PathBuf::from(&config.state_root).join(&config.app_name).join("state.json");
    if state::load(&state_path).is_ok_and(|v| v.is_some()) {
        return Ok(tr(config.language, "Estado válido; nenhuma recuperação necessária.", "State is valid; no recovery is required.").into());
    }
    let install_root = PathBuf::from(&config.install_root);
    let current = install::current_tag(&install_root, &config.app_name)
        .map_err(|e| e.to_string())?
        .or_else(|| install::list_releases(&install_root, &config.app_name).ok().and_then(|v| v.into_iter().next()))
        .ok_or_else(|| tr(config.language, "Nenhuma release instalada foi encontrada para reconstruir o estado.", "No installed release was found to rebuild state.").to_string())?;
    let recovered = state::State {
        latest_tag: current.clone(),
        etag: String::new(),
        last_modified: String::new(),
        installed_at_unix: state::now_unix(),
    };
    state::save_atomic(&state_path, &recovered).map_err(|e| e.to_string())?;
    log(config, "WARNING", &format!("State rebuilt from releases/current.txt: {current}"));
    Ok(format!("{}: {current}", tr(config.language, "Estado reconstruído", "State rebuilt")))
}

pub fn is_downgrade(current: &str, target: &str) -> bool {
    compare_versions(target, current) == Ordering::Less
}

fn compare_versions(a: &str, b: &str) -> Ordering {
    let ka = version_key(a);
    let kb = version_key(b);
    ka.cmp(&kb).then_with(|| a.cmp(b))
}

fn version_key(value: &str) -> Vec<u64> {
    value
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .take(4)
        .map(|s| s.parse::<u64>().unwrap_or(0))
        .collect()
}

pub fn verify_authenticode_tree(config: &Config, root: &Path) -> Result<String, String> {
    if !config.verify_authenticode {
        return Ok("Authenticode disabled".into());
    }
    let mut exes = Vec::new();
    collect_exes(root, &mut exes).map_err(|e| e.to_string())?;
    let mut signed = 0usize;
    for exe in exes {
        match authenticode_status(&exe)? {
            status if status.eq_ignore_ascii_case("Valid") => signed += 1,
            status if status.eq_ignore_ascii_case("NotSigned") || status.is_empty() => {}
            status => return Err(format!("Authenticode validation failed for {}: {status}", exe.display())),
        }
    }
    Ok(if signed == 0 { "No signed executable found".into() } else { format!("Authenticode valid for {signed} executable(s)") })
}

fn collect_exes(root: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if root.is_file() {
        if root.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) { out.push(root.to_path_buf()); }
        return Ok(());
    }
    if !root.exists() { return Ok(()); }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() { collect_exes(&p, out)?; }
        else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")) { out.push(p); }
    }
    Ok(())
}

fn authenticode_status(path: &Path) -> Result<String, String> {
    #[cfg(windows)]
    {
        let escaped = path.display().to_string().replace('\'', "''");
        let script = format!("(Get-AuthenticodeSignature -LiteralPath '{escaped}').Status.ToString()");
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()); }
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    #[cfg(not(windows))]
    { let _ = path; Ok(String::new()) }
}

pub fn notify(config: &Config, title: &str, message: &str) {
    if !config.notifications { return; }
    #[cfg(windows)]
    {
        let title = ps_escape(title);
        let message = ps_escape(message);
        let script = format!(
            "$ErrorActionPreference='SilentlyContinue'; [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType=WindowsRuntime] > $null; [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType=WindowsRuntime] > $null; $x=New-Object Windows.Data.Xml.Dom.XmlDocument; $x.LoadXml('<toast><visual><binding template=\"ToastGeneric\"><text>{title}</text><text>{message}</text></binding></visual></toast>'); $t=[Windows.UI.Notifications.ToastNotification]::new($x); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Distronomicon Desktop').Show($t)"
        );
        let _ = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
}

fn ps_escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

pub fn open_path(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        Command::new("explorer.exe").arg(path).creation_flags(CREATE_NO_WINDOW).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    { let _ = path; Err("Windows only".into()) }
}

pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        Command::new("cmd.exe").args(["/C", "start", "", url]).creation_flags(CREATE_NO_WINDOW).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    { let _ = url; Err("Windows only".into()) }
}

pub fn current_release_path(config: &Config) -> Result<PathBuf, String> {
    let root = PathBuf::from(&config.install_root);
    let tag = install::current_tag(&root, &config.app_name).map_err(|e| e.to_string())?.ok_or_else(|| "No active release".to_string())?;
    Ok(root.join(&config.app_name).join("releases").join(tag))
}

pub fn prune_policy(config: &Config, current_tag: &str) -> Result<Vec<String>, String> {
    let releases_dir = PathBuf::from(&config.install_root).join(&config.app_name).join("releases");
    if !releases_dir.exists() { return Ok(Vec::new()); }
    let mut entries = fs::read_dir(&releases_dir).map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| {
            let modified = e.metadata().and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
            (e.file_name().to_string_lossy().to_string(), e.path(), modified)
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.2.cmp(&a.2));
    let now = SystemTime::now();
    let mut deleted = Vec::new();
    let mut keep = Vec::new();
    for (index, entry) in entries.into_iter().enumerate() {
        let age_days = now.duration_since(entry.2).unwrap_or_default().as_secs() / 86_400;
        let too_old = config.retention_days > 0 && age_days > config.retention_days;
        let over_count = config.retain > 0 && index >= config.retain;
        if entry.0 != current_tag && (too_old || over_count) {
            if fs::remove_dir_all(&entry.1).is_ok() { deleted.push(entry.0); }
        } else { keep.push(entry); }
    }
    if config.max_disk_mb > 0 {
        let limit = config.max_disk_mb.saturating_mul(1024 * 1024);
        keep.sort_by(|a, b| a.2.cmp(&b.2));
        let mut total = keep.iter().map(|e| dir_size(&e.1)).sum::<u64>();
        for (tag, path, _) in keep {
            if total <= limit { break; }
            if tag == current_tag { continue; }
            let size = dir_size(&path);
            if fs::remove_dir_all(&path).is_ok() { total = total.saturating_sub(size); deleted.push(tag); }
        }
    }
    Ok(deleted)
}

fn dir_size(path: &Path) -> u64 {
    if path.is_file() { return path.metadata().map(|m| m.len()).unwrap_or(0); }
    fs::read_dir(path).ok().into_iter().flatten().filter_map(Result::ok).map(|e| dir_size(&e.path())).sum()
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 { format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
    else { format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_semver_downgrade() {
        assert!(is_downgrade("v2.0.0", "v1.9.9"));
        assert!(!is_downgrade("v1.2.0", "v1.3.0"));
    }
}
