# 01 — Data model (lc_core)

Definitions (templates) are separate from Instances (level content). IDs are stable (u64 newtypes or UUID), never Vec indices.
All structs below are created in the CONTRACT phase (T-0004) for ALL domains, so feature agents only add Commands + UI + tests, not new fields
(adding a field later = contract change, see workflow doc).

```rust
pub struct Project { version: u32, name: String, defs: Definitions, worlds: Vec<World>, settings: ProjectSettings }
pub struct Definitions { tilesets: Vec<TilesetDef>, enums: Vec<EnumDef>, entities: Vec<EntityDef>, layers: Vec<LayerDef> }

pub struct World { id: WorldId, name: String, layout: WorldLayout /* Free|GridVania|LinearH|LinearV */,
                   default_grid: Option<UVec2>, levels: Vec<Level> }
pub struct Level { id: LevelId, name: String, world_pos: IVec2, world_depth: i32, size_px: UVec2,
                   layers: Vec<LayerInstance>, fields: FieldValues }

pub enum LayerKind { IntGrid, Entities, Tiles, Auto }
pub struct LayerInstance { def_id: LayerDefId, visible: bool, data: LayerData }
pub enum LayerData { IntGrid(Grid<i32>), Tiles(Grid<Option<TileRef>>), Entities(Vec<EntityInstance>), Auto /* reserved, not implemented v1 */ }
pub struct EntityInstance { id: EntityId, def_id: EntityDefId, pos_px: IVec2, size_px: UVec2, fields: FieldValues }

pub enum FieldType { Int, Float, Bool, String, Color, Enum(EnumDefId), Point, EntityRef, Array(Box<FieldType>) }
```

Rules
- `Level.layers` order = render order. A level's layers must match project `LayerDef`s (missing ones are created lazily by a Command, never silently by UI).
- Levels with different `world_depth` may overlap in `world_pos`. The world view shows one active depth; others dimmed.
- Deleting a def (entity/enum/tileset/layer) is a Command that either blocks when referenced or cascades explicitly and reverts cleanly.
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

