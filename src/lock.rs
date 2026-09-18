//! One writer at a time, per repository.
//!
//! Capture is read-modify-write: load every `hi/*.md`, decide where the new
//! criterion goes, write the whole file back. `doc::write_atomically` makes the
//! write itself atomic, which stops a torn file, and does nothing at all about
//! two processes reading the same original and each writing their own version
//! over the other. Eight concurrent captures used to land two.
//!
//! That stopped being theoretical the day agents started bulk-capturing into a
//! repository, so the whole read-modify-write is taken under a lock
//! (hi: FILE-19).
//!
//! **The lock is the kernel's, not the file's** (DECISIONS.md §34). Two earlier
//! designs decided for themselves when somebody else's lock had expired — first
//! by age, then by a heartbeat the holder wrote and a waiter watched. Both are
//! guesses, and a guess that is wrong hands the repository to two writers. The
//! second one was also racy in a way no amount of extra checking fixes: between
//! deciding a lock was abandoned and removing it, the holder can change, and
//! the removal is already approved. A reviewer forced exactly that interleaving
//! against a live, heartbeating holder and lost a criterion whose capture had
//! reported success.
//!
//! So hi holds `flock(2)` on unix and `LockFileEx` on Windows, and never breaks
//! a lock at all. Both are owned by the open file, which means the kernel drops
//! them when the process exits however it exits, so a `hi` that was killed
//! leaves nothing to clean up and the next writer simply takes the lock
//! (hi: FILE-23) — while a `hi` that is merely slow keeps what it took, for as
//! long as it is alive (hi: FILE-24). No dependency: both are three lines of
//! `extern` each, and hi has four dependencies for a reason.
//!
//! Three things here are not obvious, and each was a defect first:
//!
//! **A guard is only ever a lock that was really taken.** The lock lives inside
//! `hi/`, so before the directory exists there is nothing to create it in, and
//! the first capture in a repository used to be handed a guard it never held.
//! Thirty-two of those ran unlocked and nine were lost. `acquire` creates `hi/`
//! first and fails closed on anything else, so a `Guard` cannot exist without
//! the file it names (DECISIONS.md §33).
//!
//! **Holding the kernel's lock on a file is not holding the pathname.** The
//! holder unlinks the lock file when it releases, so a waiter that was already
//! queued on that file can be granted the lock on an inode the name no longer
//! points at, while somebody else creates a fresh file and locks that. On unix
//! `still_at` closes it: after the lock is granted, the file hi holds has to be
//! the file the path names, and if it is not, hi drops it and tries again.
//!
//! **Unlink while holding, never after.** Releasing is `remove_file` and *then*
//! the close, so nobody can take the lock between the two and have their live
//! lock file removed by us. Every removal of `.hi.lock` in this module happens
//! under the lock on the file being removed. Nothing else removes it, ever.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// The lock file, inside the `hi/` directory it protects.
const LOCK: &str = ".hi.lock";
/// How long to keep trying before giving up on another writer. Long enough for
/// a bulk capture's queue: two hundred captures serialized through this lock is
/// a few seconds of waiting for the last one.
const PATIENCE: Duration = Duration::from_secs(30);
/// How often a waiter looks again.
const RETRY: Duration = Duration::from_millis(20);
/// How long an error that is neither "somebody has it" nor "the directory went
/// away" has to keep happening before it counts as a real one. Windows marks a
/// file for deletion rather than removing it, so a waiter whose open lands in
/// the window between one writer releasing and the file actually going is told
/// access is denied. With thirty-two captures queued there are thirty-one of
/// those handoffs, and one of them failed a capture on CI.
const GRACE: Duration = Duration::from_millis(500);

