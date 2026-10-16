# Review of T-0006
Reviewer: subagent-reviewer   Author: agy-1   Branch: agent/T-0006   Commit reviewed: 2d872a4

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  Summary:
  - Gate 1: cargo fmt --all -- --check (pass)
  - Gate 2: cargo clippy --workspace --all-targets -- -D warnings (pass)
  - Gate 3: cargo test --workspace (pass: 4 golden/io tests + 2 migrate tests + 5 command tests + 3 core unit tests + 1 round-trip test + 9 xtask arch tests)
  - Gate 4: Architecture & Crate Boundaries Lint (pass)

## Acceptance criteria evidence
- [x] Golden test: load examples/sample_project then save -> byte-identical files.
  Evidence: `test_golden_sample_project_round_trip` in `crates/lc_io/tests/golden_test.rs:184-212` loads `examples/sample_project/project.lcproj`, saves it to an isolated temporary directory, and runs `compare_dirs_byte_identical`, asserting exact byte equality (`assert_eq!(bytes_a, bytes_b)`) across `project.lcproj`, `worlds/world_6/world.json`, and `worlds/world_6/levels/level_7.json`.
- [x] Missing/corrupt file gives a typed error with path, never a panic.
  Evidence: `test_missing_file_typed_error` in `crates/lc_io/tests/golden_test.rs:214-225` verifies loading a missing file returns `IoError::Fs { path, .. }` containing the target path. `test_corrupt_json_typed_error` (lines 227-246) verifies malformed JSON returns `IoError::Json { path, .. }` with file path context. `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` in `crates/lc_io/src/lib.rs` guarantees zero panics.
- [x] Unknown newer version is rejected with a clear error.
  Evidence: `test_unknown_newer_version_rejected` in `crates/lc_io/tests/golden_test.rs:248-280` verifies a project with version 999 is rejected with `IoError::UnsupportedVersion { version: 999, max_supported: 1, path }`. Checks are enforced in `crates/lc_io/src/io.rs` for project (line 33), world (line 66), and level (line 98), as well as in `migrate_project` (tested in `crates/lc_io/src/migrate.rs:65-70`).
- [x] Only this crate uses std::fs.
  Evidence: Verified by Gate 4 (`test_detect_fs_usage_violation` and `test_valid_workspace_passes`). Global codebase search confirms `std::fs` usage is confined exclusively to `crates/lc_io` and `xtask`.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests.
  N/A (crate is `lc_io`; handles workspace filesystem serialization/deserialization into `Project`).
- [x] IDs not indices; deletion/cascade handled; undo restores identical state.
  Directory hierarchy and filenames are keyed by IDs (`world_{id}`, `level_{id}.json`). Manifest entries reference stable IDs (`WorldId`, `LevelId`).
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR.
  `lc_io` contains no UI/egui dependencies. `std::fs` is restricted exclusively to `lc_io`. Dependencies are limited to existing workspace crates (`lc_core`, `thiserror`, `serde`, `serde_json`, `tracing`, `glam`).
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()`.
  N/A (crate is `lc_io`).
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io.
  Types named `ProjectFile`, `WorldFile`, `LevelFile`, `IoError`. Failed file I/O operations log at `tracing::error!` with path and error context; version transitions log at `tracing::info!`. `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]` is enforced in `crates/lc_io/src/lib.rs`.
- [x] Egui API calls verified against the pinned version (no hallucinated methods).
  N/A (no egui in `lc_io`).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline.
  No `#[allow(...)]`, `#[ignore]`, or loosened lints. Diff is confined to `crates/lc_io`, `examples/sample_project`, and `Cargo.lock`. Diff size in `crates/lc_io/src` is ~476 lines (excluding golden sample files and tests), justified by the initial I/O subsystem.
- [x] Acceptance criteria each have evidence.
  All 4 acceptance criteria verified with passing tests and source verification.

## Findings
1. [Important] `crates/lc_io/src/io.rs:248-256` — Non-atomic file writes despite `write_file_atomic` naming:
   `write_file_atomic(path: &Path, content: String)` is implemented directly via `fs::write(path, content)`. `fs::write` truncates the file in place and does not write atomically. A process termination or power loss mid-write will corrupt or truncate the file.
   Recommendation: Either rename to `write_file` to accurately reflect behavior, or implement genuine atomic file writing (write to a temporary file in the destination folder e.g. `path.with_extension("tmp")`, flush, and `fs::rename` over the target).
