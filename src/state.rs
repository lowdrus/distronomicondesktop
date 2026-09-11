use crate::atomic_file;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

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
    let state = serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(state))
}

pub fn save_atomic(path: &Path, state: &State) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_file::write(path, &bytes)
}
