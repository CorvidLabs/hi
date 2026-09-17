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
//!
//! Two things here are not obvious, and both were bugs first
//! (DECISIONS.md §31):
//!
//! **A guard is only ever a lock that was really taken.** The lock lives inside
//! `hi/`, so before the directory exists there is nothing to create it in, and
//! the first capture in a repository used to be handed a guard it never held.
//! Thirty-two of those ran unlocked and nine were lost. `acquire` creates `hi/`
//! first and fails closed on anything else, so a `Guard` cannot exist without
//! the file it names.
//!
//! **A lock is broken because nobody is holding it, never because it is old.**
//! The holder refreshes the file four times a second from a thread of its own,
//! and a waiter breaks the lock only after watching that mtime stand still for
//! `ABANDONED` of the waiter's own elapsed time. Age proved a process was slow,
//! not that it was dead, and a bulk capture is slow.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result, bail};

/// The lock file, inside the `hi/` directory it protects.
const LOCK: &str = ".hi.lock";
/// How long to keep trying before giving up on another writer. Long enough to
/// outlast `ABANDONED`, or a lock whose holder died could never be recovered,
/// and long enough for a bulk capture's queue: two hundred captures serialized
/// through this lock is a few seconds of waiting for the last one.
const PATIENCE: Duration = Duration::from_secs(30);
/// How often the holder proves it is still there.
const HEARTBEAT: Duration = Duration::from_millis(250);
/// A lock nobody has refreshed for this long, measured on the waiter's own
/// clock, is nobody's.
const ABANDONED: Duration = Duration::from_secs(5);
/// How often a waiter looks again.
const RETRY: Duration = Duration::from_millis(20);

/// Held for as long as the caller may write. Releases on drop, including when
/// the caller returns an error, so a refusal never leaves the lock behind.
///
/// There is no public constructor, and the private one takes the file that was
/// exclusively created. A guard that does not hold the lock cannot be built, so
/// releasing one can never remove somebody else's (DECISIONS.md §31).
pub struct Guard {
    path: PathBuf,
    dir: PathBuf,
    /// True when taking the lock had to create `hi/` itself.
    created_dir: bool,
    /// Dropping this ends the heartbeat.
    stop: Option<Sender<()>>,
    beat: Option<JoinHandle<()>>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Stop saying we are alive before the file goes, so the heartbeat can
        // never write a lock back after it has been released.
        drop(self.stop.take());
        if let Some(beat) = self.beat.take() {
            let _ = beat.join();
        }
        let _ = fs::remove_file(&self.path);
        if self.created_dir {
            // Only ever an empty one: `remove_dir` refuses a directory with
            // anything in it, so a capture that wrote a file keeps its `hi/`
            // and a refusal leaves nothing at all behind (hi: CAPTURE-5).
            let _ = fs::remove_dir(&self.dir);
        }
    }
}

impl Guard {
    /// Take ownership of a lock file this process exclusively created.
    fn held(mut file: fs::File, path: PathBuf, dir: PathBuf, created_dir: bool) -> Guard {
        // The pid is for the person deciding whether the holder is still alive,
        // and it is what the heartbeat rewrites. The same bytes every time, so
        // the file never grows and never needs truncating.
        let _ = writeln!(file, "{}", std::process::id());
        drop(file);

        let (stop, wake) = mpsc::channel::<()>();
        let beating = path.clone();
        let beat = thread::spawn(move || {
            // Every wake refreshes the mtime, which is the only evidence a
            // waiter has that somebody is still working. The channel ends this:
            // `Disconnected` when the guard drops its sender, which is a moment
            // before the file goes.
            while matches!(wake.recv_timeout(HEARTBEAT), Err(RecvTimeoutError::Timeout)) {
                // Never `create`: once the guard has released, there is nothing
                // to refresh, and recreating the file would lock out every
                // writer for `ABANDONED` with nobody holding anything.
                let refreshed = fs::OpenOptions::new()
                    .write(true)
                    .open(&beating)
                    .and_then(|mut file| writeln!(file, "{}", std::process::id()));
                if refreshed.is_err() {
                    return;
                }
            }
        });

        Guard {
            path,
            dir,
            created_dir,
            stop: Some(stop),
            beat: Some(beat),
        }
    }
}

/// Take the write lock for `hi_dir`, waiting for another writer to finish.
pub fn acquire(hi_dir: &Path) -> Result<Guard> {
    acquire_within(hi_dir, PATIENCE, ABANDONED)
}

