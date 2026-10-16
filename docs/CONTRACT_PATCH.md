# Level Craft — Contract Patch Progress & Verification Log

Progress tracker and audit log for `fix/contract-v0.2` contract hardening.

## Progress Checklist

- [x] **STEP 1: VERIFY**
  - [x] Run `scripts/board.sh`, `git status`, `git tag`, `cargo xtask check`, `cargo test --workspace`
  - [x] Compare results against claims in `docs/PHASE_A_REPORT.md` (47 tests, tag, gates)
  - [x] List all discrepancies in `docs/CONTRACT_PATCH.md`
  - [x] Commit Step 1 results
- [x] **STEP 2: INDEPENDENT RE-REVIEW of T-0008, T-0009, T-0010, T-0011**
  - [x] Re-review T-0008 (`docs/reviews/T-0008-rereview.md`)
  - [x] Re-review T-0009 (`docs/reviews/T-0009-rereview.md`)
  - [x] Re-review T-0010 (`docs/reviews/T-0010-rereview.md`)
  - [x] Re-review T-0011 (`docs/reviews/T-0011-rereview.md`)
  - [x] Fix every Critical and Important finding:
    - Fixed `RotatingLogWriter` mutex drop race condition and partial writes using `write_all`.
    - Fixed `DocKind` parsing for `Display`/`Debug` formatted IDs (`LevelId(N)`, `Level(LevelId(N))`).
    - Fixed `EventVisitor` field ordering space separation.
    - Pre-lowercased search query in `ConsoleFilter` to avoid per-iteration heap allocations.
    - Removed non-deterministic `thread::sleep` from `console_test.rs`.
    - Generated World and Level IDs using `max(id) + 1` across existing IDs in the project to prevent collisions.
    - Deduplicated "Move to Depth..." buttons in Level context menu and filtered current depth.
    - Supported double-clicking World header in project panel to open World document tab.
    - Included candidate depth drop slots (0, 1, -1) in World depth grouping.
    - Reset `self.project_path = None` on `File > New`.
    - Prevented clearing `is_dirty = false` on `SaveProject` when `project_path` is `None` (added error status and warning log).
    - Re-queried active panel ID after tab rendering in `render_dock_slot` to eliminate duplicate rendering on move.
    - Added `save_to_memory` and `load_from_memory` for `ShellDockLayout` via egui persisted memory.
    - Split `shell_app.rs` into `menu.rs`, `status_bar.rs`, and `dock_panels.rs`, bringing `shell_app.rs` under 325 lines (well below the 500-line constraint).
    - Added `find_layer_handler` alias in `Registry`.
    - Replaced `SystemTime::now()` in `project_panel_test.rs` with process ID.
  - [x] Commit Step 2 fixes and reviews
- [x] **STEP 3: CONTRACT GAP CHECK**
  - [x] 3a: Shared session state readable by panels, tools, inspector sections (`SessionState` in `lc_ui_api::session`, exposed via `EditorCtx`, updated via commands and `EditorEvent`s, documented in ADR-0009)
  - [x] 3b: Asset/texture service (`AssetService` trait in `lc_ui_api::assets`, implemented in `lc_shell` via `lc_io::read_bytes` and `image` decoding, cached, typed `AssetError`, documented in ADR-0010)
  - [x] 3c: DocKind location confirmation (`DocKind` lives in `lc_core::command` so `Command::focus` can use it without depending on `lc_ui_api`; re-exported in `lc_ui_api`; updated `docs/02-architecture.md`)
  - [x] 3d: Command error surfacing: logged at error level and surfaced in status bar on deferred apply failure; verified and covered with `test_command_error_surfacing_in_status_bar`
  - [x] 3e: Viewport `screen_to_world` negative coord floor verification, added `screen_to_world_f32`, and documented integer world coordinate quantization for zoom < 1 in `docs/02-architecture.md`
  - [x] 3f: Timing of `EditorCtx::exec` documented in `docs/02-architecture.md` + paint stroke multi-frame undo test `test_paint_stroke_multi_frame_undo_merging`
  - [x] 3g: Native file dialog (`rfd`) deferred to task `T-0109` for shell role with ADR-0011 to prevent desktop-portal/GTK linkage dependencies in headless CI
  - [x] 3h: `rust-toolchain.toml`: pinned newest stable `1.99.0` (verified by building egui 0.36 on stable without nightly flags, documented in ADR-0012)
  - [x] 3i: Panel tab drag & drop between left, right, and bottom docks implemented in `lc_shell` (~50 lines in `dock_layout.rs` and `dock_panels.rs`), verified with `test_panel_tab_drag_and_drop_between_docks`
  - [x] Commit Step 3 contract patches and ADRs
