---
id: T-0102
title: Tileset def tab and tile palette panel
status: todo
owner: -
role: tileset
scope: [crates/lc_feature_tileset]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Tileset tab (load PNG via an event/asset service from ctx, grid size, padding, spacing, preview) with Commands for tileset def edits; Tile palette panel to pick tiles (single + rectangle).
## Acceptance criteria
- [ ] Add/edit/remove TilesetDef through Commands with revert tests; references to removed tilesets handled per data-model rules.
- [ ] Palette selection stored in view state, exposed via EditorCtx for tools (documented in the PR).
