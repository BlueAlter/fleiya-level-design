# Review of T-0010 (Independent Re-review)
Reviewer: independent-reviewer   Author: agy-1   Branch: agent/T-0010   Commit reviewed: 24fba7c

Verdict: CHANGES_REQUESTED

## Gates
- cargo xtask check: pass
  - fmt: clean (`cargo fmt --all -- --check`)
  - clippy: clean (`cargo clippy --workspace --all-targets -- -D warnings`)
  - tests: 50 passed across workspace
  - architecture lint: clean (no forbidden crate dependencies, no `std::fs` outside `lc_io`)

## Acceptance criteria evidence
- [ ] Create project, add world + level, open level tab (placeholder DocumentView), save, reload, tree identical — **Partially Met**.
  - In `crates/lc_shell/tests/project_panel_test.rs::test_project_panel_create_add_save_reload`, mutations are made directly to vectors rather than testing `ProjectPanel` widget generation.
  - Production defect: When "File > New" is selected (`shell_app.rs:273`), `self.project_path` is not reset to `None`, which causes subsequent saves to silently overwrite the previously opened project file. Furthermore, when `self.project_path` is `None`, calling "Save" silently marks the project clean without writing to disk.
- [ ] Double-click on an open document focuses it, does not duplicate it — **Partially Met**.
  - Double-clicking on a World header does not emit `OpenDocument` (it only toggles collapsing state); opening a world tab is only possible via right-click context menu.
- [ ] Drag a level to another depth group issues MoveLevel command and is undoable — **Partially Met**.
  - In `ProjectPanel::render_world` (`panel.rs:196-204`), depth groups are rendered only for depth `0` and depths that already have at least one level. Consequently, users cannot drag a level into any empty non-zero depth (e.g. depth -1, 1, or 2), because drop zones for unpopulated depths are not rendered.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — `ProjectPanel` uses `ActionQueue` to issue commands.
- [ ] IDs not indices; deletion/cascade handled; undo restores identical state — **FAIL**: `panel.rs:89` and `panel.rs:215` allocate IDs via `worlds.len() + 1` and `world.levels.len() + 1`. Deleting a world or level causes immediate ID collision on next creation. Level IDs must be unique project-wide.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — Verified.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — `ProjectPanel` implements `PanelView`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Proper tracing levels used.
- [ ] Egui API calls verified against the pinned version (no hallucinated methods) — `panel.rs:327-339` generates duplicate button labels ("Depth 1", "Depth -1") in the same popup, causing egui widget ID collisions.
- [ ] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — Test `project_panel_test.rs:34` introduces non-deterministic wall-clock `SystemTime::now()` violating `CONSTRAINTS.md`.

## Findings

### Critical (Must Fix)

1. **[Critical] Data Loss: New Project does not reset `project_path`** (`crates/lc_shell/src/shell_app.rs:272-280`)
   When the user clicks "File > New", `self.project_path` is not reset to `None`. If the user previously had a project open, pressing Ctrl+S or clicking Save will immediately overwrite that file with the empty untitled project.
   *Remediation*: Add `self.project_path = None;` inside the "New" handler.

2. **[Critical] Data Loss / Phantom Save: Saving without `project_path` silently clears dirty flag** (`crates/lc_shell/src/shell_app.rs:611-614`)
   In `process_commands_and_events()`, when handling `EditorEvent::SaveProject`, if `self.project_path` is `None`:
   `is_dirty` is cleared and the status bar displays "Project saved", but no file was written to disk.
   *Remediation*: Do not clear `is_dirty` when `project_path` is `None`; report an error status message.

3. **[Critical] ID Collision on World and Level Creation** (`crates/lc_shell/src/project_panel/panel.rs:89, 215, 321`)
   - `panel.rs:89`: `let next_id = (project.worlds.len() as u64) + 1;`
   - `panel.rs:215`: `let next_id = (world.levels.len() as u64) + 1;`
   Generating IDs using vector lengths causes collisions when entities are removed. Level IDs must be globally unique across all worlds.
   *Remediation*: Generate IDs using maximum ID across the project + 1:
   - For worlds: `project.worlds.iter().map(|w| w.id.0).max().unwrap_or(0) + 1`
   - For levels: `project.worlds.iter().flat_map(|w| &w.levels).map(|l| l.id.0).max().unwrap_or(0) + 1`

### Important (Should Fix)

4. **[Important] Duplicate Buttons and Widget ID Collision in Context Menu** (`crates/lc_shell/src/project_panel/panel.rs:327-339`)
   In the Level context menu, the "Move to Depth..." submenu iterates over `[level.world_depth - 1, level.world_depth + 1, 0, 1, -1]`. When `level.world_depth == 0`, it produces duplicate buttons with identical text, triggering egui widget ID collisions.
   *Remediation*: Deduplicate target depths and filter out current depth before rendering buttons.

5. **[Important] World Tree Items Do Not Support Double-Click to Open** (`crates/lc_shell/src/project_panel/panel.rs:190-205`)
   Double-clicking on a World header does not emit `DocKind::World`.
   *Remediation*: Add double-click handling to open World documents.

6. **[Important] Cannot Drag Level to Unpopulated Depth Groups** (`crates/lc_shell/src/project_panel/panel.rs:196-204`)
   Drop zones are only rendered for depths that already have at least one level.
   *Remediation*: Render candidate drop targets or a "+ Add Depth" drop slot.

7. **[Important] Non-Deterministic Wall-Clock in Test** (`crates/lc_shell/tests/project_panel_test.rs:34-40`)
   `test_project_panel_create_add_save_reload` uses `std::time::SystemTime::now()`, violating `CONSTRAINTS.md` line 25.
   *Remediation*: Use deterministic process-isolated paths (e.g. `lc_test_proj_{std::process::id()}`) and ensure clean teardown.

### Nit / Optional / FYI
8. **[Nit] Stale Selection on Document Deletion** (`crates/lc_shell/src/project_panel/panel.rs:42, 63`)
9. **[Nit] Status Event Overwrites Command Failure Messages** (`crates/lc_shell/src/project_panel/panel.rs:274`)
10. **[FYI] Self-Review Protocol Breach in Original Review** (`docs/reviews/T-0010-review.md:2`)

## Doubt pass
- Claim: `ActionQueue` internal deferral in `ProjectPanel::ui` prevents borrow checker conflicts with `cx.project()`.
- Evidence: Encapsulates mutation intent and dispatches cleanly at frame boundary.
- Resolution: Maintain `ActionQueue`, but fix the critical ID generation bugs and save/reset path bugs.
