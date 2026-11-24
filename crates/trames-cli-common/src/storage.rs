// SPDX-License-Identifier: GPL-3.0-or-later
use std::{io, process::Command};
pub fn volume_space(path: &str) -> io::Result<(u64, u64)> {
    let path = std::fs::canonicalize(path)?;
    let output = Command::new("df")
        .arg("-Pk")
        .arg(path)
        .env("LC_ALL", "C")
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("cannot query storage volume"));
    }
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    let fields: Vec<_> = text
        .lines()
        .last()
        .unwrap_or("")
        .split_whitespace()
        .collect();
    let index = fields
        .iter()
        .position(|s| s.ends_with('%'))
        .filter(|&i| i >= 3)
        .ok_or_else(|| io::Error::other("invalid df capacity response"))?;
    let number = |s: &str| {
        s.parse::<u64>()
            .ok()
            .and_then(|n| n.checked_mul(1024))
            .ok_or_else(|| io::Error::other("invalid volume size"))
    };
    Ok((number(fields[index - 3])?, number(fields[index - 1])?))
}
pub fn storage_budget(value: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let (number, scale) = [
        ("TiB", 1u64 << 40),
        ("GiB", 1 << 30),
        ("MiB", 1 << 20),
        ("KiB", 1 << 10),
    ]
    .into_iter()
    .find_map(|(suffix, scale)| value.strip_suffix(suffix).map(|n| (n, scale)))
    .unwrap_or((value, 1));
    let bytes = number
        .parse::<u64>()?
        .checked_mul(scale)
        .filter(|&n| n != 0 && n <= i64::MAX as u64)
        .ok_or("invalid storage budget")?;
    Ok(bytes)
}
