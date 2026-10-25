---
id: T-0101
title: Level tab: canvas host, pan/zoom, grid, layer render loop, tool dispatch
status: done
owner: agy-level
role: level
scope: [crates/lc_feature_level, crates/lc_canvas]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
DocumentView for Level: canvas with Viewport pan/zoom, grid overlay, iterate level layers calling registered LayerHandler::render, route input to the active CanvasTool,
floating-window helper in lc_canvas (constrained to canvas rect) that hosts InspectorSections.

## Acceptance criteria
- [x] Pan (middle mouse/space-drag), zoom to cursor, grid snap; viewport culling of off-screen layers.
- [x] Floating properties window: draggable, collapsible, pin, position remembered per tab and clamped on resize.
- [x] Works with a fake LayerHandler/CanvasTool in tests; no dependency on other feature crates.

## Notes & Test Evidence
- Implemented `LevelDocumentView` in `lc_feature_level::view` with `DocumentView` trait.
- Implemented interactive canvas rendering, pan/zoom, grid, checkerboard, toolbar overlay, and layer render loop in `lc_feature_level::canvas`.
- Implemented floating inspector window helper and state in `lc_canvas::widgets::floating_window`.
- Registered factory in `lc_feature_level::register`.
- Added unit and integration tests in `crates/lc_feature_level/tests/level_canvas_test.rs` and `crates/lc_canvas/src/widgets/floating_window.rs`.
- `cargo xtask check` passed all 4 gates (fmt, clippy with zero warnings, all workspace tests, architecture lint).
