---
id: T-0001
title: Cargo workspace skeleton and empty eframe window
status: todo
owner: -
role: integrator
scope: [Cargo.toml, rust-toolchain.toml, crates/*, examples/]
depends_on: [T-0000]
skills: [incremental-implementation, test-driven-development, source-driven-development, git-workflow-and-versioning]
---

## Goal
Create the workspace with all crates listed in docs/02-architecture.md (empty libs, `lc_app` opens an empty eframe window). Pin versions of
eframe/egui/egui_dock; add `[workspace.lints]` from CONSTRAINTS.md. Add `examples/sample_project/` placeholder.
## Acceptance criteria
- [ ] `cargo build --workspace` and `cargo run -p lc_app` show an empty window.
- [ ] Lints from CONSTRAINTS.md configured; crates inherit them.
- [ ] Each crate depends only on what the matrix allows (no egui in lc_core).
## Out of scope
Any real feature code.
