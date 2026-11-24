// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(unix)]
use std::{
    io::{Read, Write},
    os::{
        fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
        unix::net::{UnixListener, UnixStream},
    },
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use trames_cli_common::{
    arguments,
    exec::{self, Request},
    terminal::RawTerminal,
};
fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
#[test]
fn literal_arguments_cannot_become_host_shell_syntax_or_environment_options() {
    let (selected, args) = arguments::selection(strings(&[
        "-e",
        "work",
        "exec",
        "--setenv",
        "X=a b",
        "--",
        "printf",
        "%s",
        "-e",
        "$(touch x)",
    ]))
    .unwrap();
    assert_eq!(selected.as_deref(), Some("work"));
    let request = arguments::parse_exec(&args[1..], false).unwrap();
    assert_eq!(request.argv, strings(&["printf", "%s", "-e", "$(touch x)"]));
    assert_eq!(request.environment, strings(&["X=a b"]));
    assert!(arguments::parse_exec(&strings(&["--cwd", "/a\0b", "--", "true"]), false).is_err());
    let (_, args) = arguments::selection(strings(&["--", "-e", "literal"])).unwrap();
    assert_eq!(args, strings(&["--", "-e", "literal"]));
}
fn frame(socket: &mut UnixStream, kind: u8, body: &[u8]) {
    socket.write_all(&[kind]).unwrap();
    socket
        .write_all(&(body.len() as u32).to_be_bytes())
        .unwrap();
    socket.write_all(body).unwrap();
}
#[test]
fn existing_exec_wire_preserves_binary_streams_exit_code_and_programmatic_signal_handlers() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let console = std::env::temp_dir().join(format!(
        "trames-cli-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let socket = exec::path(&console);
    let listener = UnixListener::bind(&socket).unwrap();
    let (input, mut writer) = UnixStream::pair().unwrap();
    let payload: Vec<u8> = (0..200000).map(|n| (n % 256) as u8).collect();
    let expected = payload.clone();
    let source = std::thread::spawn(move || writer.write_all(&payload).unwrap());
    let server = std::thread::spawn(move || {
        let (mut peer, _) = listener.accept().unwrap();
        let mut header = [0; 12];
        peer.read_exact(&mut header).unwrap();
        assert_eq!(&header[..8], b"TRMEXEC1");
        let mut bytes = vec![0; u32::from_be_bytes(header[8..].try_into().unwrap()) as usize];
        peer.read_exact(&mut bytes).unwrap();
        let request: Request = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(request.argv, ["literal", "a b"]);
        let mut incoming = vec![];
        loop {
            let mut h = [0; 5];
            peer.read_exact(&mut h).unwrap();
            let mut b = vec![0; u32::from_be_bytes(h[1..].try_into().unwrap()) as usize];
            peer.read_exact(&mut b).unwrap();
            match h[0] {
                0 => incoming.extend(b),
                1 => break,
                _ => panic!("unexpected frame"),
            }
        }
        assert_eq!(incoming, expected);
        frame(&mut peer, 1, &[0, 255, 13, 10]);
        frame(&mut peer, 2, b"separate stderr");
        frame(&mut peer, 3, &(7i32 << 8).to_be_bytes());
    });
    let mut stdout = vec![];
    let mut stderr = vec![];
    let code = exec::run_streams_deadline(
        &console,
        Request {
            argv: strings(&["literal", "a b"]),
            ..Default::default()
        },
        input,
        &mut stdout,
        &mut stderr,
        Some(Instant::now() + Duration::from_secs(5)),
    )
    .unwrap();
    assert_eq!(code, 7);
    assert_eq!(stdout, [0, 255, 13, 10]);
    assert_eq!(stderr, b"separate stderr");
    source.join().unwrap();
    server.join().unwrap();
    std::fs::remove_file(socket).unwrap();
}
#[test]
fn raw_mode_restores_the_retained_terminal_after_original_fd_closes() {
    let (mut master, mut slave) = (-1, -1);
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0
    );
    let _master = unsafe { OwnedFd::from_raw_fd(master) };
    let slave = unsafe { OwnedFd::from_raw_fd(slave) };
    let monitor = slave.try_clone().unwrap();
    let get = || {
        let mut t = std::mem::MaybeUninit::uninit();
        assert_eq!(
            unsafe { libc::tcgetattr(monitor.as_raw_fd(), t.as_mut_ptr()) },
            0
        );
        unsafe { t.assume_init() }
    };
    let before = get();
    let raw = RawTerminal::from_fd(slave.as_fd(), true).unwrap();
    drop(slave);
    assert_eq!(get().c_lflag & libc::ICANON, 0);
    drop(raw);
    assert_eq!(
        get().c_lflag & !libc::PENDIN,
        before.c_lflag & !libc::PENDIN
    );
    assert_eq!(get().c_iflag, before.c_iflag);
}

#[test]
fn foreground_options_preserve_guest_argv_and_never_create_a_host_shell() {
    let command = arguments::Foreground::parse(
        strings(&[
            "--rootfs",
            "/tmp/root",
            "--cpus",
            "4",
            "--timeout-ms",
            "1000",
            "--setenv",
            "X=a b",
            "--",
            "/bin/printf",
            "%s",
            "--cpus",
            "$(id)",
        ]),
        None,
    )
    .unwrap();
    assert_eq!(command.cpus, 4);
    assert_eq!(command.rootfs.as_deref(), Some("/tmp/root"));
    assert_eq!(command.request.environment, ["X=a b"]);
    assert_eq!(
        command.request.argv,
        strings(&["/bin/printf", "%s", "--cpus", "$(id)"])
    );
    assert!(arguments::Foreground::parse(strings(&["--cpus", "0", "--", "true"]), None).is_err());
    assert!(
        arguments::Foreground::parse(strings(&["--timeout-ms", "0", "--", "true"]), None).is_err()
    );
    let shell = arguments::Foreground::parse(
        strings(&["--rootfs", "/tmp/root"]),
        Some(&["/bin/sh", "-l"]),
    )
    .unwrap();
    assert_eq!(shell.request.argv, ["/bin/sh", "-l"]);
}
