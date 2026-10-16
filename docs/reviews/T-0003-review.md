# Review of T-0003
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0003   Commit reviewed: 98d6908

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (passed)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (passed)
  - Gate 3: cargo test --workspace (passed: 12 crate targets, 9 unit tests in xtask, all doc-tests ok)
  - Gate 4: Architecture & Crate Boundaries Lint (passed)
  All gates passed cleanly with zero warnings or errors.

## Acceptance criteria evidence
- [x] Workflow file valid and runs the same gates as local.
  - Evidence: `.github/workflows/ci.yml` is valid YAML and conforms to the GitHub Actions workflow schema. It defines two jobs: `check-linux` (on `ubuntu-latest`) and `check-windows` (on `windows-latest`). Both jobs run `cargo xtask check`, ensuring exact parity with local gates (formatting check, clippy with `-D warnings`, full workspace test suite, and the architecture/dependency matrix linter). Linux dependencies (`libasound2-dev`, `libgl1-mesa-dev`, `libx11-xcb-dev`, `libxkbcommon-x11-dev`, `libwayland-dev`, `libxcursor-dev`, `libxi-dev`) are installed to satisfy `eframe`/`winit` requirements. `Swatinem/rust-cache@v2` is configured on both platforms to optimize build and test cache reuse.
- [x] Fails on a deliberately broken clippy warning (demonstrate in PR description).
  - Evidence: Verified experimentally by injecting an unnecessary mutable variable binding (`let mut x = 1; x`) into `crates/lc_core/src/lib.rs`. Running `cargo xtask check` terminated immediately at Gate 2 with non-zero exit status (exit status: 101) and explicit error output:
    ```
    error: variable does not need to be mutable
      --> crates/lc_core/src/lib.rs:12:9
       |
    12 |     let mut x = 1;
       |         ----^
       = note: `-D unused-mut` implied by `-D warnings`
    Error: Gate failed: clippy reported warnings or errors.
    Caused by:
        Command `cargo clippy --workspace --all-targets -- -D warnings` failed with exit status: exit status: 101
    ```
    After reverting the injected warning, `cargo xtask check` returned to green.
- [x] Autopilot rule respected: only create workflow file, no remote.
  - Evidence: Verified that no remote is configured (`git remote -v` outputs nothing), no push was attempted, and the working diff is strictly restricted to `.github/workflows/ci.yml` (and task status update in `docs/tasks/T-0003-ci.md`).

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  - N/A for T-0003 (CI workflow configuration; no command or domain logic).
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  - N/A for T-0003.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  - Satisfied. No crate dependencies or Rust code added. Uses well-vetted, standard GitHub Actions (`actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`).
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  - N/A for T-0003.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  - Satisfied. No code changes to crates.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  - N/A for T-0003.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  - Satisfied. Zero silenced lints or skipped tests. Diff is +63/-3 lines within scope `[.github/workflows/]`, well within the 300-line budget.
- [x] Acceptance criteria each have evidence.
  - Satisfied. Both criteria thoroughly tested and verified with reproducible evidence above.

## Findings
Use labels: Critical (must fix) / Important (should fix) / Nit / Optional / FYI. Reference file:line.
1. [Important] `.github/workflows/ci.yml:10-11`: Unconditional `cancel-in-progress: true` on `main`. The concurrency configuration `group: ${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true` will cancel in-progress CI runs on the `main` branch if successive commits are pushed in close succession. On `main`, completing CI runs is desirable to populate build caches and ensure unbroken test records. Recommend updating to:
   ```yaml
   concurrency:
     group: ${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
   ```
2. [Important] `.github/workflows/ci.yml:14,44`: Missing `timeout-minutes` on jobs. By default, GitHub Actions jobs timeout after 360 minutes (6 hours). If a test hangs or experiences deadlock during parallel CI runs, runners will remain locked. Recommend adding `timeout-minutes: 15` (or 20) to both `check-linux` and `check-windows`.
3. [Nit] `.github/workflows/ci.yml:1-12`: Missing top-level least-privilege `permissions` block. Following GitHub/OpenSSF security best practices, explicit read-only permissions should be set:
   ```yaml
   permissions:
     contents: read
   ```
4. [Nit] `.github/workflows/ci.yml:3`: In YAML 1.1, unquoted `on:` evaluates to a boolean `true` in standard YAML parsers. While GitHub Actions handles unquoted `on:` natively, quoting `'on':` prevents parser ambiguity and avoids warnings in third-party linters (e.g., yamllint / actionlint).
5. [FYI] `.github/workflows/ci.yml:31`: `libgl1-mesa-dev` is a transitional dummy package on newer Ubuntu/Debian releases (e.g., Ubuntu 24.04 noble), pointing to `libgl-dev`. Specifying `libgl-dev` directly is recommended for future compatibility.
6. [FYI] `.github/workflows/ci.yml:14-60`: Job deduplication via matrix. The Linux and Windows jobs share identical steps other than the apt system package installation. In the future, this can be unified into a single matrix job (`strategy: matrix: os: [ubuntu-latest, windows-latest]`) using `if: runner.os == 'Linux'` for the dependency step.

## Doubt pass (required if a trait/contract/format changed)
- **Claim**: The GitHub Actions workflow in `.github/workflows/ci.yml` reliably mirrors local validation across target OS platforms and protects main from broken builds and lint failures.
- **Evidence**: The workflow invokes `cargo xtask check`, identical to the local gate runner. When tested against deliberate clippy failures, `cargo xtask check` fails with exit code 101. The Linux job provisions all required GUI/X11/Wayland libraries, and both jobs utilize `Swatinem/rust-cache@v2` for performant runner caching.
- **Counter-argument**: Windows runners currently do not install additional system dependencies; if a future dependency requires native C/C++ libraries on Windows, builds may fail. Additionally, unconditional cancellation could abort CI runs on `main`.
- **Resolution**: Windows MSVC provides default Windows SDK libraries required by `winit` and `eframe` without extra setup. The findings noted above are non-blocking enhancements that can be addressed in future maintenance without blocking T-0003 completion.
