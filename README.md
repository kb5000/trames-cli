<p align="center">
  <img src="https://raw.githubusercontent.com/project-trames/.github/main/assets/trames-icon.png" alt="Trames icon" width="108">
</p>

<h1 align="center">Trames CLI</h1>

<p align="center">
  <strong>The shared host command-line and wire-protocol library for Project Trames.</strong>
</p>

<p align="center">
  Literal argument parsing, binary stream multiplexing, raw PTY lifecycle management, and framed IPC clients.<br>
  Decoupled from execution engines and shared across Trames products.
</p>

<p align="center">
  <a href="https://github.com/project-trames/trames-cli/actions/workflows/check.yml"><img src="https://github.com/project-trames/trames-cli/actions/workflows/check.yml/badge.svg?branch=main" alt="Shared CLI checks"></a>
  <a href="crates/trames-cli-common/Cargo.toml"><img src="https://img.shields.io/badge/crate-trames--cli--common-f06b45?style=flat-square" alt="trames-cli-common"></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/Rust-nightly-b65437?style=flat-square&amp;logo=rust&amp;logoColor=white" alt="Rust nightly"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-7c5269?style=flat-square" alt="GPL-3.0-or-later"></a>
</p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#modules">Modules</a> ·
  <a href="#integration">Integration</a> ·
  <a href="#building-and-testing">Building</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

## Features

`trames-cli` provides the **`trames-cli-common`** Rust library driving host CLI parsing, terminal sessions, and IPC protocols across Trames products (`trames-ipad`, `trames-mac`, and future hosts):

- **Literal Argument Parsing**: Captures command arrays, working directories, and environment variables without host-shell word splitting or expansion quirks.
- **Strict Binary Stream Separation**: Multiplexes binary `stdin`, `stdout`, and `stderr` with exact byte fidelity, EOF propagation, and guest exit codes.
- **Raw Terminal & PTY Lifecycle**: POSIX `termios` raw mode, `SIGWINCH` window-size forwarding, and guaranteed host terminal restoration on exit.
- **Self-Pipe Signal Dispatch**: Readiness-driven `SIGINT`/`SIGTERM` forwarding without busy-polling or background timer threads.
- **Framed Wire Protocols**: Implements the `TRMEXEC1` (command execution) and `TRAMES02` (interactive console streaming) Unix-socket client protocols.

---

## Modules

| Module | Responsibility |
| :--- | :--- |
| [`arguments`](crates/trames-cli-common/src/arguments.rs) | Environment selection, exec syntax validation, and flag normalization. |
| [`exec`](crates/trames-cli-common/src/exec.rs) | `TRMEXEC1` protocol framing, deadlines, and streaming pipelines. |
| [`console`](crates/trames-cli-common/src/console.rs) | `TRAMES02` interactive console framing and session discovery. |
| [`terminal`](crates/trames-cli-common/src/terminal.rs) | POSIX `termios` raw-mode management and window size queries. |
| [`signals`](crates/trames-cli-common/src/signals.rs) | Unix signal trapping via self-pipe readiness. |
| [`storage`](crates/trames-cli-common/src/storage.rs) | Storage budget parsing (e.g. `20GiB`, `512MiB`) and host volume queries. |

---

## Integration

Consuming repositories pin `trames-cli` as a Git submodule under `cli/`:

```toml
[dependencies]
trames-cli-common = { path = "cli/crates/trames-cli-common" }
```

---

## Building and Testing

### Prerequisites
- Rust nightly toolchain (pinned in [`rust-toolchain.toml`](rust-toolchain.toml))
- Linux or macOS host

### Checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

---

## Contributing & Security

- **Contributing**: See [`CONTRIBUTING.md`](CONTRIBUTING.md) and [`AGENTS.md`](AGENTS.md). Public issues and PRs are normally analyzed with AI tools—do not post secrets or personal data.
- **Security**: Report vulnerabilities privately per [`SECURITY.md`](SECURITY.md).

---

## License

Trames CLI is licensed under the **GNU General Public License v3.0 or later** ([`LICENSE`](LICENSE)).
Third-party notices are recorded in [`THIRD_PARTY_NOTICES.txt`](THIRD_PARTY_NOTICES.txt).

---

<p align="center">
  Part of <a href="https://github.com/project-trames">Project Trames</a> · A portable userspace Linux runtime with pluggable execution engines.
</p>
