// SPDX-License-Identifier: GPL-3.0-or-later
//! Literal argv and selection. Parsing never opens an environment or CPU.
pub fn selection(
    args: Vec<String>,
) -> Result<(Option<String>, Vec<String>), Box<dyn std::error::Error>> {
    let mut it = args.into_iter();
    let mut selected = None;
    let mut rest = vec![];
    while let Some(a) = it.next() {
        if a == "--" {
            rest.push(a);
            rest.extend(it);
            break;
        }
        if a == "-e" || a == "--environment" {
            selected = Some(it.next().ok_or("--environment requires a name")?);
        } else if let Some(value) = a.strip_prefix("--environment=") {
            selected = Some(value.to_owned());
        } else {
            rest.push(a);
            rest.extend(it);
            break;
        }
    }
    Ok((selected, rest))
}
pub fn parse_exec(
    args: &[String],
    literal: bool,
) -> Result<crate::exec::Request, Box<dyn std::error::Error>> {
    let mut r = crate::exec::Request::default();
    let mut i = 0;
    if !literal {
        while let Some(a) = args.get(i) {
            match a.as_str() {
                "--" => {
                    i += 1;
                    break;
                }
                "-t" | "--tty" => r.tty = true,
                "--setenv" | "--cwd" | "-w" => {
                    i += 1;
                    let value = args.get(i).ok_or("missing exec option value")?.clone();
                    if a == "--setenv" {
                        r.environment.push(value);
                    } else {
                        r.cwd = Some(value);
                    }
                }
                a if a.starts_with("--setenv=") => r.environment.push(a[9..].into()),
                a if a.starts_with('-') => {
                    return Err(
                        format!("unknown exec option '{a}'; put -- before the command").into(),
                    )
                }
                _ => break,
            }
            i += 1;
        }
    }
    r.argv = args[i..].to_vec();
    if r.argv.is_empty() {
        return Err("usage: trames [-e NAME] exec [-t] [--setenv KEY=VALUE] [--cwd DIR] -- COMMAND [ARG...]".into());
    }
    for entry in &r.environment {
        if !entry.split_once('=').is_some_and(|(k, _)| !k.is_empty()) {
            return Err("--setenv requires KEY=VALUE".into());
        }
    }
    if r.argv
        .iter()
        .chain(&r.environment)
        .any(|a| a.contains('\0'))
    {
        return Err("arguments cannot contain NUL".into());
    }
    if r.cwd.as_ref().is_some_and(|cwd| cwd.contains('\0')) {
        return Err("working directory cannot contain NUL".into());
    }
    r.size = r.tty.then(crate::terminal::window_size).flatten();
    Ok(r)
}

#[derive(Debug)]
pub struct Foreground {
    pub cpus: usize,
    pub timeout: Option<std::time::Duration>,
    pub rootfs: Option<String>,
    pub request: crate::exec::Request,
}
impl Foreground {
    /// Explicit host-run options followed by literal guest argv. A shell's default
    /// command is supplied by its product, never executed by a host command shell.
    pub fn parse(
        args: Vec<String>,
        default_command: Option<&[&str]>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut args = args.into_iter();
        let mut cpus = 1;
        let mut timeout = None;
        let mut rootfs = None;
        let mut options = vec![];
        let command = loop {
            match args.next().as_deref() {
                Some("--") => break args.collect::<Vec<_>>(),
                Some("--cpus")=>{cpus=args.next().ok_or("missing CPU count")?.parse()?;if !(1..=32).contains(&cpus){return Err("CPU count must be 1..=32".into());}},
                Some("--timeout-ms")=>{let ms=args.next().ok_or("missing timeout")?.parse::<u64>()?;if !(1..=86400000).contains(&ms){return Err("timeout must be 1..=86400000 ms".into());}timeout=Some(std::time::Duration::from_millis(ms));},
                Some("--rootfs")=>rootfs=Some(args.next().ok_or("missing rootfs directory")?),
                Some("--pty"|"-t"|"--tty")=>options.push("-t".into()),
                Some("--no-pty")=>options.retain(|s|s!="-t"),
                Some(option @ ("--setenv"|"--cwd"|"-w"))=>{options.push(option.into());options.push(args.next().ok_or("missing exec option value")?);},
                Some(option) if option.starts_with("--setenv=")=>options.push(option.into()),
                None if default_command.is_some()=>break default_command.unwrap().iter().map(|s|s.to_string()).collect(),
                _=>return Err("usage: trames run [--rootfs DIR] [--cpus N] [--timeout-ms N] [--pty] -- ELF [ARG...]".into()),
            }
        };
        options.push("--".into());
        options.extend(command);
        let request = parse_exec(&options, false)?;
        Ok(Self {
            cpus,
            timeout,
            rootfs,
            request,
        })
    }
}
