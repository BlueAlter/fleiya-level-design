---
id: T-0008
title: Shell: dock layout spike + implementation, menu, status bar
status: done
owner: agy-1
role: shell
scope: [crates/lc_shell, crates/lc_app, docs/decisions/]
depends_on: [T-0007]
skills: [incremental-implementation, test-driven-development, source-driven-development, documentation-and-adrs]
---

## Goal
Spike per docs/03-ui-layout.md ("Spike required"), then implement: central document area (egui_dock tabs for DocumentView), movable left/right/bottom panel docks for
PanelView, collapsible bottom dock, layout persistence + Reset Layout, menu bar (File: New/Open/Save, Edit: Undo/Redo, View), status bar.
## Acceptance criteria
- [x] ADR-0002 updated with the spike result and chosen approach.
- [x] Panels drag between left/right/bottom; documents cannot enter docks and vice versa.
- [x] Bottom dock collapses/expands via header click and Ctrl+`; state persists across restarts.
- [x] Tabs close when `is_valid` returns false. Dirty `*` indicator works.
- [x] Manual verification checklist in the PR (egui UI), plus kittest tests where feasible.
