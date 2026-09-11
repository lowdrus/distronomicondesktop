use crate::config::{Config, Language};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub name: String,
    pub app_name: String,
    pub repo: String,
    pub asset_pattern: String,
    pub checksum_pattern: String,
    pub install_root: String,
    pub state_root: String,
    pub github_host: String,
    pub allow_prerelease: bool,
    pub skip_verification: bool,
    pub retain: usize,
    pub restart_command: String,
    pub health_check_command: String,
    pub pinned_version: String,
    pub auto_mode: String,
    pub interval_minutes: u32,
}

impl Profile {
    pub fn to_config(&self, language: Language, github_token: String) -> Config {
        Config {
            language,
            app_name: self.app_name.clone(),
            repo: self.repo.clone(),
            asset_pattern: self.asset_pattern.clone(),
            checksum_pattern: self.checksum_pattern.clone(),
            install_root: self.install_root.clone(),
            state_root: self.state_root.clone(),
            github_host: self.github_host.clone(),
            github_token,
            allow_prerelease: self.allow_prerelease,
            skip_verification: self.skip_verification,
            retain: self.retain,
            restart_command: self.restart_command.clone(),
            health_check_command: self.health_check_command.clone(),
            pinned_version: self.pinned_version.clone(),
        }
    }
}

pub fn default_path(base: &Path) -> PathBuf {
    base.join(".distronomicon").join("profiles.json")
}

pub fn load(path: &Path) -> io::Result<Vec<Profile>> {
    if !path.exists() { return Ok(Vec::new()); }
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn save(path: &Path, profiles: &[Profile]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "profiles path has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("json.tmp");
    let backup = path.with_extension("json.bak");
    let bytes = serde_json::to_vec_pretty(profiles).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    {
        let mut file = fs::File::create(&tmp)?;
        use std::io::Write;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    let _ = fs::remove_file(&backup);
    if path.exists() { fs::rename(path, &backup)?; }
    if let Err(error) = fs::rename(&tmp, path) {
        if backup.exists() && !path.exists() { let _ = fs::rename(&backup, path); }
        return Err(error);
    }
    let _ = fs::remove_file(backup);
    Ok(())
}

pub fn upsert(profiles: &mut Vec<Profile>, profile: Profile) {
    if let Some(existing) = profiles.iter_mut().find(|p| p.name.eq_ignore_ascii_case(&profile.name)) {
        *existing = profile;
    } else {
        profiles.push(profile);
        profiles.sort_by_key(|p| p.name.to_ascii_lowercase());
    }
}

pub fn remove(profiles: &mut Vec<Profile>, name: &str) {
    profiles.retain(|p| !p.name.eq_ignore_ascii_case(name));
}
