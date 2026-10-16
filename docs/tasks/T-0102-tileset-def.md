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
Tileset tab (load PNG via `cx.assets().load_texture(ui.ctx(), &rel_path)`, grid size, padding, spacing, preview) with Commands for tileset def edits; Tile palette panel to pick tiles (single + rectangle).
## Acceptance criteria
- [ ] Add/edit/remove TilesetDef through Commands with revert tests; references to removed tilesets handled per data-model rules.
- [ ] Palette selection emits `EditorEvent::SetTileSelection(Some(TileSelection))` and is readable via `cx.tile_selection()`.
- [ ] Asset loading uses `cx.assets().load_texture` (no direct `std::fs` or `lc_io` in feature crate).

