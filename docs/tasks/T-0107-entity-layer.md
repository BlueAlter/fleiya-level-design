---
id: T-0107
title: Entity layer handler, tools, and floating inspector section
status: done
owner: opencode
role: entities
scope: [crates/lc_feature_entities]
depends_on: [T-0101, T-0106]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
LayerHandler for Entities layers: place, select, move, resize, delete, duplicate; InspectorSection showing the selected entity's fields in the floating window (multi-select shows common fields). Selection is managed via `Selection::Entities(...)` in `cx.selection()` and `EditorEvent::SelectionChanged`.
## Acceptance criteria
- [x] Entity commands with apply/revert/merge tests (drag = one merged command).
- [x] Field edits through Commands; typing merges into one undo step per focus session.
- [x] Entity refs resolve by ID; deleting a referenced entity is handled per data-model rules.
- [x] Selection updates via `EditorEvent::SelectionChanged(Selection::Entities(...))`, read by inspector via `cx.selection()`.

## Notes
- `instance_commands` holds `AddEntityInstancesCommand`, `RemoveEntityInstancesCommand`, `SetEntityPositionsCommand`, `SetEntitySizesCommand` and `SetEntityFieldValuesCommand`. Move/resize/field commands take an optional stroke id; commands with the same id and targets merge so one drag or typing session is one undo step.
- Instance deletion reuses the weak-reference bookkeeping extracted into `refs.rs`: references to the removed instances are cleared in every entity instance, level and entity-field default (including nested arrays), and restored by undo.
- `FocusStrokeTracker` (in `stroke.rs`) keeps a field's stroke id alive while the editor reports focus or a pointer drag, so pauses between keystrokes or mid-drag do not split the undo step; focus/drag loss starts a new stroke.
- `handler::EntitiesLayerHandler` draws entity rectangles, definition colour, pivot marker and labels with viewport culling.
- `tools::EntityCanvasTool` dispatches on the active tool name: `Paint`/`Place #<uid>` places, `Erase` deletes under the pointer, anything else selects; Delete/Backspace and Ctrl/Cmd+D act on the selection. Keyboard shortcuts are ignored while a widget owns the keyboard, so editing a field cannot delete or duplicate the entity. Placement resolves the definition from `Place #<uid>` first, then a selected instance in the active level, then the project's first entity definition.
- Duplicate keeps each copy on its source entity's Entities layer (grouped per layer); it never regroups copies onto whichever layer happens to be active.
- `inspector::EntityInspectorSection` shows POSITION/SIZE for a single selection and the definition fields; a multi-selection shows only fields present on every selected definition with the same type, marking differing values `(mixed)`. It resolves the level from `cx.active_level()` and ignores selected ids from other levels. All edits are queued as commands after rendering.
- `panel::EntitiesPanel` (`PanelId` `"entities"`) lists definitions with `#uid` and usage count, opens the definition tab on name click, and emits `SetActiveTool("Place #<uid>")` from the Place button.
- Pure grid/pick/selection helpers live in `geometry.rs`; instance commands in `instance_commands.rs` and field-value edits in `field_instance_commands.rs`, keeping every module under the 500-line rule.
- Tests: `crates/lc_feature_entities/tests/entity_layer_test.rs` covers command apply/revert/merge (including merge-false for add/remove), cascade refs (including nested arrays), grid helpers, click placement/selection through a simulated egui canvas, keyboard-focus shortcut guard, duplicate-on-source-layer, cross-level shift-click filtering, numeric-drag pause strokes, registration, and headless inspector rendering. The inspector module also tests level scoping of selected ids.
- Known follow-up: instance ids are allocated as `max(live) + 1`, so the highest deleted id can be reused. A persisted allocator or UUID-like IIDs need a contract change; logged in `docs/QUESTIONS.md`.
- Out-of-scope shell wiring was required to make the feature reachable: `crates/lc_shell/src/tool_router.rs` routes canvas input to the tile or entity tool by active layer kind, `shell_app.rs` attaches the entity handler/inspector to level tabs, and the status bar shortens `Place #<uid>` to "Place Entity". Registering the panel also required tightening the pre-fix-layout assertion in `crates/lc_shell/tests/shell_test.rs` to `[project, entities]`. All are recorded in `docs/QUESTIONS.md`.
- `LayerRenderCtx` has no selection data, so the selected rectangle is not outlined on the canvas; the floating inspector shows its fields only. The handler gap is recorded in `docs/QUESTIONS.md`.
- Manual verification steps are in `docs/MANUAL_CHECKS.md` under "Entity Layer, Tools & Floating Inspector (T-0107)".

