---
id: T-0002
title: xtask: check command with architecture lint
status: done
owner: agy-1
role: integrator
scope: [xtask/, .cargo/config.toml]
depends_on: [T-0001]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
`cargo xtask check` runs fmt --check, clippy -D warnings, test --workspace, then an architecture lint using `cargo metadata`
that fails when the dependency matrix in docs/02-architecture.md is violated (also third-party rules: egui only in allowed crates, std::fs only in io, egui_dock only in shell).
Uncomment the alias in .cargo/config.toml.
## Acceptance criteria
- [x] Each gate runs and returns non-zero on failure, with a readable message.
- [x] Tests for the lint: a fixture violating each rule is detected.
- [x] `cargo xtask check` is green on main.
