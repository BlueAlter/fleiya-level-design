---
id: T-0203
title: Dock interaction polish: bottom dock height and drag/drop affordances
status: in_review
owner: deepseek-v4.1-flash-free
role: shell
scope: [crates/lc_shell/src/dock_panels.rs, crates/lc_shell/src/dock_layout.rs, crates/lc_shell/src/shell_app.rs, crates/lc_shell/tests/shell_test.rs, crates/lc_shell/tests/shell_interaction_test.rs, docs/tasks/T-0203-dock-interaction-polish.md, docs/MANUAL_CHECKS.md, docs/decisions/0002-ui-docking.md]
depends_on: [T-0008, T-0201]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Make the custom panel docks (left, right, bottom) behave predictably when the
bottom console is expanded and make panel-tab drag and drop legible. The central
`egui_dock` document area keeps its existing behaviour and styling.

## Acceptance criteria
- [x] Expanding the bottom dock after a collapse restores its remembered height
  instead of expanding to the full central area.
- [x] The expanded bottom dock is clamped to 55% of the central area, including
  after a stale full-height `PanelState` or a manual resize.
- [x] While dragging a panel tab, a floating tab preview follows the cursor and
  uses the editor design tokens.
- [x] Hovering a valid dock target shows an accent insertion line between tabs
  and a subtle highlight on the target slot.
- [x] Dropping inside a tab strip inserts at the indicated index; dropping on
  the slot content appends. Same-slot reordering works.
- [x] Dropping on the central document area is rejected and cancels the drag.
  `Esc` also cancels the drag.
- [x] Headless tests cover the height clamp, stale panel state, drop-target
  hit-testing, insertion indexes and same-slot/cross-slot moves.
- [x] `cargo xtask check` is green.

## Out of scope
- Split panes inside a panel dock (above/below/left/right sub-docks).
- Drag previews for central `egui_dock` document tabs.
- Persisting dock layout across application restarts.
- Empty dock slots that are hidden when they have no panels.
- New dependencies or changes to `lc_core`, `lc_io` or `lc_ui_api`.

## Notes
- The custom dock slots are tab strips, not split trees. The drop indicator
  therefore shows an insertion point, not a split-pane overlay.
- The central document area remains the domain of `egui_dock`; panel tabs can
  never be dropped there.
- The bottom dock height is owned by `BottomDockState`; `egui`'s `PanelState`
  is only a rendering cache and must be normalised when it is stale.
- All visuals use `lc_canvas::theme` tokens: accent insertion line, tinted slot
  highlight, zero corner radius and hairline borders.

## Implementation notes
- `BottomDockState` now owns the height policy: `MIN_EXPANDED_HEIGHT = 96`,
  `MAX_HEIGHT_FRACTION = 0.55`, `max_expanded_height` and
  `desired_expanded_height`. `render_bottom_dock` sets `min_size`/`max_size`,
  normalises a stale `PanelState` (below the collapsed header or above the
  current maximum) and clamps the remembered height after every frame.
- `DockSlotState::insert_panel` and `ShellDockLayout::move_panel_to` /
  `drop_dragged_at` implement explicit insertion indexes. Moving a panel within
  its own slot is a no-op when the index does not change.
- `DockGeometry`/`DockSlotGeometry` collect each slot rect, tab-strip rect and
  tab rect during rendering. `handle_drag_and_drop` then centralises hit-testing,
  the floating ghost, the accent insertion line, the slot highlight and the
  release/Esc handling. Dropping on content appends; dropping on the central
  area returns no target and cancels.
- The ghost is an `egui::Area` at `Order::Tooltip`; the indicator is painted in
  a `Order::Foreground` layer so it stays above panels but below the ghost.

## Tests
- `dock_layout::tests::insert_panel_reorders_within_a_slot`
- `dock_layout::tests::move_panel_to_index_across_slots`
- `dock_layout::tests::moving_to_the_same_position_is_a_no_op`
- `dock_layout::tests::bottom_dock_height_is_clamped_to_available_space`
- `dock_panels::tests::insertion_index_uses_tab_centres_and_appends_outside_the_strip`
- `dock_panels::tests::drop_target_finds_the_hovered_slot_and_rejects_the_central_area`
- `shell_test::test_bottom_dock_expand_restores_height_and_clamps_stale_state`
- `shell_interaction_test::test_panel_tab_reorder_and_insert_at_index`
- `shell_interaction_test::test_escape_cancels_panel_drag`
- Existing `shell_interaction_test::test_panel_tab_drag_and_drop_between_docks`
  and `test_drag_cancelled_when_released_outside_dock` continue to pass.

## Visual verification
Headless Xvfb screenshots inspected during implementation:
`/tmp/opencode/dock_reexpanded.png` (height restored),
`/tmp/opencode/dock_drag_ghost2.png` (ghost + right-slot highlight),
`/tmp/opencode/dock_after_drop_right.png` (cross-slot move),
`/tmp/opencode/dock_reorder_indicator.png` (insertion line),
`/tmp/opencode/dock_central_nodrop.png` (central drop rejected) and
`/tmp/opencode/dock_max_height4.png` (manual resize capped at 55%).

## Gate
`cargo xtask check` — fmt, clippy `-D warnings`, all tests and architecture lint
pass.
