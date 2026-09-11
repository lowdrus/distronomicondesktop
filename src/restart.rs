use std::io;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn execute(command: &str) -> io::Result<()> {
    if command.trim().is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/C", command]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let mut details = format!("post-update command failed with status {}", output.status);
        if !stdout.is_empty() {
            details.push_str(&format!("; stdout: {stdout}"));
        }
        if !stderr.is_empty() {
            details.push_str(&format!("; stderr: {stderr}"));
        }
        Err(io::Error::other(details))
    }
}
