use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp_unix: u64,
    pub action: String,
    pub from_version: String,
    pub to_version: String,
    pub asset: String,
    pub sha256: String,
    pub result: String,
}

impl HistoryEntry {
    pub fn new(
        action: &str,
        from_version: &str,
        to_version: &str,
        asset: &str,
        sha256: &str,
        result: &str,
    ) -> Self {
        Self {
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            action: action.into(),
            from_version: from_version.into(),
            to_version: to_version.into(),
            asset: asset.into(),
            sha256: sha256.into(),
            result: result.into(),
        }
    }
}

pub fn load(path: &Path) -> io::Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn append(path: &Path, entry: HistoryEntry) -> io::Result<()> {
    let mut items = load(path).unwrap_or_default();
    items.push(entry);
    if items.len() > 500 {
        items.drain(0..items.len() - 500);
    }
    save(path, &items)
}

fn save(path: &Path, items: &[HistoryEntry]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "history path has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    let bytes = serde_json::to_vec_pretty(items)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    {
        use std::io::Write;
        let mut file = fs::File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    let _ = fs::remove_file(&backup);
    if path.exists() {
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&tmp, path) {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(error);
    }
    let _ = fs::remove_file(backup);
    Ok(())
}

pub fn format_recent(path: &Path, limit: usize) -> io::Result<String> {
    let mut items = load(path)?;
    items.reverse();
    let mut out = String::new();
    for item in items.into_iter().take(limit) {
        out.push_str(&format!(
            "{} | {} | {} -> {} | {}\n",
            item.timestamp_unix, item.action, item.from_version, item.to_version, item.result
        ));
    }
    Ok(out)
}
