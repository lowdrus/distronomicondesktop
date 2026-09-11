use fs2::FileExt;
use std::{fs::{self, File, OpenOptions}, io, path::{Path, PathBuf}, thread, time::{Duration, Instant}};

pub struct LockGuard {
    file: File,
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire(path: &Path, timeout: Duration) -> io::Result<LockGuard> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)?;
    let start = Instant::now();
    let mut delay = Duration::from_millis(100);

    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(LockGuard { file, path: path.to_path_buf() }),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if start.elapsed() >= timeout {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock, "update lock is busy"));
                }
                thread::sleep(delay);
                delay = (delay * 2).min(Duration::from_secs(1));
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn force_unlock(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
