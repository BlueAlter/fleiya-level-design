---
id: T-0104
title: Layer def tab and Layers panel
status: todo
owner: -
role: layers
scope: [crates/lc_feature_layers]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
LayerDef tab (kind, grid size, tileset link, IntGrid values) and Layers panel following the active level tab (visibility toggle, reorder, select active layer).
## Acceptance criteria
- [ ] Add/edit/remove/reorder via Commands with revert tests.
- [ ] Active layer exposed through EditorCtx selection; panel shows empty state with no active level.
