use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path, time::{SystemTime, UNIX_EPOCH}};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    pub latest_tag: String,
    pub etag: String,
    pub last_modified: String,
    pub installed_at_unix: u64,
}

pub fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

pub fn load(path: &Path) -> io::Result<Option<State>> {
    if !path.exists() { return Ok(None); }
    let text = fs::read_to_string(path)?;
    let state = serde_json::from_str(&text)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(state))
}

pub fn save_atomic(path: &Path, state: &State) -> io::Result<()> {
    let parent = path.parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "state path has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&tmp, text)?;
    if path.exists() { fs::remove_file(path)?; }
    fs::rename(tmp, path)?;
    Ok(())
}
