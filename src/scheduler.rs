use std::{io, path::Path, process::Command};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn task_name(profile: &str) -> String {
    format!(
        "DistronomiconDesktop-{}",
        profile
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            })
            .collect::<String>()
    )
}

pub fn install(profile: &str, exe: &Path, mode: &str, interval_minutes: u32) -> io::Result<()> {
    let minutes = interval_minutes.clamp(1, 1440).to_string();
    let task = task_name(profile);
    let action = format!(
        "\"{}\" --scheduled-profile \"{}\" --scheduled-mode {}",
        exe.display(),
        profile.replace('"', ""),
        mode
    );
    let mut cmd = Command::new("schtasks.exe");
    cmd.args([
        "/Create", "/F", "/SC", "MINUTE", "/MO", &minutes, "/TN", &task, "/TR", &action,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

pub fn uninstall(profile: &str) -> io::Result<()> {
    let task = task_name(profile);
    let mut cmd = Command::new("schtasks.exe");
    cmd.args(["/Delete", "/F", "/TN", &task]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}
