# Third-party dependencies — Trames CLI

Reviewed on 2026-11-26. Third-party components keep their own copyright and
license terms. The project's GPL-3.0-or-later license does not replace them.

The complete Rust license texts and copyright notices are collected in
[THIRD_PARTY_NOTICES.txt](../../THIRD_PARTY_NOTICES.txt). The reviewed source
origins, package checksums and selected license branches are recorded in
[third-party/licenses.json](../../third-party/licenses.json). `OR` selects an
available branch; `AND` retains every applicable additional term, including the
BSD notice in `encoding_rs` and the Unicode notice in `unicode-ident`.

<!-- BEGIN GENERATED RUST DEPENDENCIES -->

## Rust dependencies

Generated from the committed lockfiles. This is the complete locked closure,
including optional, build, test and other-target packages; it is not a claim
that every entry is linked into each product. Project-owned packages are excluded.

| Package | Version | Declared license | Selected license | Scope |
| --- | --- | --- | --- | --- |
| itoa | 1.0.19 | MIT OR Apache-2.0 | MIT | Main |
| libc | 0.2.190 | MIT OR Apache-2.0 | MIT | Main |
| memchr | 2.8.3 | Unlicense OR MIT | MIT | Main |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | MIT | Main |
| quote | 1.0.47 | MIT OR Apache-2.0 | MIT | Main |
| serde | 1.0.229 | MIT OR Apache-2.0 | MIT | Main |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | MIT | Main |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | MIT | Main |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | MIT | Main |
| syn | 3.0.10 | MIT OR Apache-2.0 | MIT | Main |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | MIT AND Unicode-3.0 | Main |
| zmij | 1.0.23 | MIT | MIT | Main |

Auxiliary lockfiles: none.

<!-- END GENERATED RUST DEPENDENCIES -->

## Repository boundary

This repository supplies a host command-line library. It does not vendor a CPU
backend, Linux kernel runtime, Swift frontend, native graphics driver or guest
root filesystem. Products consuming it maintain notices for those components
in their own repositories. The command-line parser and support crates above
retain their original permissive terms.

## CI tools

GitHub Actions used by this repository: `actions/checkout@v4`, `dtolnay/rust-toolchain@master`. Their upstream action implementations are MIT-licensed build services, not linked product libraries.

## Updating and checking this record

Use Python 3.9 or later:

```sh
python3 scripts/prepare-third-party-notices.py --check
```

After changing dependencies, review the new exact package source and license
texts, then update `third-party/licenses.json`. Run the script without `--check`
to refresh this table, the full notice bundle and `third-party/inputs.json`.
The generator checks original-text hashes and Cargo source checksums, rejects
unreviewed package versions, and downloads nothing. Its input fingerprints catch
lockfile drift even when a package is used only by a benchmark or another target.
The selected branch is a recorded license choice, not a change to upstream terms.
