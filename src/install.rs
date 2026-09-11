#[path = "extract.rs"]
mod extract;

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = staging_parent.join(format!("{tag}.{suffix}"));
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;

    let result = match extract::unpack(asset_name, downloaded_path, &staging) {
        Ok(true) => Ok(()),
        Ok(false) => fs::copy(downloaded_path, staging.join(asset_name))
            .map(|_| ())
            .map_err(|e| e.to_string()),
        Err(e) => Err(e),
    };
    if let Err(e) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(e);
    }

    sync_tree(&staging).map_err(|e| e.to_string())?;
    let target = releases.join(tag);
    if target.exists() {
        let _ = fs::remove_dir_all(&staging);
        return Err(format!("release already exists: {}", target.display()));
    }
    fs::rename(&staging, &target).map_err(|e| e.to_string())?;
    activate_release(root, app, tag).map_err(|e| e.to_string())?;
    Ok(target)
}

pub fn current_tag(root: &Path, app: &str) -> io::Result<Option<String>> {
    let path = root.join(app).join("current.txt");
    if !path.exists() {
        return Ok(None);
    }
    let tag = fs::read_to_string(path)?.trim().to_string();
    Ok((!tag.is_empty()).then_some(tag))
}

pub fn list_releases(root: &Path, app: &str) -> io::Result<Vec<String>> {
    let releases = root.join(app).join("releases");
    if !releases.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(releases)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            Some((
                e.file_name().to_string_lossy().to_string(),
                e.metadata().ok()?.modified().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    Ok(entries.into_iter().map(|(tag, _)| tag).collect())
}

pub fn activate_release(root: &Path, app: &str, tag: &str) -> io::Result<()> {
    let app_root = root.join(app);
    let release_dir = app_root.join("releases").join(tag);
    if !release_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("release not found: {}", release_dir.display()),
        ));
    }
    refresh_bin(&app_root.join("bin"), &release_dir)?;
    write_current_tag(root, app, tag)
}

fn write_current_tag(root: &Path, app: &str, tag: &str) -> io::Result<()> {
    let app_root = root.join(app);
    fs::create_dir_all(&app_root)?;
    let current = app_root.join("current.txt");
    let temp = app_root.join("current.tmp");
    fs::write(&temp, tag)?;
    if current.exists() {
        fs::remove_file(&current)?;
    }
    fs::rename(temp, current)
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

fn mirror_tree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            mirror_tree(&source_path, &destination_path)?;
        } else if file_type.is_file()
            && fs::hard_link(&source_path, &destination_path).is_err()
        {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn refresh_bin(bin_dir: &Path, release_dir: &Path) -> io::Result<()> {
    let temp = bin_dir.with_extension("new");
    let old = bin_dir.with_extension("old");
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::remove_dir_all(&old);

    mirror_tree(release_dir, &temp)?;
    sync_tree(&temp)?;

    if bin_dir.exists() {
        fs::rename(bin_dir, &old)?;
    }
    if let Err(error) = fs::rename(&temp, bin_dir) {
        if old.exists() && !bin_dir.exists() {
            let _ = fs::rename(&old, bin_dir);
        }
        return Err(error);
    }
    let _ = fs::remove_dir_all(old);
    Ok(())
}

pub fn prune_old_releases(
    releases_dir: &Path,
    current_tag: &str,
    retain: usize,
) -> io::Result<Vec<String>> {
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(releases_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            Some((
                e.file_name().to_string_lossy().to_string(),
                e.path(),
                e.metadata().ok()?.modified().ok()?,
            ))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.0.cmp(&a.0)));
    let mut deleted = Vec::new();
    for (tag, path, _) in entries.into_iter().skip(retain) {
        if tag != current_tag && fs::remove_dir_all(path).is_ok() {
            deleted.push(tag);
        }
    }
    Ok(deleted)
}
