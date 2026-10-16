# ADR-0007 — Workspace Dependencies for lc_shell

## Status
ACCEPTED

## Context
`crates/lc_shell` implements the shell dock layout, menu bar, status bar, and central document docking.
- `glam` is needed for status bar cursor coordinates (`IVec2`).
- `serde` is needed for serializing and deserializing dock layout configuration (`ShellDockLayout`) for persistence.

## Decision
Add `glam = { workspace = true }` and `serde = { workspace = true }` to `crates/lc_shell/Cargo.toml`.

## Consequences
- Clean persistence of dock panel positions and bottom dock collapse state.
- Strongly-typed coordinate reporting in the shell status bar.
- Conforms to architecture boundaries and workspace dependency specifications.