/// Held for as long as the caller may write. Releases on drop, including when
/// the caller returns an error, so a refusal never leaves the lock behind.
///
/// There is no public constructor, and the private one takes the file the
/// kernel granted the lock on. A guard that does not hold the lock cannot be
/// built, so releasing one can never remove somebody else's (DECISIONS.md §33).
pub struct Guard {
    path: PathBuf,
    dir: PathBuf,
    /// True when taking the lock had to create `hi/` itself.
    created_dir: bool,
    /// The open file the kernel's lock belongs to. Dropping it releases the
    /// lock, and so does the process exiting for any reason at all.
    held: Option<fs::File>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Unlink first, close second. While the lock is still held nobody else
        // can be holding this file, so this can only ever remove our own; after
        // the close it could be somebody's live lock (DECISIONS.md §34).
        let _ = fs::remove_file(&self.path);
        drop(self.held.take());
        if self.created_dir {
            // Only ever an empty one: `remove_dir` refuses a directory with
            // anything in it, so a capture that wrote a file keeps its `hi/`
            // and a refusal leaves nothing at all behind (hi: CAPTURE-5).
            let _ = fs::remove_dir(&self.dir);
        }
    }
}

impl Guard {
    /// Take ownership of a file the kernel has granted this process the lock on.
    fn held(mut file: fs::File, path: PathBuf, dir: PathBuf, created_dir: bool) -> Guard {
        // The pid is for the person who finds the file and wants to know who
        // has it. It is not evidence of anything and nothing reads it back: the
        // lock is the kernel's, and a file with a stale pid in it is taken by
        // the next writer without ceremony.
        let _ = file.set_len(0);
        let _ = writeln!(file, "{}", std::process::id());

        Guard {
            path,
            dir,
            created_dir,
            held: Some(file),
        }
    }
}

/// Take the write lock for `hi_dir`, waiting for another writer to finish.
pub fn acquire(hi_dir: &Path) -> Result<Guard> {
    acquire_within(hi_dir, PATIENCE)
}

