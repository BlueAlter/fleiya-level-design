# ADR-0017 — LDtk-Parity Layer Definitions, Auto-Layer Rules, and World Layer Tag Filtering

- **Status:** Accepted
- **Date:** 2026-11-26
- **Updates:** ADR-0001 (Item 4: Auto-layer rules)

## Context
ADR-0001 deferred Auto-Layer rule evaluation in v1, keeping `LayerKind::Auto` and `LayerData::Auto` reserved. With the Layers Workspace (`AppMode::Layers`) now implementing the three-tab layout (`LAYERS DEF`, `AUTO TILE`, `DOCUMENT`), designers need full LDtk-parity layer configuration, Auto-Layer rule groups/patterns with live sandbox preview, and per-world layer filtering by tags (`included_layer_tags` / `excluded_layer_tags`) so multi-biome worlds do not carry unused layers.

## Decision
1. **Extend `LayerDef` & `IntGridValueDef` (`lc_core::defs`)** with LDtk-compatible properties (`tags`, `doc`, `ui_color`, opacities, `px_offset`, `parallax_factor`, `parallax_scaling`, `guide_grid_size`, editor flags, `int_grid_value_groups`, `auto_source_layer_def_id`, `auto_tiles_killed_by_other_layer_id`, `biome_field_id`, `auto_rule_groups`, `required_tags`, `excluded_tags`), all using `#[serde(default, skip_serializing_if = ...)]` to keep `examples/sample_project` byte-identical.
2. **Pure Auto-Rule Engine (`lc_core::auto_rules`, `lc_core::auto_rules_eval`)**: Implement deterministic pure-Rust rule evaluation (including 2D Perlin noise, seeded chance, `flip_x`/`flip_y`, `break_on_match`, `x_modulo`/`y_modulo`, `checker`, `out_of_bounds_value`, biome filtering, and `Single`/`Stamp` output) with **zero new crate dependencies**.
3. **Auto-Rules Eligibility (`LayerDef::uses_auto_rules`)**: `LayerKind::Auto` always uses auto-rules; `LayerKind::IntGrid` uses auto-rules when `tileset_id.is_some()`; `LayerKind::Tiles`, `LayerKind::Entities`, and untileset `IntGrid` render the `AUTO TILE` tab in an uninteractable locked state (only tab switching / layer selection remains interactive).
4. **World Layer Tag Filtering (`World::included_layer_tags`, `World::excluded_layer_tags`)**:
   - Exclude overrides include; empty `included_layer_tags` permits all non-excluded layers; non-empty `included_layer_tags` uses OR/Any matching against `LayerDef::tags`.
   - Filtering is non-destructive to existing `Level.layers` instances (skipped in UI/render until an explicit undoable prune command runs).
