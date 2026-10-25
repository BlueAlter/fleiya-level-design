---
id: T-0104
title: Layer def tab and Layers panel
status: done
owner: agy-layers
role: layers
scope: [crates/lc_feature_layers]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
LayerDef tab (kind, grid size, tileset link, IntGrid values) and Layers panel following the active level tab (visibility toggle, reorder, select active layer).
## Acceptance criteria
- [x] Add/edit/remove/reorder via Commands with revert tests.
- [x] Active level read via `cx.active_level()`; panel displays empty state when `cx.active_level()` is `None`.
- [x] Layer selection updates session state via `EditorEvent::SetActiveLayer(Some(id))` and is readable via `cx.active_layer()`.