/// `acquire`, with the wait named, so a test can wait in milliseconds.
fn acquire_within(hi_dir: &Path, patience: Duration) -> Result<Guard> {
    let path = hi_dir.join(LOCK);

    // The lock lives inside `hi/`, so the first capture in a repository has to
    // make the directory before it can take one. Doing it here rather than in
    // capture is the whole point: opening a lock file under a directory that
    // does not exist fails, and a failure used to be handed back as a guard
    // (hi: FILE-19, DECISIONS.md §33). `create_dir_all` succeeds when another
    // capture won the same race, so the bootstrap is safe by construction.
    let mut created_dir = !hi_dir.is_dir();
    fs::create_dir_all(hi_dir).with_context(|| format!("creating {}", hi_dir.display()))?;

    let waited = Instant::now();
    // When an error we are willing to sit out started.
    let mut trouble: Option<Instant> = None;

    loop {
        match os::open_lock(&path) {
            Ok(file) => {
                trouble = None;
                match os::try_hold(&file) {
                    // Granted by the kernel. On unix that can still be a lock
                    // on a file this pathname no longer names, because the
                    // previous holder unlinks on release: hold the name, not
                    // just the inode (DECISIONS.md §34).
                    Ok(true) => match os::still_at(&file, &path) {
                        Ok(true) => {
                            return Ok(Guard::held(file, path, hi_dir.to_path_buf(), created_dir));
                        }
                        // Somebody replaced the file while we queued on it.
                        // Drop the lock we were granted and start over.
                        Ok(false) | Err(_) => drop(file),
                    },
                    Ok(false) => drop(file),
                    Err(err) => {
                        return Err(err).with_context(|| {
                            format!(
                                "could not lock {}.\n\
                                 hint:  hi needs a filesystem that supports locking. If {} is on \
                                 a network share, run hi against a local checkout",
                                path.display(),
                                hi_dir.display()
                            )
                        });
                    }
                }
            }
            // Only the fallback implementation reports this, where creating the
            // file exclusively *is* the lock.
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => trouble = None,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // `hi/` went away under us: another writer refused and took the
                // empty directory it had made with it. Put it back.
                fs::create_dir_all(hi_dir)
                    .with_context(|| format!("creating {}", hi_dir.display()))?;
                created_dir = true;
                trouble = None;
            }
            // Anything else fails closed, because a guard that was not acquired
            // is not a lock and handing one back is how every writer comes to
            // believe it is alone. It fails closed after `GRACE` rather than at
            // once: a release on Windows leaves the file briefly present and
            // undeletable, and a waiter that arrives in that window is told
            // access is denied when what is really happening is a handoff. An
            // error still there half a second later is a real one.
            Err(err) => {
                let since = *trouble.get_or_insert_with(Instant::now);
                if since.elapsed() > GRACE {
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
        }

        if waited.elapsed() > patience {
            bail!("{}", busy(hi_dir, &path))
        }

        std::thread::sleep(RETRY);
    }
}

/// What to say to somebody whose capture waited out another writer.
///
/// Deliberately not "delete this file". Where the kernel owns the lock, the
/// file is not the lock: deleting it while a writer holds it is the one act
/// that can put two writers in a repository at once, and deleting it when
/// nobody holds it achieves nothing hi would not have done itself.
fn busy(hi_dir: &Path, path: &Path) -> String {
    if os::RELEASED_ON_EXIT {
        format!(
            "another hi is writing to {} and has not finished.\n\
             hint:  the lock belongs to a running process, not to {}. hi takes it back by \
             itself the moment that process exits, so wait for it or stop it — deleting \
             the file while it is held is the one thing that lets two writers in",
            hi_dir.display(),
            path.display()
        )
    } else {
        format!(
            "another hi is writing to {} and has not finished.\n\
             hint:  if nothing else is running, delete {}",
            hi_dir.display(),
            path.display()
        )
    }
}

/// The three operations that differ per platform, and nothing else.
///
/// `open_lock` opens (and creates) the lock file, `try_hold` asks the kernel
/// for the exclusive lock without waiting, and `still_at` says whether the file
/// hi now holds is the one the pathname names.
#[cfg(unix)]
mod os {
    use std::ffi::c_int;
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::unix::fs::MetadataExt;
    use std::os::unix::io::AsRawFd;
    use std::path::Path;

    /// The kernel drops a `flock` when the descriptor closes, and closes every
    /// descriptor when the process exits, whatever killed it (hi: FILE-23).
    pub const RELEASED_ON_EXIT: bool = true;

    const LOCK_EX: c_int = 2;
    const LOCK_NB: c_int = 4;

    // Three lines rather than a dependency. `flock` has had these two flag
    // values and this signature on Linux, macOS and the BSDs for decades.
    unsafe extern "C" {
        fn flock(fd: c_int, operation: c_int) -> c_int;
    }

    pub fn open_lock(path: &Path) -> io::Result<File> {
        // `create`, never `create_new`: the file is not the lock, so joining an
        // existing one is right, including one a killed hi left behind.
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
    }

    /// Ask for the exclusive lock, returning false when somebody else has it.
    pub fn try_hold(file: &File) -> io::Result<bool> {
        // SAFETY: the descriptor belongs to `file`, which outlives this call,
        // and `flock` does nothing with it but lock.
        if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } == 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        match err.kind() {
            // Somebody has it, or a signal arrived: either way, look again.
            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => Ok(false),
            // A filesystem that cannot lock is reported rather than pretended
            // away. hi fails closed: there is no third answer that is honest.
            _ => Err(err),
        }
    }

    /// Whether the locked file is still the file `path` names.
    ///
    /// The previous holder unlinks the lock file as it releases, so a waiter
    /// can be granted the lock on an inode that has no name left while another
    /// writer creates a fresh file and locks that. Comparing the open file
    /// against the pathname is what makes the lock a lock on the repository
    /// rather than on an orphan (DECISIONS.md §34).
    pub fn still_at(file: &File, path: &Path) -> io::Result<bool> {
        let held = file.metadata()?;
        let named = std::fs::metadata(path)?;
        Ok(held.dev() == named.dev() && held.ino() == named.ino())
    }
}

