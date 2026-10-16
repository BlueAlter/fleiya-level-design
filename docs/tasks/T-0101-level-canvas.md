---
id: T-0101
title: Level tab: canvas host, pan/zoom, grid, layer render loop, tool dispatch
status: todo
owner: -
role: level
scope: [crates/lc_feature_level, crates/lc_canvas]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
DocumentView for Level: canvas with Viewport pan/zoom, grid overlay, iterate level layers calling registered LayerHandler::render, route input to the active CanvasTool,
floating-window helper in lc_canvas (constrained to canvas rect) that hosts InspectorSections.
## Acceptance criteria
- [ ] Pan (middle mouse/space-drag), zoom to cursor, grid snap; viewport culling of off-screen layers.
- [ ] Floating properties window: draggable, collapsible, pin, position remembered per tab and clamped on resize.
- [ ] Works with a fake LayerHandler/CanvasTool in tests; no dependency on other feature crates.
