---
id: T-0108
title: World tab: level placement, depth selector, layouts, multi-world
status: done
owner: opencode
role: world
scope: [crates/lc_feature_world]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
World DocumentView: level rectangles at world_pos, drag with snap, layouts (Free, GridVania, LinearH, LinearV), depth selector (active depth full color, others dimmed),
create level from the world view, duplicate level, move depth. Multiple worlds per project, each its own tab.
## Acceptance criteria
- [x] Levels overlapping at different depths render correctly; only active depth is interactive.
  Evidence: `only_the_active_depth_is_interactive`, `layout::tests::pick_prefers_the_topmost_level_at_the_active_depth`, `ui_renders_overlapping_depths_and_empty_worlds`.
- [x] All moves/creations are Commands (undoable); double-click level emits OpenDocument(Level).
  Evidence: `dragging_a_level_emits_one_snapped_move_command`, `linear_layout_drag_reorders_all_levels_in_one_command`, `double_click_opens_the_level_document`,
  `commands::tests::*` (apply/revert/merge round trips), `WorldScopedCommand` keeps the world tab focused.
- [x] Layout mode behaviors have unit tests (snap math, linear arrangement).
  Evidence: `layout::tests::snap_scalar_rounds_on_both_sides_of_zero`, `free_layout_snaps_only_when_enabled`, `grid_vania_always_snaps`,
  `linear_layouts_lock_the_cross_axis`, `arrange_linear_uses_size_and_spacing`, `linear_order_ranks_the_dragged_level_by_its_preview`.

## Notes
- Commands added in `crates/lc_feature_world/src/commands.rs`: `DuplicateLevelCommand`, `SetLevelPositionsCommand` (batch/linear reorder in one undo step),
  `SetWorldLayoutCommand`, and `WorldScopedCommand` (wraps core `AddLevelCommand`/`MoveLevelCommand` so the world tab stays focused).
- Level creation, duplication and depth moves allocate ids with `helpers::next_level_id` (`max(live id) + 1`), matching the project panel. The reuse window is
  logged in `docs/QUESTIONS.md` (T-0107 entry) and remains a contract follow-up.
- One-line shell wiring (`lc_feature_world::register`) is documented in `docs/QUESTIONS.md` because the task scope is the feature crate only.
- View state (camera, active depth, selection, drag) is transient and never serialized; the project remains the single source of truth.
