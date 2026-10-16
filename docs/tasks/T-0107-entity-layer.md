---
id: T-0107
title: Entity layer handler, tools, and floating inspector section
status: todo
owner: -
role: entities
scope: [crates/lc_feature_entities]
depends_on: [T-0101, T-0106]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
LayerHandler for Entities layers: place, select, move, resize, delete, duplicate; InspectorSection showing the selected entity's fields in the floating window (multi-select shows common fields). Selection is managed via `Selection::Entities(...)` in `cx.selection()` and `EditorEvent::SelectionChanged`.
## Acceptance criteria
- [ ] Entity commands with apply/revert/merge tests (drag = one merged command).
- [ ] Field edits through Commands; typing merges into one undo step per focus session.
- [ ] Entity refs resolve by ID; deleting a referenced entity is handled per data-model rules.
- [ ] Selection updates via `EditorEvent::SelectionChanged(Selection::Entities(...))`, read by inspector via `cx.selection()`.

