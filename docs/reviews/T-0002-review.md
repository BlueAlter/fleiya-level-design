# Review of T-0002
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0002   Commit reviewed: 8057a77

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (passed)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (passed)
  - Gate 3: cargo test --workspace (passed: 9 unittests in xtask, doc-tests ok)
  - Gate 4: Architecture & Crate Boundaries Lint (passed)
  All gates passed in ~8s.

## Acceptance criteria evidence
- [x] Each gate runs and returns non-zero on failure, with a readable message.
  - Gate 1 (fmt failure): returns exit code 1 with `Error: Gate failed: cargo fmt check failed. Run cargo fmt --all to fix.`
  - Gate 2 (clippy failure): returns exit code 1 with `Error: Gate failed: clippy reported warnings or errors.`
  - Gate 3 (test failure): returns exit code 1 with `Error: Gate failed: workspace tests failed.`
  - Gate 4 (arch-lint failure): returns exit code 1 with `Error: Gate failed: N architecture lint violation(s) found.` and lists individual violations with source locations.
- [x] Tests for the lint: a fixture violating each rule is detected.
  - 9 unit tests in `xtask/src/arch_lint.rs` verify all matrix and third-party rules:
    - `test_valid_workspace_passes`
    - `test_detect_forbidden_core_dependency`
    - `test_detect_cross_feature_dependency`
    - `test_detect_feature_depending_on_io`
    - `test_detect_egui_in_core`
    - `test_detect_egui_dock_outside_shell`
    - `test_detect_serde_json_outside_io`
    - `test_detect_eframe_outside_app`
    - `test_detect_fs_usage_violation`
- [x] `cargo xtask check` is green on main/branch.
  - Clean execution with 0 warnings and 0 errors on branch `agent/T-0002`.
- [x] Alias in .cargo/config.toml is uncommented.
  - `.cargo/config.toml` contains `xtask = "run --package xtask --"`, enabling `cargo xtask check` and `cargo xtask board` from any directory in the workspace.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  - N/A for T-0002 (infrastructure/developer tooling; no data model or commands touched).
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  - N/A for T-0002.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  - Satisfied. T-0002 implements the automated lint enforcing these rules. All added dependencies (`cargo_metadata`, `anyhow`, `tracing`, `tracing-subscriber`) are documented and approved in ADR-0004 (`docs/decisions/0004-xtask-dependencies.md`).
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  - N/A for T-0002.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  - Satisfied. Uses `tracing` with `info`, `warn`, `error` levels. No `unwrap` in core/io.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  - N/A for T-0002 (no egui calls added).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  - Satisfied. No `#[allow(...)]` or `#[ignore]` attributes added. Zero skipped tests. Scope restricted to `xtask/`, `.cargo/config.toml`, `Cargo.toml`, `Cargo.lock`, `docs/decisions/0004-xtask-dependencies.md`, and `docs/tasks/T-0002-xtask.md`.
- [x] Acceptance criteria each have evidence.
  - Satisfied. Documented above with manual validation of all failure modes.

## Findings
1. [Important] `xtask/src/arch_lint.rs:345`: The unit test `test_detect_fs_usage_violation` creates a fixed directory `std::env::temp_dir().join("xtask_test_fs_violation")`. In multi-agent parallel workflows where multiple worktrees might run `cargo xtask check` concurrently on the same host, this shared path could cause race conditions or intermittent test failures. Recommend using `temp_dir.join(format!("xtask_test_fs_violation_{}", std::process::id()))` or a unique nonce.
2. [Important] `xtask/src/arch_lint.rs:166`: `validate_fs_usage` checks for `line.contains("std::fs")`. Grouped imports such as `use std::{fs, io};` or nested imports `use std::{fs::File};` would not be caught by this string check. Although full AST parsing via `syn` was deliberately excluded to avoid heavy dependencies per ADR-0004, consider hardening the matching logic (e.g. checking for `\bfs::` or grouped std imports) in a future iteration.
3. [Nit] `xtask/src/arch_lint.rs:166`: In `if line.contains("std::fs") || line.contains("use std::fs")`, the check `line.contains("use std::fs")` is redundant because any string containing `"use std::fs"` already contains `"std::fs"`.
4. [Nit] `xtask/src/arch_lint.rs:162-166`: In `validate_fs_usage`, inline comments (e.g., `let x = 1; // std::fs note`) and string literals (e.g., `tracing::info!("std::fs is forbidden")`) will trigger false-positive violations because only full lines starting with `//`, `/*`, or `*` are stripped.
5. [Nit] `xtask/src/arch_lint.rs:6`: `PackageDepInfo` is a public struct but lacks a doc comment, contrary to `CONSTRAINTS.md` ("Public items have doc comments").
6. [FYI] `xtask/src/arch_lint.rs:100`: `validate_dependencies` defaults to `_ => {}` for unknown crates. If a new crate is created that does not start with `lc_feature_` or match known crate names, it will not be validated against the dependency matrix. A future enhancement could reject unknown workspace crates not documented in `docs/02-architecture.md`.

## Doubt pass (required if a trait/contract/format changed)
- **Claim**: The `xtask check` command and architecture linter provide deterministic, robust enforcement of workspace boundaries, third-party crate restrictions, and code quality gates.
- **Evidence**: `xtask check` runs standard toolchain commands (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`) and parses Cargo metadata via `cargo_metadata` to enforce the dependency matrix. All 9 test cases in `arch_lint.rs` pass, and each gate was manually verified to exit non-zero with informative messages upon failure.
- **Counter-argument**: The filesystem scanner `validate_fs_usage` relies on line-based string matching rather than syntax-tree parsing, leaving theoretical blind spots (grouped imports like `use std::{fs}`) and edge-case false positives (string literals or inline comments containing `std::fs`).
- **Resolution**: Given ADR-0004's constraint to keep `xtask` lightweight and avoid pulling in heavy dependencies like `syn`, string-based detection is an appropriate initial implementation for Phase A. The highlighted edge cases are non-blocking for task T-0002 and can be tracked for future hardening.
