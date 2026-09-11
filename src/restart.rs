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

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "restart command failed with status {status}"
        )))
    }
}
