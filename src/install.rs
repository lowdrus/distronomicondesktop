use std::{fs, io::{self, Read, Write}, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

const MAX_FILES: usize = 10_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 10 * 1024 * 1024 * 1024;

pub fn install_release(
    root: &Path,
    app: &str,
    tag: &str,
    asset_name: &str,
    downloaded_path: &Path,
) -> Result<PathBuf, String> {
    let app_root = root.join(app);
    let releases = app_root.join("releases");
    let staging_parent = app_root.join("staging");
    fs::create_dir_all(&releases).map_err(|e| e.to_string())?;
    fs::create_dir_all(&staging_parent).map_err(|e| e.to_string())?;

    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let staging = staging_parent.join(format!("{tag}.{suffix}"));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;

    let result = if asset_name.to_ascii_lowercase().ends_with(".zip") {
        extract_zip(downloaded_path, &staging)
    } else {
        fs::copy(downloaded_path, staging.join(asset_name)).map(|_| ()).map_err(|e| e.to_string())
    };

    if let Err(e) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }

    strip_single_root(&staging).map_err(|e| e.to_string())?;
    sync_tree(&staging).map_err(|e| e.to_string())?;

    let target = releases.join(tag);
    if target.exists() {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("release already exists: {}", target.display()));
    }

    fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    refresh_bin(&app_root.join("bin"), &target).map_err(|e| e.to_string())?;
    Ok(target)
}

fn extract_zip(source: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(source).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut files = 0usize;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let enclosed = entry.enclosed_name().ok_or_else(|| format!("unsafe archive path: {}", entry.name()))?;
        let out = destination.join(enclosed);

        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }

        if let Some(mode) = entry.unix_mode() {
            if mode & 0o170000 == 0o120000 {
                return Err(format!("symbolic links are not allowed in archives: {}", entry.name()));
            }
        }

        files += 1;
        if files > MAX_FILES { return Err("archive file-count limit exceeded".into()); }
        if entry.size() > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", entry.name())); }
        total = total.saturating_add(entry.size());
        if total > MAX_TOTAL_BYTES { return Err("archive extracted-size limit exceeded".into()); }

        if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let mut output = fs::File::create(&out).map_err(|e| e.to_string())?;
        let copied = io::copy(&mut entry.take(MAX_FILE_BYTES + 1), &mut output).map_err(|e| e.to_string())?;
        if copied > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", out.display())); }
        output.flush().map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn strip_single_root(destination: &Path) -> io::Result<()> {
    let entries = fs::read_dir(destination)?.collect::<Result<Vec<_>, _>>()?;
    if entries.len() != 1 || !entries[0].file_type()?.is_dir() { return Ok(()); }
    let root = entries[0].path();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        fs::rename(entry.path(), destination.join(entry.file_name()))?;
    }
    fs::remove_dir(root)?;
    Ok(())
}

fn sync_tree(path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if entry.file_type()?.is_dir() {
            sync_tree(&p)?;
        } else if entry.file_type()?.is_file() {
            fs::File::open(&p)?.sync_all()?;
        }
    }
    Ok(())
}

fn is_windows_executable(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "exe" | "com" | "bat" | "cmd"))
        .unwrap_or(false)
}

fn collect_executables(dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if entry.file_type()?.is_dir() { collect_executables(&p, found)?; }
        else if entry.file_type()?.is_file() && is_windows_executable(&p) { found.push(p); }
    }
    Ok(())
}

fn refresh_bin(bin_dir: &Path, release_dir: &Path) -> io::Result<()> {
    let temp = bin_dir.with_extension("new");
    let old = bin_dir.with_extension("old");
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::remove_dir_all(&old);
    fs::create_dir_all(&temp)?;

    let mut executables = Vec::new();
    collect_executables(release_dir, &mut executables)?;
    for source in executables {
        if let Some(name) = source.file_name() {
            fs::copy(&source, temp.join(name))?;
        }
    }

    if bin_dir.exists() { fs::rename(bin_dir, &old)?; }
    if let Err(e) = fs::rename(&temp, bin_dir) {
        if old.exists() && !bin_dir.exists() { let _ = fs::rename(&old, bin_dir); }
        return Err(e);
    }
    let _ = fs::remove_dir_all(old);
    Ok(())
}

pub fn prune_old_releases(releases_dir: &Path, current_tag: &str, retain: usize) -> io::Result<Vec<String>> {
    if !releases_dir.exists() { return Ok(Vec::new()); }
    let mut entries = fs::read_dir(releases_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((e.file_name().to_string_lossy().to_string(), e.path(), modified))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.0.cmp(&a.0)));

    let mut deleted = Vec::new();
    for (tag, path, _) in entries.into_iter().skip(retain) {
        if tag == current_tag { continue; }
        if fs::remove_dir_all(path).is_ok() { deleted.push(tag); }
    }
    Ok(deleted)
}
