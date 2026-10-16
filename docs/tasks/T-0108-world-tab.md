---
id: T-0108
title: World tab: level placement, depth selector, layouts, multi-world
status: todo
owner: -
role: world
scope: [crates/lc_feature_world]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
World DocumentView: level rectangles at world_pos, drag with snap, layouts (Free, GridVania, LinearH, LinearV), depth selector (active depth full color, others dimmed),
create level from the world view, duplicate level, move depth. Multiple worlds per project, each its own tab.
## Acceptance criteria
- [ ] Levels overlapping at different depths render correctly; only active depth is interactive.
- [ ] All moves/creations are Commands (undoable); double-click level emits OpenDocument(Level).
- [ ] Layout mode behaviors have unit tests (snap math, linear arrangement).
