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
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn load(path: &Path) -> io::Result<Option<State>> {
    if !path.exists() {
        let backup = backup_path(path);
        if !backup.exists() {
            return Ok(None);
        }
        return read_state(&backup).map(Some);
    }

    match read_state(path) {
        Ok(state) => Ok(Some(state)),
        Err(primary_error) => {
            let backup = backup_path(path);
            if backup.exists() {
                read_state(&backup).map(Some)
            } else {
                Err(primary_error)
            }
        }
    }
}

pub fn save_atomic(path: &Path, state: &State) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_file::write(path, &bytes)
}

fn read_state(path: &Path) -> io::Result<State> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn backup_path(path: &Path) -> std::path::PathBuf {
    path.with_extension("json.bak")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_path_is_predictable() {
        assert!(backup_path(Path::new("state.json")).ends_with("state.json.bak"));
    }
}
