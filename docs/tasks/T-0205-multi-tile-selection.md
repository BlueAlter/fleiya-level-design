---
id: T-0205
title: Multi-tile selection, stamp painting, and canvas copy/paste
status: done
owner: opencode
role: tileset
scope: [crates/lc_feature_tileset, crates/lc_feature_level]
depends_on: [T-0190]
skills: [incremental-implementation, test-driven-development]
---

## Goal
Support selecting multi-tile rectangles in the tileset palette and stamping them onto tile layers, plus copy/pasting regions directly on the level canvas.

## Acceptance criteria
- [x] Multi-tile drag selection in Tileset palette stores a grid of tile coordinates in `TileSelection`.
  Evidence: `test_tile_palette_panel_selection_events_and_cx_readability` asserts a 2x2 block (`tiles.len() == 4`, `width == 2`).
- [x] Paint and Rect tools support stamping multi-tile selections with correct tiling/offset.
  Evidence: `stamp_cells_are_offset_by_selection_width`, `test_rect_multi_tile_pattern_tiles_and_reverts_byte_identically`; `RectCommand` now accepts `FillValue::TilePattern` and repeats the block modulo its width/height.
- [x] Copy (Ctrl+C) and Paste (Ctrl+V) shortcuts on canvas copy selected tile/int-grid region into a clipboard buffer and stamp at cursor.
  Evidence: `test_canvas_copy_paste_shortcuts_emit_commands`, `copy_paste_tiles_round_trips_byte_identical`, `copy_paste_int_grid_round_trips_byte_identical`; canvas region selection is provided by the new `SelectTool`.
- [x] Revert tests for multi-tile paint and paste operations assert byte-identical snapshots.
  Evidence: `test_multi_tile_paint_revert_is_byte_identical`, `copy_paste_*_round_trips_byte_identical` compare full `Debug` snapshots before apply and after revert.

## Out of scope
- Cross-project tile copy/pasting.
- Non-rectangular brush masks.

## Notes
- `TileSelection` already supported rectangular blocks; this task wired it through Rect pattern stamping, canvas region selection, and the clipboard.
- Clipboard lives in `TileCanvasTool` as an in-memory buffer (no OS clipboard, no new runtime dependency) and is scoped to the level tab, matching other per-tab view state.
- Byte-identical snapshot assertions use the deterministic full `Debug` representation because the architecture lint restricts `serde_json` to `lc_io`.
- Manual desktop checks for selection/stamp/copy/paste are listed in `docs/MANUAL_CHECKS.md` (T-0205 section).
