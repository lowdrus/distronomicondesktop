use std::{fs, io::{self, Read, Write}, path::{Component, Path}};

const MAX_FILES: usize = 10_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 10 * 1024 * 1024 * 1024;

pub fn unpack(asset_name: &str, source: &Path, destination: &Path) -> Result<bool, String> {
    let name = asset_name.to_ascii_lowercase();
    if name.ends_with(".zip") {
        extract_zip(source, destination)?;
        return Ok(true);
    }
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") || name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tar.xz") || name.ends_with(".txz") || name.ends_with(".tar.zst") {
        extract_tar(source, destination, &name)?;
        return Ok(true);
    }
    Ok(false)
}

fn safe_relative(path: &Path) -> Result<(), String> {
    if path.is_absolute() { return Err("absolute archive paths are not allowed".into()); }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("archive paths containing '..' are not allowed".into());
    }
    Ok(())
}

fn extract_zip(source: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(source).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut files = 0usize;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        let enclosed = entry.enclosed_name().ok_or_else(|| format!("unsafe archive path: {}", entry.name()))?;
        safe_relative(&enclosed)?;
        let out = destination.join(enclosed);
        if entry.is_dir() { fs::create_dir_all(&out).map_err(|e| e.to_string())?; continue; }
        if let Some(mode) = entry.unix_mode() {
            if mode & 0o170000 == 0o120000 { return Err(format!("symbolic links are not allowed: {}", entry.name())); }
        }
        files += 1;
        if files > MAX_FILES { return Err("archive file-count limit exceeded".into()); }
        if entry.size() > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", entry.name())); }
        total = total.saturating_add(entry.size());
        if total > MAX_TOTAL_BYTES { return Err("archive extracted-size limit exceeded".into()); }
        if entry.compressed_size() > 0 && entry.size() / entry.compressed_size() > 100 { return Err(format!("archive decompression ratio exceeded: {}", entry.name())); }
        if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let mut output = fs::File::create(&out).map_err(|e| e.to_string())?;
        let copied = io::copy(&mut entry.take(MAX_FILE_BYTES + 1), &mut output).map_err(|e| e.to_string())?;
        if copied > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", out.display())); }
        output.flush().map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn extract_tar(source: &Path, destination: &Path, name: &str) -> Result<(), String> {
    let file = fs::File::open(source).map_err(|e| e.to_string())?;
    let reader: Box<dyn Read> = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Box::new(flate2::read::GzDecoder::new(file))
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        Box::new(bzip2::read::BzDecoder::new(file))
    } else if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        Box::new(xz2::read::XzDecoder::new(file))
    } else {
        Box::new(zstd::stream::read::Decoder::new(file).map_err(|e| e.to_string())?)
    };

    let mut archive = tar::Archive::new(reader);
    let mut total = 0u64;
    let mut files = 0usize;
    for item in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = item.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        safe_relative(&path)?;
        let kind = entry.header().entry_type();
        let out = destination.join(&path);
        if kind.is_dir() { fs::create_dir_all(&out).map_err(|e| e.to_string())?; continue; }
        if !kind.is_file() { return Err(format!("unsupported archive entry type: {}", path.display())); }
        let size = entry.header().size().map_err(|e| e.to_string())?;
        files += 1;
        if files > MAX_FILES { return Err("archive file-count limit exceeded".into()); }
        if size > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", path.display())); }
        total = total.saturating_add(size);
        if total > MAX_TOTAL_BYTES { return Err("archive extracted-size limit exceeded".into()); }
        if let Some(parent) = out.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        let mut output = fs::File::create(&out).map_err(|e| e.to_string())?;
        let copied = io::copy(&mut entry.take(MAX_FILE_BYTES + 1), &mut output).map_err(|e| e.to_string())?;
        if copied > MAX_FILE_BYTES { return Err(format!("archive file too large: {}", path.display())); }
        output.sync_all().map_err(|e| e.to_string())?;
    }
    Ok(())
}
