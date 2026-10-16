---
id: T-0106
title: Entity def tab and field system editors
status: todo
owner: -
role: entities
scope: [crates/lc_feature_entities]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
EntityDef tab (size, color, tile/icon, pivot, field definitions) and reusable field value editors for every FieldType (Int, Float, Bool, String, Color, Enum, Point, EntityRef, Array).
## Acceptance criteria
- [ ] Field editors are plain functions usable by the floating inspector (T-0107) and tab.
- [ ] Add/edit/remove field defs via Commands with revert tests; value migration when a field type changes is defined and tested.
