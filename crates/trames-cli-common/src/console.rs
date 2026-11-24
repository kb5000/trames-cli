// SPDX-License-Identifier: GPL-3.0-or-later
//! Shared terminal attach/list clients. Native signals wake the same CLI thread.
use std::{
    io::{self, Read, Write},
    os::{fd::AsRawFd, unix::net::UnixStream},
    path::Path,
    time::Duration,
};
pub const LIST: &[u8; 8] = b"TRMLIST1";
pub const MAGIC: &[u8; 8] = b"TRAMES02";
fn frame(peer: &mut UnixStream, kind: u8, bytes: &[u8]) -> io::Result<()> {
    peer.write_all(&[kind])?;
    peer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    peer.write_all(bytes)
}
pub fn attach(path: &Path, terminal: u64) -> io::Result<()> {
    let mut peer = UnixStream::connect(path)?;
    peer.set_read_timeout(Some(Duration::from_secs(10)))?;
    peer.set_write_timeout(Some(Duration::from_secs(10)))?;
    peer.write_all(MAGIC)?;
    peer.write_all(&terminal.to_be_bytes())?;
    let mut reply = [0; 17];
    peer.read_exact(&mut reply)?;
    if &reply[..8] != MAGIC || reply[8] != 0 {
        return Err(io::Error::other(
            "terminal unavailable, already attached, or terminal limit reached",
        ));
    }
    let id = u64::from_be_bytes(reply[9..].try_into().unwrap());
    peer.set_read_timeout(None)?;
    let signals = crate::signals::Signals::install()?;
    let _raw = crate::terminal::RawTerminal::enter(true)?;
    eprintln!("Attached terminal {id}. Ctrl+] detaches; reconnect with session attach {id}.\r");
    let mut size = None;
    let mut bytes = [0; 4096];
    loop {
        signals.drain();
        if signals.stopping() {
            return Ok(());
        }
        let now = crate::terminal::window_size();
        if now != size {
            size = now;
            if let Some((r, c)) = size {
                let mut b = [0; 4];
                b[..2].copy_from_slice(&r.to_be_bytes());
                b[2..].copy_from_slice(&c.to_be_bytes());
                frame(&mut peer, 1, &b)?;
            }
        }
        let mut fds = [
            libc::pollfd {
                fd: peer.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: libc::STDIN_FILENO,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: signals.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let n = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as _, -1) };
        if n < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }
        if fds[0].revents != 0 {
            let n = peer.read(&mut bytes)?;
            if n == 0 {
                return Ok(());
            }
            io::stdout().write_all(&bytes[..n])?;
            io::stdout().flush()?;
        }
        if fds[1].revents != 0 {
            let n = io::stdin().read(&mut bytes)?;
            if n == 0 {
                return Ok(());
            }
            if let Some(at) = bytes[..n].iter().position(|b| *b == 0x1d) {
                if at != 0 {
                    frame(&mut peer, 0, &bytes[..at])?;
                }
                frame(&mut peer, 2, &[])?;
                return Ok(());
            }
            frame(&mut peer, 0, &bytes[..n])?;
        }
    }
}
pub fn list(path: &Path, args: &[String]) -> io::Result<()> {
    if args.len() > 1
        || args
            .first()
            .is_some_and(|a| !["--json", "-q", "--quiet"].contains(&a.as_str()))
    {
        return Err(io::Error::other("usage: session ls [--json|-q]"));
    }
    let mut peer = UnixStream::connect(path)?;
    peer.set_read_timeout(Some(Duration::from_secs(5)))?;
    peer.write_all(LIST)?;
    peer.write_all(&0u64.to_be_bytes())?;
    let mut size = [0; 4];
    peer.read_exact(&mut size)?;
    let size = u32::from_be_bytes(size) as usize;
    if size > 65536 {
        return Err(io::Error::other(
            "invalid session list response; restart the environment",
        ));
    }
    let mut bytes = vec![0; size];
    peer.read_exact(&mut bytes)?;
    let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes)?;
    match args.first().map(String::as_str).unwrap_or("table") {
        "--json" => println!("{}", serde_json::to_string_pretty(&rows)?),
        "-q" | "--quiet" => {
            for row in rows {
                println!("{}", row["id"]);
            }
        }
        _ => {
            println!("ID      STATE       ATTACHED");
            for row in rows {
                println!(
                    "{:<8}{:<12}{}",
                    row["id"],
                    row["state"].as_str().unwrap_or("unknown"),
                    row["attached"]
                );
            }
        }
    }
    Ok(())
}
