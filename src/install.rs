use crate::extract;
use std::{fs, io, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

pub fn install_release(root: &Path, app: &str, tag: &str, asset_name: &str, downloaded_path: &Path) -> Result<PathBuf, String> {
    let app_root = root.join(app);
    let releases = app_root.join("releases");
    let staging_parent = app_root.join("staging");
    fs::create_dir_all(&releases).map_err(|e| e.to_string())?;
    fs::create_dir_all(&staging_parent).map_err(|e| e.to_string())?;
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let staging = staging_parent.join(format!("{tag}.{suffix}"));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;

    let archived = extract::is_supported_archive(asset_name);
    let result = if archived {
        extract::unpack(downloaded_path, &staging)
    } else {
        fs::copy(downloaded_path, staging.join(asset_name)).map(|_| ()).map_err(|e| e.to_string())
    };
    if let Err(e) = result { let _ = fs::remove_dir_all(&staging); return Err(e); }

    sync_tree(&staging).map_err(|e| e.to_string())?;
    let target = releases.join(tag);
    if target.exists() { let _ = fs::remove_dir_all(&staging); return Err(format!("release already exists: {}", target.display())); }
    fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    refresh_bin(&app_root.join("bin"), &target).map_err(|e| e.to_string())?;
    write_current_tag(root, app, tag).map_err(|e| e.to_string())?;
    Ok(target)
}

pub fn current_tag(root: &Path, app: &str) -> io::Result<Option<String>> {
    let path = root.join(app).join("current.txt");
    if !path.exists() { return Ok(None); }
    let tag = fs::read_to_string(path)?.trim().to_string();
    Ok((!tag.is_empty()).then_some(tag))
}

fn write_current_tag(root: &Path, app: &str, tag: &str) -> io::Result<()> {
    let app_root = root.join(app);
    fs::create_dir_all(&app_root)?;
    let current = app_root.join("current.txt");
    let temp = app_root.join("current.tmp");
    fs::write(&temp, tag)?;
    if current.exists() { fs::remove_file(&current)?; }
    fs::rename(temp, current)
}

fn sync_tree(path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?; let p = entry.path();
        if entry.file_type()?.is_dir() { sync_tree(&p)?; }
        else if entry.file_type()?.is_file() { fs::File::open(&p)?.sync_all()?; }
    }
    Ok(())
}

fn is_windows_executable(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()).map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "exe"|"com"|"bat"|"cmd")).unwrap_or(false)
}

fn collect_executables(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?; let p = entry.path();
        if entry.file_type()?.is_dir() { collect_executables(&p, out)?; }
        else if entry.file_type()?.is_file() && is_windows_executable(&p) { out.push(p); }
    }
    Ok(())
}

fn refresh_bin(bin_dir: &Path, release_dir: &Path) -> io::Result<()> {
    let temp = bin_dir.with_extension("new"); let old = bin_dir.with_extension("old");
    let _ = fs::remove_dir_all(&temp); let _ = fs::remove_dir_all(&old); fs::create_dir_all(&temp)?;
    let mut executables = Vec::new(); collect_executables(release_dir, &mut executables)?;
    for source in executables { if let Some(name) = source.file_name() { fs::copy(&source, temp.join(name))?; } }
    if bin_dir.exists() { fs::rename(bin_dir, &old)?; }
    if let Err(e) = fs::rename(&temp, bin_dir) {
        if old.exists() && !bin_dir.exists() { let _ = fs::rename(&old, bin_dir); }
        return Err(e);
    }
    let _ = fs::remove_dir_all(old); Ok(())
}

pub fn prune_old_releases(releases_dir: &Path, current_tag: &str, retain: usize) -> io::Result<Vec<String>> {
    if !releases_dir.exists() { return Ok(Vec::new()); }
    let mut entries = fs::read_dir(releases_dir)?.filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| Some((e.file_name().to_string_lossy().to_string(), e.path(), e.metadata().ok()?.modified().ok()?)))
        .collect::<Vec<_>>();
    entries.sort_by(|a,b| b.2.cmp(&a.2).then_with(|| b.0.cmp(&a.0)));
    let mut deleted = Vec::new();
    for (tag,path,_) in entries.into_iter().skip(retain) { if tag != current_tag && fs::remove_dir_all(path).is_ok() { deleted.push(tag); } }
    Ok(deleted)
}
