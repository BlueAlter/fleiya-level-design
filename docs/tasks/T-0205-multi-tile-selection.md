---
id: T-0205
title: Multi-tile selection, stamp painting, and canvas copy/paste
status: todo
owner: -
role: tileset
scope: [crates/lc_feature_tileset, crates/lc_feature_level]
depends_on: [T-0190]
skills: [incremental-implementation, test-driven-development]
---

## Goal
Support selecting multi-tile rectangles in the tileset palette and stamping them onto tile layers, plus copy/pasting regions directly on the level canvas.

## Acceptance criteria
- [ ] Multi-tile drag selection in Tileset palette stores a grid of tile coordinates in `TileSelection`.
- [ ] Paint and Rect tools support stamping multi-tile selections with correct tiling/offset.
- [ ] Copy (Ctrl+C) and Paste (Ctrl+V) shortcuts on canvas copy selected tile/int-grid region into a clipboard buffer and stamp at cursor.
- [ ] Revert tests for multi-tile paint and paste operations assert byte-identical snapshots.

## Out of scope
- Cross-project tile copy/pasting.
- Non-rectangular brush masks.
