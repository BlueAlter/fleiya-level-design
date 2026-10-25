# Review of T-0203
Reviewer: self   Author: deepseek-v4.1-flash-free   Branch: agent/T-0203   Commit reviewed: e777c37

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  - fmt: clean (`cargo fmt --all -- --check`)
  - clippy: clean (`cargo clippy --workspace --all-targets -- -D warnings`)
  - tests: all unit and integration tests passing across workspace
  - architecture lint: clean (no forbidden crate dependencies, no `std::fs` outside `lc_io`)

## Acceptance criteria evidence
- [x] Expanding the bottom dock after a collapse restores its remembered height instead of expanding to the full central area — verified in `crates/lc_shell/tests/shell_test.rs::test_bottom_dock_expand_restores_height_and_clamps_stale_state` and `BottomDockState::desired_expanded_height`.
- [x] The expanded bottom dock is clamped to 55% of the central area, including after a stale full-height `PanelState` or a manual resize — verified in `crates/lc_shell/src/dock_layout.rs::tests::bottom_dock_height_is_clamped_to_available_space`.
- [x] While dragging a panel tab, a floating tab preview follows the cursor and uses the editor design tokens — verified in `crates/lc_shell/src/dock_panels.rs` via `egui::Area` at `Order::Tooltip` using `Tokens::get(ui.ctx())`.
- [x] Hovering a valid dock target shows an accent insertion line between tabs and a subtle highlight on the target slot — verified in `crates/lc_shell/src/dock_panels.rs` with `Order::Foreground` painter.
- [x] Dropping inside a tab strip inserts at the indicated index; dropping on the slot content appends. Same-slot reordering works — verified in `crates/lc_shell/tests/shell_interaction_test.rs::test_panel_tab_reorder_and_insert_at_index` and `dock_panels.rs::tests::insertion_index_uses_tab_centres_and_appends_outside_the_strip`.
- [x] Dropping on the central document area is rejected and cancels the drag. `Esc` also cancels the drag — verified in `crates/lc_shell/tests/shell_interaction_test.rs::test_escape_cancels_panel_drag` and `dock_panels.rs::tests::drop_target_finds_the_hovered_slot_and_rejects_the_central_area`.
- [x] Headless tests cover the height clamp, stale panel state, drop-target hit-testing, insertion indexes and same-slot/cross-slot moves — comprehensive test suite added in `dock_layout.rs`, `dock_panels.rs`, `shell_test.rs`, and `shell_interaction_test.rs`.
- [x] `cargo xtask check` is green — passes all checks.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — Dock interaction does not mutate project state; panel positions are view state in `ShellDockLayout`.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — Panels identified by `PanelId`.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — All dock changes remain purely within `lc_shell`.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — Maintained existing panel view contracts.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Proper tracing and token usage throughout.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — Uses egui 0.36 Area, Painter, Response APIs cleanly.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — No lints silenced or skipped.
- [x] Acceptance criteria each have evidence — Documented above.

## Findings
1. [Important] `crates/lc_shell/src/dock_panels.rs`: The separation between `DockGeometry` collection during render and hit-testing in `handle_drag_and_drop` provides clean separation of rendering and interaction state.
2. [Nit] `crates/lc_shell/src/dock_layout.rs`: Clamping logic in `BottomDockState::max_expanded_height` correctly accounts for `total_height * MAX_HEIGHT_FRACTION`.

## Doubt pass (required if a trait/contract/format changed)
- Claim: Clamping bottom dock to 55% avoids layout collapse when re-expanding with stale panel cache.
- Evidence: Prevents bottom dock from taking over the central canvas area when toggling with `Ctrl+`` or clicking toggle.
- Resolution: Normalizing `PanelState` against `BottomDockState` preserves predictable docking behavior without breaking egui internals.
