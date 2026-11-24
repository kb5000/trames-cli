// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(unix)]
#[test]
fn process_signal_lease_and_self_pipe_are_isolated_from_test_runner() {
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "signal_child", "--ignored"])
        .status()
        .unwrap();
    assert!(result.success());
}
#[test]
#[ignore = "subprocess-only signal fixture"]
fn signal_child() {
    use std::os::fd::AsRawFd;
    use trames_cli_common::signals::Signals;
    let signals = Signals::install().unwrap();
    assert!(Signals::install().is_err());
    signals.drain();
    assert_eq!(signals.take_signal(), None);
    assert_eq!(unsafe { libc::raise(libc::SIGWINCH) }, 0);
    let mut fd = libc::pollfd {
        fd: signals.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert_eq!(unsafe { libc::poll(&mut fd, 1, 1000) }, 1);
    signals.drain();
    assert!(!signals.stopping());
    assert_eq!(unsafe { libc::raise(libc::SIGINT) }, 0);
    assert!(signals.stopping());
    assert_eq!(signals.take_signal(), Some(libc::SIGINT));
    assert_eq!(signals.take_signal(), None);
    drop(signals);
    drop(Signals::install().unwrap());
}
