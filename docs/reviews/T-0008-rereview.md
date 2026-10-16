# Review of T-0008 (Independent Re-review)
Reviewer: adversarial-reviewer   Author: agy-1   Branch: main (fix/contract-v0.2)   Commit reviewed: 55c3d64 (with fixes 9fc81db / b0eb5a7)

Verdict: CHANGES_REQUESTED

## Gates
- cargo xtask check: pass
  - fmt: clean (`cargo fmt --all -- --check` passed)
  - clippy: clean (`cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings)
  - tests: 34 passed (including 5 shell tests in `crates/lc_shell/tests/shell_test.rs`)
  - architecture lint: clean (crate dependency matrix in `docs/02-architecture.md` satisfied, forbidden filesystem / cross-crate imports clean)

## Acceptance criteria evidence
- [x] ADR-0002 updated with the spike result and chosen approach — `docs/decisions/0002-ui-docking.md` documents `egui_dock` 0.21.1 spike findings (cannot restrict tab drops by type, lack of native collapsible dock nodes), establishing the dual-container architecture (`egui_dock::DockArea` for central documents and `ShellDockLayout` for side/bottom panels).
- [ ] Panels drag/move between left/right/bottom; documents cannot enter docks and vice versa — **Partially Met**. The separation between documents and docks is structurally guaranteed by the type system and container boundaries (`DocKind` vs `PanelId`). However, panels cannot be dragged (mouse drag-and-drop is absent; only right-click context menu relocation is implemented), tab reordering within a dock slot is impossible, and moving an active panel causes stale rendering in two docks simultaneously on the same frame.
- [ ] Bottom dock collapses/expands via header click and Ctrl+`; state persists across restarts — **Partially Met**. Bottom dock collapse/expand via header click, tab click, and `Ctrl+Backtick` is implemented and verified. However, state persistence across restarts has no production implementation in `lc_app` or `lc_shell`.
- [ ] Tabs close when `is_valid` returns false. Dirty `*` indicator works — **Partially Met**. Tabs close when `DocumentView::is_valid` returns false (verified in `test_dirty_indicator_and_automatic_tab_close_on_invalid`). However, the dirty indicator appends `*` to every open tab indiscriminately whenever any command executes anywhere in the project, and `UndoStack` has no clean-revision tracking so undoing commands does not restore clean state.
- [x] Manual verification checklist in the PR (egui UI), plus kittest tests where feasible — `docs/MANUAL_CHECKS.md` contains 37 checklist items covering interactive shell, menu, docks, and status bar behavior. Automated headless egui tests are implemented in `crates/lc_shell/tests/shell_test.rs`.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — Project mutations route through `pending_commands` and `UndoStack::execute`.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — `DocKind` references stable IDs (`LevelId`, `WorldId`, def IDs).
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — Workspace lint and `cargo xtask check` arch-lint verify this.
- [ ] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — While views implement the traits, newly registered panels via `Registry::register_panel` are invisible unless hardcoded in `ShellDockLayout::default()`, because `ensure_panel()` is never called.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Tracing logs command execution and IO errors; no `unwrap()`, `expect()`, or `panic!` outside tests in `lc_shell`.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — Uses egui 0.36.0 panels and egui_dock 0.21.1.
- [ ] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — **Failed**: `crates/lc_shell/src/shell_app.rs` is 620 lines, violating `CONSTRAINTS.md` line 31 ("Modules > 500 lines get split").
- [ ] Acceptance criteria each have evidence — **Failed**: State persistence across restarts has no production implementation, and panel drag-and-drop is missing.

## Findings

### Critical (Must Fix — Data Loss & Unmet Acceptance Criteria)

1. **[Critical] Silent Data Loss on `SaveProject` when `project_path` is `None`** (`crates/lc_shell/src/shell_app.rs:611-614`)
   When a user creates a new project and triggers `EditorEvent::SaveProject`, `self.project_path` is `None`. Instead of warning the user or prompting for a path, the handler resets `is_dirty = false` and displays `"Project saved"` without writing a single byte to disk. Closing the application destroys all work.
   *Remediation*: When `project_path` is `None`, do not clear `is_dirty`; set an error status message indicating no project path is configured.

2. **[Critical] Catastrophic File Overwrite on `File > New` Followed by Save** (`crates/lc_shell/src/shell_app.rs:272-280`)
   When `File > New` is clicked, `self.project` is replaced with `Project::new("Untitled")`, but `self.project_path` is never cleared to `None`. If the user previously had a project open on disk, subsequent saves immediately overwrite that file with the untitled project without confirmation.
   *Remediation*: Clear `self.project_path = None;` on `File > New`.

3. **[Critical] Layout State Persistence Across Restarts Unimplemented** (`crates/lc_shell/src/shell_app.rs:86`, `crates/lc_app/src/main.rs:8-12`)
   AC 3 requires layout persistence across restarts. `ShellDockLayout` derives `Serialize` and `Deserialize`, but neither `lc_shell` nor `lc_app` loads or saves layout state to disk or application storage on startup/shutdown.
   *Remediation*: Implement layout save/load using RON or JSON in user config directory, or via eframe storage.

### Important (Should Fix)

4. **[Important] Stale Active Panel State Causes Duplicate Widget Rendering and ID Clashes on Move** (`crates/lc_shell/src/shell_app.rs:471-484, 541-551`)
   In `render_dock_slot`, `active` panel ID is captured before rendering tabs. If a context menu relocation runs during tab rendering, line 541 renders the stale active panel in the source dock, while the target dock also renders it, causing duplicate egui widget IDs on that frame.
   *Remediation*: Re-query `self.dock_layout` for the active panel ID after rendering the tab strip before rendering content.

5. **[Important] Module Size Limit Violated (`shell_app.rs` is 620 lines)** (`crates/lc_shell/src/shell_app.rs:1-620`)
   `CONSTRAINTS.md` line 31 mandates: *"Modules > 500 lines get split."*
   *Remediation*: Extract `menu_bar.rs` and `status_bar.rs` into separate modules in `lc_shell`.

6. **[Important] Global `is_dirty` Marks All Open Tabs with `*` Indiscriminately** (`crates/lc_shell/src/doc_tab_viewer.rs:36-40`)
   `DocTabViewer::title` appends `*` whenever `self.is_dirty` is true, marking all open document tabs modified regardless of which document was edited.
   *Remediation*: Document document-level dirty state behavior and align with `UndoStack`.

7. **[Important] `File > Open` Menu Item Missing from Menu Bar** (`crates/lc_shell/src/shell_app.rs:270-285`)
   Task T-0008 requires `File: New/Open/Save`. `Open` was omitted from the menu bar.
   *Remediation*: Add `File > Open` menu action.

8. **[Important] Dynamic Panel Registration Disconnected from Dock Layout** (`crates/lc_shell/src/dock_layout.rs:122-140`, `crates/lc_shell/src/shell_app.rs:192-196`)
   Panels registered dynamically via `Registry::register_panel` are invisible unless `ensure_panel()` is called during shell initialization.
   *Remediation*: Call `ensure_panel` for all registered panels on startup and during layout reset.

9. **[Important] Status Bar Cursor/Zoom/Tool Metrics Stranded from Canvas Views** (`crates/lc_shell/src/shell_app.rs:41-46, 334-363`)
   Metrics in the status bar are default placeholders with no communication contract from document canvas views.
   *Remediation*: Expose status metrics through session state or `EditorEvent`.

### Nit / FYI
10. **[Nit] Panel Drag-and-Drop Not Implemented** (`docs/decisions/0002-ui-docking.md:33`)
    Context menu relocation is implemented, but tab mouse dragging is absent.
11. **[Nit] Dead Code in `render_dock_slot` for `DockSlotLocation::Bottom`** (`crates/lc_shell/src/shell_app.rs:480-483`)
    Bottom dock is handled exclusively in `render_bottom_dock`.
12. **[Nit] Non-Deterministic Tab Ordering on `reset_layout`** (`crates/lc_shell/src/shell_app.rs:194`)
    `open_documents.keys()` hash iteration produces random order. Use sorted keys.

## Doubt pass
- Claim: Hybrid docking architecture is preferable to forcing everything into egui_dock.
- Evidence: Prevents tool panels from being dragged into central document areas, and enables collapsible bottom tray.
- Counter-argument: Requires custom panel strip and relocation controls.
- Resolution: Maintain hybrid architecture, but fix the critical state bugs (data loss on save, overwrite on new, stale active ID on move) and split `shell_app.rs` to satisfy module size constraint.
