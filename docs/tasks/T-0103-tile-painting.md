---
id: T-0103
title: Tiles and IntGrid layer handlers with paint/erase/rect/fill/eyedropper
status: todo
owner: -
role: tileset
scope: [crates/lc_feature_tileset]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
LayerHandler for Tiles and IntGrid layers + CanvasTools: paint, erase, rectangle, flood fill, eyedropper, flip x/y. A stroke is one merged Command.
## Acceptance criteria
- [ ] `PaintTilesCommand`, `FillCommand`, `RectCommand` with apply/revert/merge tests.
- [ ] Flood fill is iterative (no recursion overflow) and tested on a 1024x1024 grid.
- [ ] Rendering uses viewport culling.
