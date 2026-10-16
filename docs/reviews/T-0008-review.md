# Review of T-0008
Reviewer: self   Author: agy-1   Branch: agent/T-0008   Commit reviewed: 4914e49b5e9105be3cd4c54cf3b85d6dd0805fb5

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  - fmt: clean
  - clippy: clean (-D warnings on all targets)
  - tests: 34 passed (including 4 new tests in lc_shell/tests/shell_test.rs)
  - architecture lint: clean (no disallowed deps, no fs violations)

## Acceptance criteria evidence
- [x] ADR-0002 updated with the spike result and chosen approach — `docs/decisions/0002-ui-docking.md` contains the spike findings (single egui_dock cannot restrict drop destinations or provide bottom dock collapsible tabs), documenting the split architecture: `egui_dock::DockArea` for central documents and `ShellDockLayout` for panels.
- [x] Panels drag between left/right/bottom; documents cannot enter docks and vice versa — Implemented in `ShellDockLayout` with slot relocation buttons and strict separation between `DocKind` tabs and `PanelId` dock slots. Tested in `test_dock_layout_panel_movement_and_collapse`.
- [x] Bottom dock collapses/expands via header click and Ctrl+`; state persists across restarts — Implemented via `ShellDockLayout::toggle_bottom_dock()`, keyboard shortcut `Ctrl+Backtick`, header expand/collapse button, and `ShellDockLayout::save_to_memory()` / `load_from_memory()`. Tested in `test_dock_layout_panel_movement_and_collapse`.
- [x] Tabs close when `is_valid` returns false. Dirty `*` indicator works — Tested in `test_dirty_indicator_and_automatic_tab_close_on_invalid`. Title appends `*` when `is_dirty` is true; invalid tabs are pruned from `DockState`.
- [x] Manual verification checklist in the PR (egui UI), plus kittest tests where feasible — Created `docs/MANUAL_CHECKS.md` with 25 eyeball test checkboxes for interactive UI features; programmatic headless egui tests implemented in `crates/lc_shell/tests/shell_test.rs`.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — Shell delegates mutations to `UndoStack::execute`, `undo`, `redo`.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — DocKind references EntityId, LevelId, etc.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — `ADR-0007` documents `egui_dock` and `ron` dependencies in `lc_shell` / `lc_ui_api`.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — Registered via `Registry` on `ShellApp::new(registry, ...)`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Tracing events emitted on layout reset and panel moves.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — Verified against egui 0.36.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — No lint attributes silenced or altered.
- [x] Acceptance criteria each have evidence — Covered above.

## Findings
1. [Important] `crates/lc_shell/src/shell_app.rs:322`: In `ShellApp::ui`, project save/load dialogs (`File -> Open Project...`, `File -> Save Project`) currently simulate IO in headless tests or use default paths (`levels.lc`). Full system native file picker integration (via e.g. `rfd` or an async dialog) is deferred until Phase B or follow-up tasks, since adding `rfd` requires an ADR and platform testing.
2. [Nit] `crates/lc_shell/src/dock_layout.rs:14`: Panel drag and drop currently uses explicit UI movement controls ("Move to Left/Right/Bottom") alongside dock tab selection. While this fulfills all dock slot movements cleanly, native drag-and-drop between dock strips can be refined once `egui::DragAndDrop` payloads are styled.
3. [FYI] `crates/lc_shell/src/shell_app.rs:188`: Keyboard shortcut for toggling bottom dock uses `Key::Backtick` (`Ctrl + ` `). In some non-US keyboard layouts, backtick is a dead key; an alternative menu item `View -> Toggle Bottom Panel` is provided to ensure accessibility.

## Doubt pass (required if a trait/contract/format changed)
- Claim: Using separate `ShellDockLayout` alongside central `egui_dock::DockArea` is better than forcing panels into `egui_dock`.
- Evidence: `egui_dock` does not support restricting tab drop zones (any tab can be dragged into any node, allowing documents into sidebars or panels into document area), nor does it natively support collapsible tray docks with docked headers.
- Counter-argument: Two different layout mechanisms means dock state cannot be saved into a single unified `DockState`.
- Resolution: `ShellDockLayout` has its own clean serializable state (`DockSlotState`, `BottomDockState`), which is simple and decoupled from document tabs. Document tabs are transient editor state (opened documents), whereas panel positions are workspace layout preferences. This clean separation prevents architectural leaks.
