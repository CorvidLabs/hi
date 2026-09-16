//! One writer at a time, per repository.
//!
//! Capture is read-modify-write: load every `hi/*.md`, decide where the new
//! criterion goes, write the whole file back. `doc::write_atomically` makes the
//! write itself atomic, which stops a torn file, and does nothing at all about
//! two processes reading the same original and each writing their own version
//! over the other. Eight concurrent captures used to land two.
//!
//! That stopped being theoretical the day agents started bulk-capturing into a
//! repository, so the whole read-modify-write is taken under a lock file. No
//! dependency: `create_new` is the exclusive-create every platform already has
//! (hi: FILE-19).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Result, bail};

/// How long to keep trying before giving up on another writer.
const PATIENCE: Duration = Duration::from_secs(5);
/// A lock older than this belonged to a process that died holding it.
const STALE: Duration = Duration::from_secs(60);

/// Held for as long as the caller may write. Releases on drop, including when
/// the caller returns an error, so a refusal never leaves the lock behind.
pub struct Guard {
    path: PathBuf,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Take the write lock for `hi_dir`, waiting for another writer to finish.
pub fn acquire(hi_dir: &Path) -> Result<Guard> {
    let path = hi_dir.join(".hi.lock");
    let started = SystemTime::now();

    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => return Ok(Guard { path }),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            // A hi/ we cannot write to is a problem the caller will hit anyway,
            // and it will say something more useful than "could not lock".
            Err(_) => return Ok(Guard { path }),
        }

        if stale(&path) {
            let _ = fs::remove_file(&path);
            continue;
        }

        if started.elapsed().unwrap_or_default() > PATIENCE {
            bail!(
                "another hi is writing to {} and has not finished.\n\
                 hint:  if nothing else is running, delete {}",
                hi_dir.display(),
                path.display()
            )
        }

        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A lock whose holder died leaves a file nobody will ever remove.
fn stale(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .and_then(|at| at.elapsed().map_err(std::io::Error::other))
        .is_ok_and(|age| age > STALE)
}
