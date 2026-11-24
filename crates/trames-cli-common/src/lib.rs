// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared host CLI, independent of runtime, CPU backend and GUI initialization.
#[cfg(unix)]
pub mod arguments;
#[cfg(unix)]
pub mod console;
#[cfg(unix)]
pub mod exec;
#[cfg(unix)]
pub mod signals;
pub mod storage;
#[cfg(unix)]
pub mod terminal;
