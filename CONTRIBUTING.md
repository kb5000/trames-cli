# Contributing to Trames CLI

Thank you for contributing to **`trames-cli`** (`trames-cli-common`), the shared host CLI and wire-protocol library for Project Trames.

---

## Scope & Standards

Keep `trames-cli-common` decoupled from execution hypervisors (HVF/KVM), guest Linux kernel internals, and GUI frameworks. Both Linux and macOS hosts must build and pass all tests. See [`AGENTS.md`](AGENTS.md) for repository workflow conventions.

## Verification

Before opening a pull request against `main`, run:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

## AI-Assisted Workflow Disclosure

Trames is an AI-driven open-source project. Unless explicitly stated otherwise, public GitHub issues, pull requests, and discussions are normally analyzed with external AI tools. **Do not post passwords, private keys, or personal data in public threads.** Public submission does not transfer copyright or grant model-training permission.

## Licensing

Contributions are licensed under **GNU General Public License v3.0 or later** ([`LICENSE`](LICENSE)).
