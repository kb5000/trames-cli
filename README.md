# Trames CLI

Common host CLI used by Trames Mac (HVF), Linux (KVM) and the interpreter runner.
This repository contains `trames-cli-common`; product executables remain thin
compositions with their own execution, lifecycle, native providers and GUI.

The library owns literal argv/environment selection, foreground options, Unix
terminal restoration, process-signal leases and existing TRMEXEC1 / TRAMES02
clients. Binary stdin/stdout/stderr, exit status and frame limits retain the
original protocol. Idle clients wait on sockets, stdin, signal self-pipes and
explicit deadlines, without periodic polling or helper/service threads.
Programmatic captures preserve their embedding's process signal handlers.

It has no runtime, interpreter, HVF, KVM, AppKit or graphics dependency. Parsing
and help paths never open devices or start an environment. Product-specific
commands, registries, backend diagnostics and GUI actions remain in the product.
The existing iPad workspace package `trames-cli` and `trames-run` executable keep
their identities; they use this library instead of carrying duplicate host code.

Products pin this repository as the `cli/` submodule. Update a tested shared
commit and all participating pins together. GNU GPL v3 or later, matching the
original Trames source.
