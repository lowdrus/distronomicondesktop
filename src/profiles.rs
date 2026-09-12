use crate::{
    atomic_file,
    config::{Config, Language},
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub channel: String,
    pub architecture: String,
    pub backup_paths: String,
    pub verify_authenticode: bool,
    pub max_disk_mb: u64,
    pub retention_days: u64,
    pub allow_downgrade: bool,
    pub notifications: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            app_name: String::new(),
            repo: String::new(),
            asset_pattern: r"(?i).*\.(zip|exe|tgz|tbz2|txz)$|.*\.tar\.(gz|bz2|xz|zst)$".into(),
            checksum_pattern: r"(?i)^(SHA256SUMS|checksums?(\.txt)?|.*sha256.*)$".into(),
            install_root: String::new(),
            state_root: String::new(),
            github_host: "https://api.github.com".into(),
            allow_prerelease: false,
            skip_verification: false,
            retain: 3,
            restart_command: String::new(),
            health_check_command: String::new(),
            pinned_version: String::new(),
            auto_mode: "check".into(),
            interval_minutes: 60,
            channel: "stable".into(),
            architecture: "auto".into(),
            backup_paths: String::new(),
            verify_authenticode: true,
            max_disk_mb: 0,
            retention_days: 0,
            allow_downgrade: false,
            notifications: true,
        }
    }
}

impl Profile {
    pub fn effective_channel(&self) -> String {
        if self.allow_prerelease && (self.channel.is_empty() || self.channel == "stable") {
            "nightly".into()
        } else if self.channel.is_empty() {
            "stable".into()
        } else {
            self.channel.clone()
        }
    }

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
            channel: self.effective_channel(),
            architecture: if self.architecture.is_empty() {
                "auto".into()
            } else {
                self.architecture.clone()
            },
            backup_paths: self.backup_paths.clone(),
            verify_authenticode: self.verify_authenticode,
            max_disk_mb: self.max_disk_mb,
            retention_days: self.retention_days,
            allow_downgrade: self.allow_downgrade,
            notifications: self.notifications,
        }
    }
}

pub fn default_path(base: &Path) -> PathBuf {
    base.join(".distronomicon").join("profiles.json")
}
pub fn export_path(base: &Path) -> PathBuf {
    base.join("DistronomiconProfiles.json")
}

pub fn load(path: &Path) -> io::Result<Vec<Profile>> {
    if path.exists() {
        match read_profiles(path) {
            Ok(profiles) => return Ok(profiles),
            Err(primary_error) => {
                let backup = atomic_file::backup_path(path);
                if backup.exists() {
                    return read_profiles(&backup);
                }
                return Err(primary_error);
            }
        }
    }
    let backup = atomic_file::backup_path(path);
    if backup.exists() {
        return read_profiles(&backup);
    }
    Ok(Vec::new())
}

fn read_profiles(path: &Path) -> io::Result<Vec<Profile>> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn save(path: &Path, profiles: &[Profile]) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(profiles)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_file::write(path, &bytes)
}

pub fn export_profiles(base: &Path, profiles: &[Profile]) -> io::Result<PathBuf> {
    let path = export_path(base);
    save(&path, profiles)?;
    Ok(path)
}

pub fn import_profiles(base: &Path) -> io::Result<Vec<Profile>> {
    read_profiles(&export_path(base))
}

pub fn upsert(profiles: &mut Vec<Profile>, profile: Profile) {
    if let Some(existing) = profiles
        .iter_mut()
        .find(|p| p.name.eq_ignore_ascii_case(&profile.name))
    {
        *existing = profile;
    } else {
        profiles.push(profile);
        profiles.sort_by_key(|p| p.name.to_ascii_lowercase());
    }
}

pub fn remove(profiles: &mut Vec<Profile>, name: &str) {
    profiles.retain(|p| !p.name.eq_ignore_ascii_case(name));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_legacy_prerelease_profile_to_nightly() {
        let profile = Profile {
            allow_prerelease: true,
            channel: "stable".into(),
            ..Profile::default()
        };
        assert_eq!(profile.effective_channel(), "nightly");
    }

    #[test]
    fn keeps_explicit_beta_channel() {
        let profile = Profile {
            allow_prerelease: true,
            channel: "beta".into(),
            ..Profile::default()
        };
        assert_eq!(profile.effective_channel(), "beta");
    }
}
