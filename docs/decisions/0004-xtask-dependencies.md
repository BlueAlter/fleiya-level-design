# ADR-0004 — xtask Architecture Linter Dependencies and Logging

## Status
ACCEPTED

## Context
Task T-0002 requires `cargo xtask check` to run formatting, clippy, tests, and an architecture linter verifying crate boundaries, third-party dependency rules, and filesystem usage. To inspect crate dependency graphs accurately and reliably across platforms, `xtask` needs structured access to workspace metadata. Furthermore, `CONSTRAINTS.md` forbids `print_stdout` and `print_stderr`, requiring `tracing` across all workspace crates.

## Decision
1. `xtask` is permitted to depend on `cargo_metadata` (for structured introspection of Cargo package dependency graphs) and `anyhow`.
2. Workspace logging uses `tracing` and `tracing-subscriber` for formatting log events.
3. `xtask` is a standalone developer tool in `xtask/` outside `crates/` and does not ship with the end-user binary `levelcraft`.

## Consequences
- Workspace dependency graph can be validated deterministically using Cargo's official metadata format.
- Output adheres to `CONSTRAINTS.md` by using structured logging instead of raw stdout/stderr prints.
- `xtask` dependencies do not pollute or affect the runtime crates in `crates/`.