#[cfg(windows)]
mod os {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    /// A process's handles are closed by the kernel when it exits, and the byte
    /// range lock goes with them (hi: FILE-23).
    pub const RELEASED_ON_EXIT: bool = true;

    type Handle = *mut core::ffi::c_void;

    const LOCKFILE_FAIL_IMMEDIATELY: u32 = 0x0000_0001;
    const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x0000_0002;
    const ERROR_LOCK_VIOLATION: i32 = 33;

    #[repr(C)]
    struct Overlapped {
        internal: usize,
        internal_high: usize,
        offset: u32,
        offset_high: u32,
        event: Handle,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LockFileEx(
            file: Handle,
            flags: u32,
            reserved: u32,
            bytes_low: u32,
            bytes_high: u32,
            overlapped: *mut Overlapped,
        ) -> i32;
    }

    pub fn open_lock(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
    }

    pub fn try_hold(file: &File) -> io::Result<bool> {
        let mut overlapped = Overlapped {
            internal: 0,
            internal_high: 0,
            offset: 0,
            offset_high: 0,
            event: core::ptr::null_mut(),
        };
        // One byte at offset zero: every hi locks the same range, which is all
        // an exclusive lock needs.
        // SAFETY: the handle belongs to `file`, which outlives this call, and
        // `overlapped` is a live, fully initialized OVERLAPPED for its duration.
        let taken = unsafe {
            LockFileEx(
                file.as_raw_handle() as Handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                1,
                0,
                &mut overlapped,
            )
        };
        if taken != 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(ERROR_LOCK_VIOLATION) {
            return Ok(false);
        }
        Err(err)
    }

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    /// `BY_HANDLE_FILE_INFORMATION`. Volume serial plus the two index words is
    /// the file's identity, the same fact `dev`/`ino` states on unix.
    #[repr(C)]
    struct ByHandleFileInformation {
        attributes: u32,
        creation: FileTime,
        last_access: FileTime,
        last_write: FileTime,
        volume_serial: u32,
        size_high: u32,
        size_low: u32,
        links: u32,
        index_high: u32,
        index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(file: Handle, info: *mut ByHandleFileInformation) -> i32;
    }

    fn identity(file: &File) -> io::Result<(u32, u32, u32)> {
        let mut info = ByHandleFileInformation {
            attributes: 0,
            creation: FileTime { low: 0, high: 0 },
            last_access: FileTime { low: 0, high: 0 },
            last_write: FileTime { low: 0, high: 0 },
            volume_serial: 0,
            size_high: 0,
            size_low: 0,
            links: 0,
            index_high: 0,
            index_low: 0,
        };
        // SAFETY: the handle belongs to `file`, which outlives this call, and
        // `info` is a live, correctly laid out BY_HANDLE_FILE_INFORMATION.
        let ok = unsafe { GetFileInformationByHandle(file.as_raw_handle() as Handle, &mut info) };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((info.volume_serial, info.index_high, info.index_low))
    }

    /// Whether the locked file is still the file `path` names.
    ///
    /// The delete-pending argument says this cannot fail on Windows: a removed
    /// file stays in place until its last handle closes and no `CreateFile` on
    /// that name succeeds meanwhile. That argument was never asserted by a
    /// test, and a frozen promise should not rest on it, so the same identity
    /// comparison unix makes is made here: the volume and file index of the
    /// handle hi holds against those of whatever the path names now. A path
    /// that cannot be opened (gone, or delete-pending) is not this file, and
    /// `acquire` drops the lock and starts over (DECISIONS.md §34).
    pub fn still_at(file: &File, path: &Path) -> io::Result<bool> {
        let named = File::open(path)?;
        Ok(identity(file)? == identity(&named)?)
    }
}

