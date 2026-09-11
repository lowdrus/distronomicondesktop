use std::{fs, io, path::Path};

pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
    fs::create_dir_all(parent)?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("data");
    let tmp = parent.join(format!(".{name}.tmp"));
    let backup = parent.join(format!(".{name}.bak"));
    {
        use std::io::Write;
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    let _ = fs::remove_file(&backup);
    if path.exists() { fs::rename(path, &backup)?; }
    if let Err(error) = fs::rename(&tmp, path) {
        if backup.exists() && !path.exists() { let _ = fs::rename(&backup, path); }
        return Err(error);
    }
    let _ = fs::remove_file(&backup);
    if let Ok(dir) = fs::File::open(parent) { let _ = dir.sync_all(); }
    Ok(())
}
