// SPDX-License-Identifier: GPL-3.0-or-later
use std::{
    io,
    os::fd::{AsFd, AsRawFd, OwnedFd},
};
/// The duplicated descriptor retains the actual terminal even if stdin is reused.
pub struct RawTerminal(Option<(OwnedFd, libc::termios)>);
impl RawTerminal {
    pub fn enter(enabled: bool) -> io::Result<Self> {
        Self::from_fd(io::stdin(), enabled)
    }
    pub fn from_fd(fd: impl AsFd, enabled: bool) -> io::Result<Self> {
        if !enabled || unsafe { libc::isatty(fd.as_fd().as_raw_fd()) } != 1 {
            return Ok(Self(None));
        }
        let fd = fd.as_fd().try_clone_to_owned()?;
        let mut saved = std::mem::MaybeUninit::uninit();
        if unsafe { libc::tcgetattr(fd.as_raw_fd(), saved.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let saved = unsafe { saved.assume_init() };
        let mut raw = saved;
        unsafe { libc::cfmakeraw(&mut raw) };
        if unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(Some((fd, saved))))
    }
}
impl Drop for RawTerminal {
    fn drop(&mut self) {
        if let Some((fd, saved)) = &self.0 {
            unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, saved) };
        }
    }
}
pub fn window_size() -> Option<(u16, u16)> {
    let mut size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    (unsafe { libc::ioctl(libc::STDIN_FILENO, libc::TIOCGWINSZ, &mut size) } == 0)
        .then_some((size.ws_row, size.ws_col))
        .filter(|&(r, c)| r != 0 && c != 0)
}
