---
id: T-0102
title: Tileset def tab and tile palette panel
status: done
owner: agy-tileset
role: tileset
scope: [crates/lc_feature_tileset]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Tileset tab (load PNG via `cx.assets().load_texture(ui.ctx(), &rel_path)`, grid size, padding, spacing, preview) with Commands for tileset def edits; Tile palette panel to pick tiles (single + rectangle).
## Acceptance criteria
- [x] Add/edit/remove TilesetDef through Commands with revert tests; references to removed tilesets handled per data-model rules.
- [x] Palette selection emits `EditorEvent::SetTileSelection(Some(TileSelection))` and is readable via `cx.tile_selection()`.
- [x] Asset loading uses `cx.assets().load_texture` (no direct `std::fs` or `lc_io` in feature crate).

## Notes & Test Evidence
- Implemented `AddTilesetDefCommand`, `UpdateTilesetDefCommand`, and `RemoveTilesetDefCommand` with cascading deletion and full revert in `crates/lc_feature_tileset/src/commands.rs`.
- Implemented `TilesetDocumentView` for `DocKind::TilesetDef(TilesetDefId)` with properties editor and grid slicing preview in `crates/lc_feature_tileset/src/view.rs`.
- Implemented `TilePalettePanel` with single/rectangular tile picking, texture loading via `AssetService`, and event dispatch in `crates/lc_feature_tileset/src/palette.rs`.
- Registered factory and panel in `crates/lc_feature_tileset/src/lib.rs`.
- Added unit and headless integration tests covering all commands, views, and event dispatch in `crates/lc_feature_tileset/tests/tileset_test.rs`.
- `cargo xtask check` passed all 4 gates cleanly.

