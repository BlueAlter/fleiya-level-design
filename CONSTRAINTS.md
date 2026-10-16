# CONSTRAINTS — the quality bar (enforced by `cargo xtask check` and CI)

## Hard gates (all must pass)
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- architecture lint (crate dependency matrix in `docs/02-architecture.md`)

## Workspace lints (set in root Cargo.toml `[workspace.lints]`)
- rust: `unsafe_code = "forbid"`
- clippy (deny): `dbg_macro`, `todo`, `unimplemented`, `print_stdout`, `print_stderr` -> use `tracing`
- clippy (deny in lc_core, lc_io): `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` (use `get`)

## Anti-cheating rules
- `#[allow(...)]` / `#[ignore]` / `cfg(not(test))` tricks need a one-line reason plus a task or ADR reference, and the reviewer must check them.
- Never delete or weaken an existing test to make a change pass. If a test is wrong, fix it in a separate, explained commit.
- Never lower thresholds or remove a gate in this file or in xtask without an ADR.

## Testing (Rust)
- Unit tests live next to the code (`#[cfg(test)] mod tests`). Integration tests in `crates/<name>/tests/`.
- Every `Command` has tests: `apply` changes what it claims; `apply` then `revert` restores a byte-identical `Project`; `merge` behaves.
- Every serialized type has a round-trip test. `examples/sample_project` must load and save to identical files (golden test).
- `Viewport` / coordinate conversion code gets unit tests (screen <-> world <-> grid), including negative coords and zoom.
- UI: use `egui_kittest` for interaction tests where practical (verify the crate/version first). Otherwise add a manual check list to the PR.
- Tests are deterministic: no sleeps, no wall-clock, no network, seeded randomness.

## Code
- Errors: `thiserror` in libraries, `anyhow` only in `lc_app` and `xtask`. No `String` errors.
- Logging: `tracing` only. Levels: error = user-visible failure, warn = recoverable oddity, info = user actions (save/open/command), debug/trace = internals.
  Every Command application logs at `debug`. Every failed IO logs at `error`.
- Public items have doc comments. Modules > 500 lines get split.
- Names: types `UpperCamel`, IDs end with `Id` (`LevelId`), commands end with `Command`, views end with `View`/`Panel`.
- No per-frame allocation in canvas hot paths unless measured as irrelevant. Measure before optimizing.

## Change size
Aim for <= 300 changed lines (excluding generated/golden files) per PR. Larger: split the task and say so in the PR.

## Definition of done (per task)
1. Acceptance criteria in the task file all met, each with evidence (test name or screenshot/manual note).
2. Hard gates green. 3. Docs/ADR updated if a contract or decision changed. 4. Status `in_review`. 5. Review verdict APPROVE.
