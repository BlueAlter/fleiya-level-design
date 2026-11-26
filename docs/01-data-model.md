# 01 — Data model (lc_core)

Definitions (templates) are separate from Instances (level content). IDs are stable (u64 newtypes or UUID), never Vec indices.
All structs below are created in the CONTRACT phase (T-0004) for ALL domains, so feature agents only add Commands + UI + tests, not new fields
(adding a field later = contract change, see workflow doc).

```rust
pub struct Project { version: u32, name: String, defs: Definitions, worlds: Vec<World>, settings: ProjectSettings }
pub struct Definitions { tilesets: Vec<TilesetDef>, enums: Vec<EnumDef>, entities: Vec<EntityDef>, layers: Vec<LayerDef> }

pub struct TilesetDef {
    pub id: TilesetDefId,
    pub name: String,
    pub path: String,
    pub tile_size: UVec2,
    pub spacing: u32,
    pub padding: u32,
    pub tags: Vec<String>,
    pub marking: TileMarking,
    pub custom_data: BTreeMap<u32, String>,
    pub animations: BTreeMap<u32, TileAnimationDef>,
    pub collisions: BTreeMap<u32, TileCollisionDef>,
}

pub struct TileMarking {
    pub source_enum: Option<EnumDefId>,
    pub tags: BTreeMap<String, BTreeSet<u32>>,
}

pub struct World { id: WorldId, name: String, layout: WorldLayout /* Free|GridVania|LinearH|LinearV */,
                   default_grid: Option<UVec2>, levels: Vec<Level> }
pub struct Level { id: LevelId, name: String, world_pos: IVec2, world_depth: i32, size_px: UVec2,
                   layers: Vec<LayerInstance>, fields: FieldValues }

pub enum LayerKind { IntGrid, Entities, Tiles, Auto }
pub struct LayerInstance { def_id: LayerDefId, visible: bool, data: LayerData }
pub enum LayerData { IntGrid(Grid<i32>), Tiles(Grid<Option<TileRef>>), Entities(Vec<EntityInstance>), Auto /* reserved, not implemented v1 */ }
pub struct EntityInstance { id: EntityId, def_id: EntityDefId, pos_px: IVec2, size_px: UVec2, fields: FieldValues }

pub enum FieldType { Int, Float, Bool, String, Multilines, Color, Enum(Option<EnumDefId>), Point, EntityRef, FilePath, Tile, Array(Box<FieldType>) }
```

Notes on LDtk Entity & Field parity (`lc_core::entity` & `lc_core::fields`):
- `EntityDef` carries full LDtk properties (`tags`, `doc`, `color`, `fill_opacity`, `line_opacity`, `hollow`, `render_mode`, `tile`, `tile_render_mode`, `nine_slice_borders`, `tile_opacity`, `ui_tile_rect`, `show_name`, `resizable_x`, `resizable_y`, `keep_aspect_ratio`, `allow_out_of_bounds`, `max_count`, `limit_scope`, `limit_behavior`, `export_to_toc`, `fields`), all using `serde(default, skip_serializing_if = ...)` for backwards compatibility.
- `FieldDef` includes stable `id: FieldDefId`, `doc`, `is_array`, `can_be_null`, `default_value`, `min`/`max`, `regex`, `accept_file_types`, `text_language_mode`, `allowed_refs`, `allowed_ref_tags`, `allowed_refs_entity_def`, `tileset_id`, `symmetrical_ref`, `auto_chain_ref`, `array_min_length`/`array_max_length`, `editor_display_mode`, `editor_display_pos`, `editor_link_style`, `editor_display_scale`, `editor_always_show`, `editor_cut_long_values`, `editor_show_in_world`, and `editor_text_suffix`.

Rules
- `Level.layers` order = render order. A level's layers must match project `LayerDef`s (missing ones are created lazily by a Command, never silently by UI).
- Levels with different `world_depth` may overlap in `world_pos`. The world view shows one active depth; others dimmed.
- Deleting a def (entity/enum/tileset/layer) is a Command that either blocks when referenced or cascades explicitly and reverts cleanly.
- `TilesetDef.tags` stores unique, case-insensitive string tags for classification.
- `TilesetDef.marking` binds an optional source `EnumDef` to a deterministic mapping of enum value identifiers to tile index sets.
- Cascades from enum removal, value deletion, and value renaming automatically update or clear bound tileset markings, with full command undo/revert parity.
- `Grid<T>` is dense (`Vec<T>` + width/height). Sparse/chunked storage is a later ADR if needed (target v1: up to 1024x1024 cells per layer).

## File format (ADR-0001): multi-file JSON, every file has `"version"`, asset paths relative
```
project.lcproj                      defs + world list
worlds/<world>/world.json          layout + level placement/depth
worlds/<world>/levels/<lvl>.json   level content
assets/...                         images
```
Migration functions live in `lc_io::migrate`, one per version step, each with a test fixture.

## Session state (lc_ui_api::session, ADR-0009)
Transient runtime editor state (active document `DocKind`, active level `LevelId`, active layer `LayerDefId`, canvas `Selection`, active tool name, tile palette `TileSelection`) is managed in `SessionState` and exposed read-only via `EditorCtx`. It is never serialized into the project files and never stored on the command undo stack.

