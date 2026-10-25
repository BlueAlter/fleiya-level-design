# Review of T-0104
Reviewer: self   Author: agy-layers   Branch: agent/T-0104   Commit reviewed: HEAD

Verdict: APPROVE

## Gates
- cargo xtask check: pass
  - fmt: clean
  - clippy: clean (-D warnings)
  - tests: 5 command unit tests, 7 integration tests passing in `lc_feature_layers`; all workspace tests passing
  - architecture lint: clean (zero cross-feature dependencies, zero `std::fs` or `lc_io` outside permitted crates)

## Acceptance criteria evidence
- [x] Add/edit/remove/reorder via Commands with revert tests:
  - Implemented in `crates/lc_feature_layers/src/commands.rs`:
    - `AddLayerDefCommand`: appends `LayerDef` to project definitions and instantiates matching `LayerInstance` in each level; revert cleans up both definitions and level instances at their exact indices.
    - `UpdateLayerDefCommand`: updates attributes, merges consecutive edits for smooth undo, and reverts cleanly.
    - `RemoveLayerDefCommand`: removes `LayerDef` and cascades deletion to all level `LayerInstance`s; revert restores definition and level instances at their original positions.
    - `ReorderLayerDefsCommand`: reorders project definitions and synchronizes the render order in all level instances; revert restores previous order.
    - `ToggleLayerVisibilityCommand`: toggles `visible` on a level layer instance; revert restores previous visibility.
  - Verified by 5 unit tests in `crates/lc_feature_layers/src/commands.rs::tests` and integration revert tests in `crates/lc_feature_layers/tests/layers_test.rs`.
- [x] Active level read via `cx.active_level()`; panel displays empty state when `cx.active_level()` is `None`:
  - Implemented in `crates/lc_feature_layers/src/panel.rs`:
    - Checks `cx.active_level()`; when `None` (or when active level is not in the project), renders centered informational empty state ("No active level. Open a level tab to inspect and manage its layers.").
    - When `Some(level_id)`, retrieves level and displays its layer hierarchy with reordering, visibility toggle, kind badges, and "+ Add Layer" menu.
  - Verified by headless test `test_layers_panel_empty_state_when_no_active_level` in `crates/lc_feature_layers/tests/layers_test.rs`.
- [x] Layer selection updates session state via `EditorEvent::SetActiveLayer(Some(id))` and is readable via `cx.active_layer()`:
  - Implemented in `crates/lc_feature_layers/src/panel.rs`:
    - Clicking any layer row emits `EditorEvent::SetActiveLayer(Some(layer.def_id))`.
    - Double-clicking opens the layer definition tab (`DocKind::LayerDef`).
    - Active layer is highlighted with `t.selection_fill` and `t.accent` stroke.
  - Verified by headless test `test_layers_panel_with_active_level_and_selection` in `crates/lc_feature_layers/tests/layers_test.rs`.

## Consistency checklist (see docs/04-multi-agent-workflow.md)
- [x] Mutations only via Command; Commands have apply/revert/merge tests — All layer mutations go through `lc_core::command::Command` implementations with apply, revert, and merge test coverage.
- [x] IDs not indices; deletion/cascade handled; undo restores identical state — `LayerDefId` typed IDs used; cascading removal cleanly clears and restores all dependent instances in all worlds and levels.
- [x] No egui in core; no fs outside io; no feature-to-feature dependency; no new dependency without ADR — Verified by gate 4 architecture linter.
- [x] New view uses `DocumentView` / `PanelView` / `InspectorSection` / `LayerHandler` as defined; registered through `register()` — `LayerDefDocumentView` implements `DocumentView` and `LayersPanel` implements `PanelView`; registered via `lc_feature_layers::register`.
- [x] Naming matches neighbouring crates; logging via tracing with correct levels; no `unwrap` in core/io — Standard naming and error handling.
- [x] Egui API calls verified against the pinned version (no hallucinated methods) — Verified with egui 0.36.
- [x] No silenced lint, skipped test, or loosened gate. Diff within scope and size guideline — Changes strictly isolated to `crates/lc_feature_layers`, `docs/tasks/T-0104-layer-defs.md`, `docs/MANUAL_CHECKS.md`, and this review.
- [x] Acceptance criteria each have evidence — Documented above.

## Findings
1. [Important] Type aliases `InsertedLevelLayer` and `RemovedLayerInstance` were defined to keep tuple complexity clean under `-D clippy::type_complexity`.
2. [Nit] Render order is represented from top-to-bottom in the Layers panel (highest render index at top, matching canvas visual stack and LDtk/Tiled conventions). The "▲ Move Up" button moves layers towards higher render index (on top of lower layers).
