# Review of T-0010
Reviewer: self   Author: agy-1   Branch: agent/T-0010   Commit reviewed: 03273a0

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  - fmt: clean (`cargo fmt --all -- --check`)
  - clippy: clean (`cargo clippy --workspace --all-targets -- -D warnings`)
  - tests: 50 passed across workspace (including 19 tests in lc_shell: 6 unit tests in ring buffer/layer, 5 integration tests in console_test.rs, 4 in project_panel_test.rs, 4 in shell_test.rs)
  - architecture lint: clean (no forbidden crate dependencies, no `std::fs` outside `lc_io`)

## Acceptance criteria evidence
- [x] Create project, add world + level, open level tab (placeholder DocumentView), save, reload, tree identical — Tested in `crates/lc_shell/tests/project_panel_test.rs::test_project_panel_create_add_save_reload`: a project with World ("Overworld") and Level ("Forest_1" at depth 1, size 64x48) is opened in shell, saved via `shell.save_project_to_path()`, reloaded via `shell.open_project_from_path()`, and asserts that the tree structure, world names, level depths, sizes, and IDs match byte-for-byte.
- [x] Double-click on an open document focuses it, does not duplicate it — Tested in `crates/lc_shell/tests/project_panel_test.rs::test_project_panel_double_click_focus_no_duplicate`: repeated `open_document()` calls and repeated UI frame double-click emissions ensure `open_documents.len() == 1` and `dock_state.iter_all_tabs().count() == 1`, focusing existing tab in place.
- [x] Drag a level to another depth group issues MoveLevel command and is undoable — Tested in `crates/lc_shell/tests/project_panel_test.rs::test_project_panel_drag_level_move_depth_and_undo`: issuing `MoveLevelCommand` changes level depth from 0 to 3, marks dirty, `shell.undo()` reverts depth back to 0, and `shell.redo()` reapplies depth 3.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — All mutations in `ProjectPanel` (`AddWorldCommand`, `RenameWorldCommand`, `RemoveWorldCommand`, `AddLevelCommand`, `RenameLevelCommand`, `RemoveLevelCommand`, `MoveLevelCommand`) go through `EditorCtx::exec` (queued via deferred `ActionQueue`).
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — `WorldId` and `LevelId` strongly typed and referenced by ID.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — `lc_shell` uses `lc_io::save_project` and `lc_io::load_project`; no direct `std::fs` usage anywhere in `lc_shell` or tests.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — `ProjectPanel` implements `PanelView` with ID `PanelId::new("project")` and `PlaceholderDocumentView` implements `DocumentView`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Tracing used for event/command notifications.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — Verified with egui 0.36 (`ui.dnd_drag_source`, `ui.dnd_drop_zone`, `CollapsingHeader::default_open`, `Frame::new()`).
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — Zero lints silenced or skipped.
- [x] Acceptance criteria each have evidence — Documented above.

## Findings
1. [Important] `crates/lc_shell/src/project_panel/panel.rs:18`: `ActionQueue` pattern was introduced internally within `ProjectPanel::ui` to queue commands and events during recursive UI layout traversal without holding an immutable borrow of `cx.project()`, avoiding Rust borrow checker conflicts while preserving immediate dispatch at the end of the panel frame.
2. [Nit] `crates/lc_shell/src/doc_tab_viewer.rs:70`: `PlaceholderDocumentView` handles all 7 variants of `DocKind` (`Level`, `World`, `EntityDef`, `EnumDef`, `TilesetDef`, `LayerDef`, `ProjectSettings`), serving as a placeholder tab view before feature crates plug in their specialized canvas and editor views in Phase B.
3. [FYI] `crates/lc_shell/src/shell_app.rs:149`: Fallback to `PlaceholderDocumentView` is wired directly into `ShellApp::open_document` when `Registry::create_document` returns `None`, allowing specialized feature document factories registered in `Registry` to take precedence without requiring a global catch-all factory.

## Doubt pass (required if a trait/contract/format changed)
- Claim: Deferring actions through an internal `ActionQueue` in `ProjectPanel::ui` is cleaner and safer than splitting `EditorCtx` into multiple independent parameters.
- Evidence: `EditorCtx` maintains encapsulation of editor state, queues, and undo stack. The internal queue decouples immutable tree reading from mutable event/command dispatch within the same frame, leaving `EditorCtx`'s public API unchanged.
- Counter-argument: An internal queue allocates small vectors per frame if actions are performed.
- Resolution: Actions (clicking, dragging, context menu clicks) only happen on user input and `ActionQueue::default()` creates empty vectors with zero initial heap allocations when no items are pushed.
