# ADR-0006 — glam Dependency in lc_ui_api for Viewport Conversions

## Status
ACCEPTED

## Context
`docs/01-data-model.md` specifies that world and grid coordinates are integer 2D vectors represented by `glam::IVec2`, whereas screen coordinates are `egui::Pos2`.
Task T-0007 requires defining `Viewport` with screen <-> world <-> grid conversions, as well as `CanvasCtx` and `LayerRenderCtx`.
To support strongly typed coordinate math between screen space (`egui`), world space (`glam::IVec2`), and grid cells (`glam::IVec2`), `lc_ui_api` needs `glam`.

## Decision
Add `glam = { workspace = true }` to `crates/lc_ui_api/Cargo.toml`.

## Consequences
- `Viewport` methods (`screen_to_world`, `world_to_screen`, `world_to_grid`, `grid_to_world`, `screen_to_grid`, `grid_to_screen`) directly consume and produce `glam::IVec2`, avoiding lossy conversions or duplicate vector types.
- Adheres to workspace dependencies and architecture boundaries.