/// `acquire`, with the two waits named, so a test can wait in milliseconds.
fn acquire_within(hi_dir: &Path, patience: Duration, abandoned: Duration) -> Result<Guard> {
    let path = hi_dir.join(LOCK);

    // The lock lives inside `hi/`, so the first capture in a repository has to
    // make the directory before it can take one. Doing it here rather than in
    // capture is the whole point: opening a lock file under a directory that
    // does not exist fails, and a failure used to be handed back as a guard
    // (hi: FILE-19, DECISIONS.md §31). `create_dir_all` succeeds when another
    // capture won the same race, so the bootstrap is safe by construction.
    let mut created_dir = !hi_dir.is_dir();
    fs::create_dir_all(hi_dir).with_context(|| format!("creating {}", hi_dir.display()))?;

    let waited = Instant::now();
    // The mtime last seen on somebody else's lock, and when we first saw it.
    let mut watched: Option<(SystemTime, Instant)> = None;

    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok(Guard::held(file, path, hi_dir.to_path_buf(), created_dir)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // `hi/` went away under us: another writer refused and took the
                // empty directory it had made with it. Put it back.
                fs::create_dir_all(hi_dir)
                    .with_context(|| format!("creating {}", hi_dir.display()))?;
                created_dir = true;
                watched = None;
            }
            // Fail closed. A guard that was not acquired is not a lock, and
            // handing one back is how every writer comes to believe it is alone.
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "could not take the write lock {}.\n\
                         hint:  hi has to be able to write inside {}",
                        path.display(),
                        hi_dir.display()
                    )
                });
            }
        }

        // A live holder refreshes its lock four times a second, so a lock whose
        // mtime has not moved while this process watched it for `abandoned` is
        // held by nobody. Only our own elapsed time is used, never the
        // difference between this clock and the file's: age alone never
        // established that a process had died, and a slow capture is not a dead
        // one (DECISIONS.md §31).
        match fs::metadata(&path).and_then(|meta| meta.modified()) {
            Ok(mtime) => match watched {
                Some((seen, _)) if seen != mtime => watched = Some((mtime, Instant::now())),
                Some((seen, since)) if since.elapsed() >= abandoned => {
                    // Read it again first. Another waiter may have broken this
                    // same lock and taken one of its own a moment ago, and
                    // removing that one would hand the directory to two writers
                    // at once. A fresh lock has a fresh mtime.
                    if fs::metadata(&path).and_then(|meta| meta.modified()).ok() == Some(seen) {
                        let _ = fs::remove_file(&path);
                    }
                    watched = None;
                    continue;
                }
                Some(_) => {}
                None => watched = Some((mtime, Instant::now())),
            },
            // It was there a moment ago and is not now: whoever held it has
            // finished, and the next turn of the loop takes it.
            Err(_) => watched = None,
        }

        if waited.elapsed() > patience {
            bail!(
                "another hi is writing to {} and has not finished.\n\
                 hint:  if nothing else is running, delete {}",
                hi_dir.display(),
                path.display()
            )
        }

        thread::sleep(RETRY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        // Never a fixed path: two test processes sharing one scratch directory
        // is why the suite used to flake.
        let root = std::env::temp_dir().join(format!("hi-lock-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn the_first_writer_in_a_repository_takes_a_real_lock() {
        // The state every first capture starts in: no `hi/` at all. This used to
        // return a guard that held nothing, so thirty-two concurrent captures
        // all believed they were alone (hi: FILE-19, DECISIONS.md §31).
        let root = scratch("bootstrap");
        fs::create_dir_all(&root).unwrap();
        let hi = root.join("hi");

        let held = acquire(&hi).expect("a lock before hi/ exists");
        assert!(
            hi.join(LOCK).is_file(),
            "a guard must be a lock that was really taken, bootstrap included"
        );

        // And it is exclusive from the moment it is granted.
        let second = acquire_within(&hi, Duration::from_millis(80), Duration::from_secs(60));
        assert!(second.is_err(), "a held lock cannot be taken twice");
        assert!(
            hi.join(LOCK).is_file(),
            "and the writer that failed to take it left the holder's lock alone"
        );

        drop(held);
        assert!(!hi.join(LOCK).exists(), "the holder releases it");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_lock_that_cannot_be_taken_is_refused_rather_than_pretended() {
        // Only the unix bit is portable enough to make a directory unwritable.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let root = scratch("readonly");
            let hi = root.join("hi");
            fs::create_dir_all(&hi).unwrap();
            fs::set_permissions(&hi, fs::Permissions::from_mode(0o555)).unwrap();

            let refused = acquire(&hi);

            fs::set_permissions(&hi, fs::Permissions::from_mode(0o755)).unwrap();
            assert!(
                refused.is_err(),
                "a lock hi could not create must never come back as a guard"
            );
            let _ = fs::remove_dir_all(&root);
        }
    }

    #[test]
    fn a_refusal_leaves_no_hi_directory_behind() {
        // The lock has to make `hi/` to live in it, and a capture that refuses
        // must still write nothing at all (hi: CAPTURE-5).
        let root = scratch("empty");
        fs::create_dir_all(&root).unwrap();
        let hi = root.join("hi");

        drop(acquire(&hi).unwrap());

        assert!(!hi.exists(), "an empty hi/ the lock made is taken back");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_lock_that_is_being_refreshed_is_never_broken() {
        // The old rule broke any lock older than sixty seconds, which says a
        // process is slow and not that it is dead. This waiter watches for well
        // past a heartbeat and must still refuse.
        let root = scratch("alive");
        let hi = root.join("hi");
        fs::create_dir_all(&hi).unwrap();

        let held = acquire(&hi).unwrap();
        // Well past a heartbeat, and past a whole second, so this holds even on
        // a filesystem that records mtimes to the second.
        let refused = acquire_within(
            &hi,
            Duration::from_millis(1500),
            Duration::from_millis(1200),
        );

        assert!(refused.is_err(), "a lock somebody is holding stays theirs");
        assert!(hi.join(LOCK).is_file(), "and is still on disk");
        drop(held);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_lock_nobody_is_refreshing_is_broken_and_taken() {
        // The other half: a hi that died holding the lock must not wedge the
        // repository until somebody deletes a file by hand.
        let root = scratch("dead");
        let hi = root.join("hi");
        fs::create_dir_all(&hi).unwrap();
        fs::write(hi.join(LOCK), "999999\n").unwrap();

        let taken = acquire_within(&hi, Duration::from_secs(2), Duration::from_millis(100));

        assert!(taken.is_ok(), "a lock nobody refreshes is nobody's");
        drop(taken);
        let _ = fs::remove_dir_all(&root);
    }
}
