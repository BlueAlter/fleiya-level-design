---
id: T-0007
title: lc_ui_api: traits, EditorCtx, events, Viewport, ViewState
status: done
owner: agy-1
role: integrator
scope: [crates/lc_ui_api]
depends_on: [T-0005]
skills: [incremental-implementation, test-driven-development, source-driven-development, api-and-interface-design, doubt-driven-development, documentation-and-adrs]
---

## Goal
Define DocKind, DocumentView, PanelView, LayerHandler, CanvasTool, InspectorSection, Selection, EditorCtx (read project, `exec(Command)`, `emit(EditorEvent)`),
EditorEvent, Registry, and `Viewport` (zoom/pan, screen<->world<->grid conversions).
## Acceptance criteria
- [x] Signatures match docs/02-architecture.md or the doc is updated in the same PR with a short ADR.
- [x] Viewport conversion unit tests incl. negative coordinates and zoom limits.
- [x] A dummy feature in tests registers a DocumentView and a PanelView through Registry.
- [x] Doubt pass recorded in the review file (contract task).
