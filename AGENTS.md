# Shared host frontend

- Keep runtime/CPU backends, guest memory and product GUI/lifecycle out of this repository.
- Preserve literal argv, separate binary streams, existing wire formats and exit codes.
- Use actual descriptor/signal/deadline readiness. Add no polling timer, coordinator or service thread.
- Retain original resource owners across descriptor close/reuse. Restore host terminal/signal state.
- Parsing/help/completion must not initialize a CPU backend, device or environment.
- Validate affected Mac/Linux consumers, then commit/push to main with paired product pins.
- Preserve tested commits and unrelated in-progress work; use fast-forward integration.
