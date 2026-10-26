---
id: T-0106
title: Entity def tab and field system editors
status: done
owner: opencode
role: entities
scope: [crates/lc_feature_entities]
depends_on: [T-0011]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
EntityDef tab (size, color, tile/icon, pivot, field definitions) and reusable field value editors for every FieldType (Int, Float, Bool, String, Color, Enum, Point, EntityRef, Array).
## Acceptance criteria
- [x] Field editors are plain functions usable by the floating inspector (T-0107) and tab.
- [x] Add/edit/remove field defs via Commands with revert tests; value migration when a field type changes is defined and tested.

## Notes
- `field_editors::field_value_editor` / `field_type_editor` are plain functions taking a scratch value plus `FieldEditorContext`; both the tab and the floating inspector (T-0107) can call them.
- `field_defs::coerce_field_value` defines the migration: numeric/string conversions where lossless-ish, enum values validated against the enum definition, arrays coerced element-wise, everything else falls back to the target default. The command also rewrites a stored `default_value` to the new type.
- Commands cover the definition lifecycle (`commands`) and the field schema (`field_commands`); cascade deletion of an entity definition also clears dangling `EntityRef` values in entity instances (including arrays), level fields and entity field defaults, and restores them on undo.
- Revert tests compare the whole `Project` snapshot, so restore order and collateral changes are covered, not only counts.
- `register(registry)` adds the `DocKind::EntityDef` factory; `lc_shell` registers the feature next to the other feature crates.
