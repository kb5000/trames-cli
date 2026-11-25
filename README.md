<p align="center">
  <img src="https://raw.githubusercontent.com/project-trames/.github/main/assets/trames-icon.png" alt="Trames icon" width="112">
</p>

<h1 align="center">Trames CLI</h1>

<p align="center">
  <strong>The shared host command-line and protocol client engine for Project Trames.</strong>
</p>

<p align="center">
  Literal argument parsing, isolated binary stream multiplexing, raw PTY management, and framed IPC clients.<br>
  Decoupled from execution hypervisors and shared across Mac, Linux, and interpreter frontends.
</p>

<p align="center">
  <a href="https://github.com/project-trames/trames-cli/actions/workflows/check.yml"><img src="https://github.com/project-trames/trames-cli/actions/workflows/check.yml/badge.svg?branch=main" alt="Shared CLI checks"></a>
  <a href="crates/trames-cli-common/Cargo.toml"><img src="https://img.shields.io/badge/crate-trames--cli--common-f06b45?style=flat-square" alt="trames-cli-common library"></a>
  <a href=".github/workflows/check.yml"><img src="https://img.shields.io/badge/hosts-Linux%20%7C%20macOS-30303d?style=flat-square" alt="Linux and macOS host checks"></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/Rust-nightly-b65437?style=flat-square&amp;logo=rust&amp;logoColor=white" alt="Rust nightly"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-7c5269?style=flat-square" alt="GPL-3.0-or-later"></a>
</p>

<p align="center">
  <a href="#features">Features</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#integration">Integration</a> ·
  <a href="#crates--modules">Modules</a> ·
  <a href="#building-and-testing">Building</a> ·
  <a href="#documentation">Documentation</a>
</p>

---

## Features

`trames-cli` provides the **`trames-cli-common`** Rust library—the canonical frontend layer driving terminal interactions, session attachments, and IPC communication across all Trames host platforms:

* **Literal Argument Parsing**: Accurately capture command arrays, working directories, and environment variables without leaking host-shell expansions or word-splitting quirks.
* **Strict Binary Stream Separation**: Cleanly multiplex and stream binary `stdin`, `stdout`, and `stderr` independently, preserving exact byte fidelity, EOF conditions, and guest exit codes.
* **Robust Terminal & PTY Lifecycles**: Full raw-mode terminal management, real-time window size (`SIGWINCH`) synchronization, and guaranteed host terminal restoration upon exit or interruption.
* **Event-Driven Signal & Deadline Dispatch**: Self-pipe readiness-driven signal handling that forwards interactive signals (`SIGINT`, `SIGTERM`) without busy-polling or background timer threads.
* **Framed Wire Protocols**: Implements the official `TRMEXEC1` (command execution) and `TRAMES02` (interactive console streaming) socket client protocols.
* **Zero System Coupling**: Completely decoupled from hypervisor drivers (HVF/KVM), the guest Linux runtime, and native GUI frameworks.

---

## Architecture

`trames-cli-common` serves as the universal user-facing entry layer. Product repositories consume it as a submodule to deliver a consistent CLI experience regardless of the underlying execution engine:

```text
┌─────────────────────────────────────────────────────────────┐
│                 Host Terminal / User Shell                  │
└──────────────────────────────┬──────────────────────────────┘
                               │ CLI Invocations (Raw Args / PTY)
┌──────────────────────────────┴──────────────────────────────┐
│                      trames-cli-common                      │
│   ┌────────────────────┐  ┌─────────────────────────────┐   │
│   │  Argument Parsing  │  │ Terminal Raw Mode / Resize  │   │
│   └────────────────────┘  └─────────────────────────────┘   │
│   ┌────────────────────┐  ┌─────────────────────────────┐   │
│   │ Stream Forwarding  │  │ Protocols (TRMEXEC/TRAMES02)│   │
│   └────────────────────┘  └─────────────────────────────┘   │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼ Local API Calls              ▼ Unix Domain Socket
┌─────────────────────────────┐┌──────────────────────────────┐
│        trames-linux         ││          trames-mac          │
│   Standalone Foreground CLI ││   Manager Daemon & Workers   │
│   Backed by KVM Engine      ││   Backed by HVF Engine       │
└─────────────────────────────┘└──────────────────────────────┘
```

### Ecosystem Roles

