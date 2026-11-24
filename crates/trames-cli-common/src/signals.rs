// SPDX-License-Identifier: GPL-3.0-or-later
//! A process signal lease and stable self-pipe; no signal-time Rust references.
use std::{
    io::{self, Read},
    os::{
        fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
        unix::net::UnixStream,
    },
    sync::{
        atomic::{AtomicI32, AtomicU64, Ordering},
        Mutex, MutexGuard, OnceLock,
    },
};
static OWNER: Mutex<()> = Mutex::new(());
static PIPE: OnceLock<(UnixStream, UnixStream)> = OnceLock::new();
static WRITE: AtomicI32 = AtomicI32::new(-1);
static PENDING: AtomicU64 = AtomicU64::new(0);
extern "C" fn receive(signal: libc::c_int) {
    // write() may report EAGAIN; preserve interrupted libc callers' errno.
    #[cfg(target_os = "linux")]
    let errno = unsafe { libc::__errno_location() };
    #[cfg(target_vendor = "apple")]
    let errno = unsafe { libc::__error() };
    #[cfg(not(any(target_os = "linux", target_vendor = "apple")))]
    let errno = std::ptr::null_mut::<i32>();
    let saved = if errno.is_null() {
        0
    } else {
        unsafe { errno.read() }
    };
    PENDING.fetch_or(1u64 << signal, Ordering::Relaxed);
    let fd = WRITE.load(Ordering::Relaxed);
    if fd >= 0 {
        unsafe { libc::write(fd, b"s".as_ptr().cast(), 1) };
    }
    if !errno.is_null() {
        unsafe { errno.write(saved) }
    }
}
/// Only interactive frontend invocations install handlers. Programmatic exec
/// clients preserve their embedding's signals and can supply explicit deadlines.
pub struct Signals {
    _owner: MutexGuard<'static, ()>,
    saved: Vec<(i32, libc::sigaction)>,
}
impl Signals {
    pub fn install() -> io::Result<Self> {
        let owner = OWNER.try_lock().map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "another foreground CLI owns process signals",
            )
        })?;
        if PIPE.get().is_none() {
            let (read, write) = UnixStream::pair()?;
            read.set_nonblocking(true)?;
            write.set_nonblocking(true)?;
            let _ = PIPE.set((read, write));
        }
        WRITE.store(PIPE.get().unwrap().1.as_raw_fd(), Ordering::Relaxed);
        let mut guard = Self {
            _owner: owner,
            saved: vec![],
        };
        guard.drain();
        PENDING.store(0, Ordering::Relaxed);
        for signal in [
            libc::SIGHUP,
            libc::SIGTERM,
            libc::SIGINT,
            libc::SIGWINCH,
            libc::SIGPIPE,
        ] {
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            let mut old = unsafe { std::mem::zeroed() };
            action.sa_sigaction = if signal == libc::SIGPIPE {
                libc::SIG_IGN
            } else {
                receive as *const () as usize
            };
            unsafe { libc::sigemptyset(&mut action.sa_mask) };
            if unsafe { libc::sigaction(signal, &action, &mut old) } != 0 {
                return Err(io::Error::last_os_error());
            }
            guard.saved.push((signal, old));
        }
        Ok(guard)
    }
    pub fn drain(&self) {
        let mut bytes = [0; 128];
        while (&PIPE.get().unwrap().0)
            .read(&mut bytes)
            .is_ok_and(|n| n != 0)
        {}
    }
    pub fn take_signal(&self) -> Option<i32> {
        for signal in [libc::SIGHUP, libc::SIGTERM, libc::SIGINT] {
            let mask = 1u64 << signal;
            if PENDING.fetch_and(!mask, Ordering::Relaxed) & mask != 0 {
                return Some(signal);
            }
        }
        None
    }
    pub fn stopping(&self) -> bool {
        PENDING.load(Ordering::Relaxed)
            & ((1 << libc::SIGHUP) | (1 << libc::SIGINT) | (1 << libc::SIGTERM))
            != 0
    }
}
impl AsRawFd for Signals {
    fn as_raw_fd(&self) -> RawFd {
        PIPE.get().unwrap().0.as_raw_fd()
    }
}
impl Drop for Signals {
    fn drop(&mut self) {
        for (signal, saved) in self.saved.iter().rev() {
            unsafe { libc::sigaction(*signal, saved, std::ptr::null_mut()) };
        }
    }
}

impl AsFd for Signals {
    fn as_fd(&self) -> BorrowedFd<'_> {
        PIPE.get().unwrap().0.as_fd()
    }
}