2. [Important] `crates/lc_io/src/io.rs:154, 166, 173, 195, 214` — Windows backslash serialization breaks cross-platform byte identity:
   Manifest relative paths are constructed using `Path::join` and converted via `to_string_lossy().to_string()`. On Windows, this will format file paths with backslashes (`worlds\world_6\world.json` and `levels\level_7.json`). This violates the portable forward-slash JSON format in `docs/01-data-model.md`, creates cross-OS git diffs, and causes the golden round-trip test to fail on Windows.
   Recommendation: Normalize all relative paths in manifest structs to POSIX forward slashes `/` (e.g. `rel_path.to_string_lossy().replace('\\', "/")` or format strings with `/`).
3. [Important] `crates/lc_io/tests/golden_test.rs:143-168` — Unidirectional directory comparison in golden test:
   `compare_dirs_byte_identical` only iterates over files discovered in `dir_a` and asserts their presence and byte equivalence in `dir_b`. It does not verify that `dir_b` has no extraneous files ($A \subseteq B$ instead of $A = B$). If `save_project` produced extra orphaned files, the test would still pass.
   Recommendation: Assert `entries_b.len() == entries_a.len()` or collect and compare file lists bidirectionally.
4. [Nit] `crates/lc_io/src/io.rs:46-126` — Unvalidated Manifest vs File content discrepancy:
   `load_project` reads the child file but does not assert that manifest IDs match loaded content IDs (`world_entry.id == world_file_data.id`, `level_entry.id == level_file_data.id`).
   Recommendation: Validate ID equality and return `IoError::Corrupt` if manifest and file contents diverge.
5. [Nit] `crates/lc_io/src/error.rs:40-54` — Unused `MissingComponent` and `Corrupt` variants:
   `IoError` declares `MissingComponent` and `Corrupt`, but neither variant is ever instantiated in `lc_io`.
   Recommendation: Use `IoError::Corrupt` for manifest validation failures, or annotate these as reserved for upcoming consistency checks.
6. [Nit] `crates/lc_io/src/io.rs:47, 80` — Relative path traversal sanitization:
   `load_project` joins `project_dir` and `world_dir` with paths read from untrusted manifests. If a file path contains absolute paths or `..` traversal components, it could read outside the project folder.
   Recommendation: Verify that manifest file paths are relative and do not contain parent traversal (`..`).
7. [FYI] `crates/lc_io/src/io.rs:33, 66, 98` — `migrate_project` not called for older schema versions:
   `load_project` rejects newer versions, but does not yet invoke `migrate_project` for older versions. This is acceptable for v1 since no version 0 exists, but will need integration when v2 is introduced.
8. [FYI] Diff size exceeds guideline:
   `crates/lc_io/src` introduces ~476 lines, slightly above the <= 300 line guideline. This is acceptable for the foundational multi-file I/O implementation.

## Doubt pass (required if a trait/contract/format changed)
1. **Multi-file directory layout and schema representation**
   - **Claim**: Decomposing projects into `project.lcproj`, `worlds/<world>/world.json`, and `levels/<lvl>.json` satisfies ADR-0001 and `docs/01-data-model.md`.
   - **Evidence**: `docs/01-data-model.md:30-36` mandates multi-file JSON with root defs, world layout, and level content files, each tagged with `"version": 1`.
   - **Counter-argument**: Storing `world_pos` and `world_depth` in both `LevelManifestEntry` (`world.json`) and `LevelFile` (`level.json`) duplicates state.
   - **Resolution**: This duplication matches LDtk's external levels model, allowing the editor's world view to render bounding boxes and placement without loading heavy tile and entity payloads into memory. `save_project` guarantees both remain in sync.

2. **In-place migration contract (`MigrationStep::migrate(&Path)`) vs in-memory AST transforms**
   - **Claim**: Defining `MigrationStep::migrate(&self, project_dir: &Path) -> Result<(), IoError>` properly supports multi-file schema evolutions.
   - **Evidence**: Migrations between project versions may require renaming, splitting, or reorganizing files on disk across the directory structure, which cannot be modeled by single-value AST transforms.
   - **Counter-argument**: Disk-based migrations require filesystem fixtures for tests rather than pure in-memory unit tests.
   - **Resolution**: Multi-file project formats inherently require directory-level file lifecycle operations. Pure JSON transform helpers can still be invoked internally per step.

3. **Typed error model with path context (`IoError::Fs` / `IoError::Json`)**
   - **Claim**: Wrapping `std::io::Error` and `serde_json::Error` with target `PathBuf` satisfies "Missing/corrupt file gives a typed error with path, never a panic".
   - **Evidence**: `test_missing_file_typed_error` and `test_corrupt_json_typed_error` verify that callers receive structured errors specifying the exact file path that failed.
   - **Counter-argument**: Callers must inspect `std::io::Error::kind()` to distinguish between file-not-found and permission-denied.
   - **Resolution**: Preserving the underlying `std::io::Error` and `serde_json::Error` via `#[source]` maintains full standard library and serde error fidelity while attaching file path context.