/// Neither unix nor Windows: hi ships for Linux, macOS and Windows, and this
/// arm exists so the crate still builds elsewhere.
///
/// There is no OS lock to ask for, so exclusive creation is the lock, and hi
/// never breaks one. That is safe and it is not self-healing: a `hi` killed
/// while holding it leaves a file somebody has to delete, which is `FILE-23`
/// unmet on a platform hi does not ship for (DECISIONS.md §34).
#[cfg(not(any(unix, windows)))]
mod os {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::path::Path;

    pub const RELEASED_ON_EXIT: bool = false;

    pub fn open_lock(path: &Path) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }

    pub fn try_hold(_file: &File) -> io::Result<bool> {
        Ok(true)
    }

    pub fn still_at(_file: &File, _path: &Path) -> io::Result<bool> {
        Ok(true)
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
        // all believed they were alone (hi: FILE-19, DECISIONS.md §33).
        let root = scratch("bootstrap");
        fs::create_dir_all(&root).unwrap();
        let hi = root.join("hi");

        let held = acquire(&hi).expect("a lock before hi/ exists");
        assert!(
            hi.join(LOCK).is_file(),
            "a guard must be a lock that was really taken, bootstrap included"
        );

        // And it is exclusive from the moment it is granted. Another *process*
        // is what proves that; within one process `flock` is per open file, so
        // this is a second open of the same path, which is what a second hi
        // does.
        let second = acquire_within(&hi, Duration::from_millis(80));
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

            // Root ignores the mode bits, and so does anything holding
            // CAP_DAC_OVERRIDE, so in a root container this test asserted that
            // a lock hi *could* take came back as an error and passed because
            // it never got that far. Four tests in this repository have been
            // caught passing while the thing they guarded was broken; a test
            // that cannot fail is the same failure with nobody to blame.
            //
            // The precondition is probed rather than inferred from the uid,
            // because the uid is a proxy for it and this is the thing itself:
            // if a write into that directory succeeds, the directory is not
            // unwritable, whoever we are and whatever granted it.
            let probe = hi.join(".writable-probe");
            if fs::write(&probe, b"").is_ok() {
                let _ = fs::remove_file(&probe);
                fs::set_permissions(&hi, fs::Permissions::from_mode(0o755)).unwrap();
                let _ = fs::remove_dir_all(&root);
                eprintln!(
                    "skipped: this process can write into a directory it has no write bit on, \
                     which is normal as root, so there is no way to make acquire fail here"
                );
                return;
            }

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
    fn a_lock_a_live_process_holds_is_never_taken_however_long_it_waits() {
        // Both earlier designs eventually took this lock: the first because the
        // file was a minute old, the second because a heartbeat stopped for
        // five seconds. Nothing hi can observe about a file proves its holder
        // is gone, so hi no longer tries: the waiter gives up instead
        // (hi: FILE-24, DECISIONS.md §34).
        let root = scratch("alive");
        let hi = root.join("hi");
        fs::create_dir_all(&hi).unwrap();

        let held = acquire(&hi).unwrap();
        let refused = acquire_within(&hi, Duration::from_millis(1500));

        let complaint = match refused {
            Ok(_) => panic!("a lock somebody is holding stays theirs"),
            Err(err) => format!("{err:#}"),
        };
        assert!(
            !complaint.contains("if nothing else is running, delete"),
            "and hi does not suggest deleting a file a live process is holding: {complaint}"
        );
        assert!(hi.join(LOCK).is_file(), "and the lock is still on disk");
        drop(held);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_lock_file_nobody_holds_is_taken_without_anybody_deleting_it() {
        // The other half: a `hi` that died holding the lock must not wedge the
        // repository until somebody deletes a file by hand. Under `flock` the
        // leftover file is not a lock at all, so this is immediate rather than
        // after a timeout (hi: FILE-23).
        let root = scratch("leftover");
        let hi = root.join("hi");
        fs::create_dir_all(&hi).unwrap();
        fs::write(hi.join(LOCK), "999999\n").unwrap();

        let started = Instant::now();
        let taken = acquire_within(&hi, Duration::from_millis(200));

        assert!(taken.is_ok(), "a lock file nobody holds is nobody's");
        assert!(
            started.elapsed() < Duration::from_millis(200),
            "and it is taken at once, not waited out"
        );
        drop(taken);
        let _ = fs::remove_dir_all(&root);
    }

    /// The interleaving that broke the heartbeat design, forced rather than
    /// raced for.
    ///
    /// A waiter opened the lock file, queued behind the holder, and was still
    /// queued when the holder released — which unlinks the file — and a third
    /// writer created a brand new one and locked that. The kernel then grants
    /// the waiter its lock, on an inode with no name. Nothing about that lock
    /// is wrong; believing it is the repository's lock is (DECISIONS.md §34).
    #[test]
    #[cfg(unix)]
    fn a_lock_granted_on_a_file_that_was_replaced_is_not_the_repository_s_lock() {
        let root = scratch("replaced");
        let hi = root.join("hi");
        fs::create_dir_all(&hi).unwrap();
        let path = hi.join(LOCK);

        // The holder.
        let holder = os::open_lock(&path).unwrap();
        assert!(os::try_hold(&holder).unwrap(), "the holder has it");

        // A waiter that opened the same file and was refused.
        let waiter = os::open_lock(&path).unwrap();
        assert!(!os::try_hold(&waiter).unwrap(), "the waiter is queued");

        // The holder releases: unlink, then close, exactly as `Guard` does.
        fs::remove_file(&path).unwrap();
        drop(holder);

        // A third writer arrives first and takes a brand new lock file.
        let fresh = acquire(&hi).expect("the next writer takes the lock");

        // Now the kernel grants the waiter the lock it queued for.
        assert!(
            os::try_hold(&waiter).unwrap(),
            "the kernel grants the orphaned inode, which is correct and useless"
        );
        assert!(
            !os::still_at(&waiter, &path).unwrap(),
            "and `still_at` is the only thing that stops two writers here"
        );

        drop(fresh);
        let _ = fs::remove_dir_all(&root);
    }

    /// A `hi` killed outright, from a second process, releases the repository.
    ///
    /// The child re-runs this test binary with `HI_LOCK_CHILD` set, takes the
    /// lock, says so, and waits to be killed. Nothing tidies up after it: the
    /// kernel drops the lock because the process is gone (hi: FILE-23).
    #[test]
    fn a_killed_holder_frees_the_repository_with_nothing_to_clean_up() {
        let hi = match std::env::var("HI_LOCK_CHILD") {
            Ok(dir) => {
                let held = acquire(Path::new(&dir)).expect("the child takes the lock");
                fs::write(Path::new(&dir).join("held"), "yes").unwrap();
                std::thread::sleep(Duration::from_secs(120));
                drop(held);
                return;
            }
            Err(_) => {
                let root = scratch("killed");
                let hi = root.join("hi");
                fs::create_dir_all(&hi).unwrap();
                hi
            }
        };

        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lock::tests::a_killed_holder_frees_the_repository_with_nothing_to_clean_up",
            ])
            .env("HI_LOCK_CHILD", &hi)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("re-running this test binary as the holder");

        let held = hi.join("held");
        let deadline = Instant::now() + Duration::from_secs(20);
        while !held.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(held.exists(), "the child said it had the lock");
        assert!(
            acquire_within(&hi, Duration::from_millis(200)).is_err(),
            "and while it lives, nobody else gets it"
        );

        child.kill().unwrap();
        child.wait().unwrap();

        let taken = acquire_within(&hi, Duration::from_secs(5));
        assert!(
            taken.is_ok(),
            "a killed holder leaves nothing anybody has to delete"
        );
        drop(taken);
        let _ = fs::remove_dir_all(hi.parent().unwrap());
    }
}