- [x] **STEP 4: RELEASE contract-v0.2**
  - [x] Update `docs/PHASE_A_REPORT.md` Section 3 verbatim signatures (including `Selection`, `DocKind`, `PanelId`, `CanvasCtx`, `LayerRenderCtx`, session-state, asset APIs)
  - [x] Add Section "Contract v0.2 changes" in `docs/PHASE_A_REPORT.md` (what, why, Phase B tasks affected)
  - [x] Update Phase B task files (`docs/tasks/T-0101.md`..`T-0108.md`) if scope/description changed
  - [x] Verify `cargo xtask check` is green (61 tests passing across workspace)
  - [x] Fresh-context review of entire diff (APPROVED by independent reviewer, all Important findings addressed)
  - [x] Merge to main with `git merge --no-ff`
  - [x] Tag `contract-v0.2`
  - [x] Remove worktree `../wt-contract-v0.2`

---

## Step 1 Verification Report & Discrepancy Log

### Command Executions & Outputs
1. **`scripts/board.sh`**:
   - Tasks T-0000 through T-0011 are all marked `done`.
   - Phase B tasks T-0101 through T-0108 and integration task T-0190 are marked `todo`.
   - Active branches: none.
2. **`git status`**:
   - Working tree clean on branch `fix/contract-v0.2`.
3. **`git tag`**:
   - Tag `contract-v0.1` present at commit `0c9c400`.
4. **`cargo xtask check`**:
   - Gate 1 (fmt): PASSED.
   - Gate 2 (clippy -D warnings): PASSED.
   - Gate 3 (cargo test): PASSED.
   - Gate 4 (architecture lint): PASSED.
5. **`cargo test --workspace`**:
   - Total workspace tests executed: 48 passed, 0 failed, 0 ignored.
     - `lc_core` (commands_test): 5 passed
     - `lc_core` (round_trip): 1 passed
     - `lc_io` (unittests): 3 passed
     - `lc_io` (golden_test): 4 passed
     - `lc_shell` (unittests): 6 passed
     - `lc_shell` (console_test): 5 passed
     - `lc_shell` (project_panel_test): 4 passed
     - `lc_shell` (shell_test): 5 passed
     - `lc_ui_api` (registry_test): 1 passed
     - `lc_ui_api` (viewport_test): 5 passed
     - `xtask` (unittests): 9 passed

### Discrepancies vs `docs/PHASE_A_REPORT.md`
1. **Workspace Test Count**:
   - `docs/PHASE_A_REPORT.md` Section 6 claims: "Workspace Test Count: 47 tests passing across all crates."
   - Current actual count: 48 tests.
   - **Cause**: Commit `9fc81db` (merged in `b0eb5a7`) fixed the bottom dock layout constraint and added `test_bottom_dock_layout_and_persistence` in `crates/lc_shell/tests/shell_test.rs`. `docs/PHASE_A_REPORT.md` was generated at tag `contract-v0.1` (`0c9c400`) before this commit was merged.
2. **Git Commit Alignment**:
   - `docs/PHASE_A_REPORT.md` Section 1 lists T-0011 commit SHA as `b4cbcf4`, and was finalized at `0c9c400` (`contract-v0.1`).
   - `main` is at `b0eb5a7`, containing 2 additional commits (`9fc81db` and `b0eb5a7`) for the bottom-dock layout fix.
3. **Layout Documentation**:
   - `docs/03-ui-layout.md` and `docs/decisions/0002-ui-docking.md` were modified on `main` to document that the bottom dock sits strictly between the left and right docks (center column only), but `docs/PHASE_A_REPORT.md` has not yet incorporated this revision.