* [**Trames Mac**](https://github.com/project-trames/trames-mac): Embeds `trames-cli-common` to parse options and connect via Unix sockets to background HVF worker environments.
* [**Trames Linux**](https://github.com/project-trames/trames-linux): Compiles directly against this library to manage direct terminal sessions, KVM vCPU execution, and directory mounting.
* [**Trames iPad (Interpreter Runner)**](https://github.com/project-trames/trames-ipad): Uses the shared parser and stream forwarding contracts within the standalone `trames-run` host tool.

---

## Integration

Consuming products link this repository as a Git submodule under `cli/`:

```toml
# In workspace Cargo.toml or crate dependencies:
[dependencies]
trames-cli-common = { path = "cli/crates/trames-cli-common" }
```

### 1. Parse Execution Requests

Parse user input directly into structured execution commands without spawning processes:

```rust
use trames_cli_common::arguments;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = [
        "--setenv", "MODE=production",
        "--cwd", "/workspace",
        "--", "/bin/printf", "%s\n", "Hello, Trames!",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let request = arguments::parse_exec(&raw_args, false)?;
    assert_eq!(request.argv, ["/bin/printf", "%s\n", "Hello, Trames!"]);
    assert_eq!(request.environment, ["MODE=production"]);
    assert_eq!(request.cwd.as_deref(), Some("/workspace"));
    Ok(())
}
```

### 2. Connect via Socket Protocols

Attach to a running environment worker or daemon session:

```rust
use std::path::Path;
use trames_cli_common::exec::{self, Request};

// The client derives the exec socket path from the environment's console path.
fn execute_remote(console_path: &Path, request: Request) -> std::io::Result<u8> {
    exec::run(console_path, request)
}
```

---

## Crates & Modules

The library is composed of modular components designed for high-concurrency and strict terminal control:

| Module | Primary Responsibility |
| :--- | :--- |
| [`arguments`](crates/trames-cli-common/src/arguments.rs) | Environment selection, exec syntax validation, and flag normalization. |
| [`exec`](crates/trames-cli-common/src/exec.rs) | `TRMEXEC1` protocol framing, deadline triggers, and streaming pipelines. |
| [`console`](crates/trames-cli-common/src/console.rs) | `TRAMES02` interactive console framing and session discovery. |
| [`terminal`](crates/trames-cli-common/src/terminal.rs) | POSIX termios management, raw mode capture, and window size querying. |
| [`signals`](crates/trames-cli-common/src/signals.rs) | Safe Unix signal trapping via self-pipe trick and notification readiness. |
| [`storage`](crates/trames-cli-common/src/storage.rs) | Storage budget parsing (e.g. `20GiB`, `512MiB`) and host volume queries. |

---

## Building and Testing

### Prerequisites

* Rust nightly toolchain (defined in [`rust-toolchain.toml`](rust-toolchain.toml)).
* Linux or macOS host (frontend tests interact with Unix terminal/socket APIs).

### Workspace Checks

```sh
git clone https://github.com/project-trames/trames-cli.git
cd trames-cli

# Run style and lint checks
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings

# Run unit and contract test suites
cargo test --workspace --locked

# Build release artifacts
cargo build --workspace --release --locked
```

Generate API documentation:
```sh
cargo doc --workspace --no-deps --open
```

---

## Documentation

| Domain | Deep Dives & Verification |
| :--- | :--- |
| **Public API** | [Library Source (`lib.rs`)](crates/trames-cli-common/src/lib.rs) |
| **Protocol Specifications** | [Wire Protocol & Contract Tests](crates/trames-cli-common/tests/contracts.rs) |
| **Signal Handling** | [Signal Management Architecture](crates/trames-cli-common/tests/signals.rs) |
| **Host Integration** | [Shared Host CLI Design Pattern](https://github.com/project-trames/trames-linux/blob/main/docs/architecture/host-cli.md) |
| **Continuous Integration** | [GitHub Actions Workflow Definition](.github/workflows/check.yml) |

---

## Community and Contributing

We welcome bug reports, reproduction cases, and pull requests. Please read our [Contribution Guidelines](https://github.com/project-trames/.github/blob/main/CONTRIBUTING.md) and [`AGENTS.md`](AGENTS.md) before submitting patches.

* **Issues**: [GitHub Issue Tracker](https://github.com/project-trames/trames-cli/issues)
* **Security**: For private vulnerability reports, see [SECURITY.md](https://github.com/project-trames/.github/blob/main/SECURITY.md)
* **Conduct**: We adhere to the [Project Code of Conduct](https://github.com/project-trames/.github/blob/main/CODE_OF_CONDUCT.md)

---

## License

Trames CLI is licensed under the **GNU General Public License v3.0 or later** ([GPL-3.0-or-later](LICENSE)).<br>
Third-party notices and attribution records are preserved in component trees.

---

<p align="center">
  Part of <a href="https://github.com/project-trames">Project Trames</a> · A portable userspace Linux runtime with pluggable execution engines.
</p>
