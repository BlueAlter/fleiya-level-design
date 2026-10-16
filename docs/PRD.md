# Level Craft — Product Requirements Document (PRD)

**Version:** 0.1.0-draft  
**Status:** Approved for Phase A Foundation  
**Project Name:** Level Craft  
**Binary Name:** `levelcraft`  
**Project File Format:** `project.lcproj` (Multi-file JSON workspace)  

---

## 1. Executive Summary & Vision

**Level Craft** is a 2D level editor written in Rust using `egui` and `eframe`. It fuses:
1. **The Modern Data Model of LDtk**: A structured hierarchy separating definitions (templates) from instances (level content), supporting multi-world workspaces, entity definitions with flexible typed fields, IntGrid layers, tile layers, and z-depth layering.
2. **The Professional Desktop UX of Tiled**: A full-featured desktop interface with a dockable workspace, a unified tabbed central document area, side/bottom panels, and in-canvas floating property inspectors.

In Level Craft, everything that represents editable content (levels, worlds, entity definitions, enum definitions, tilesets, layer definitions) opens as a **Document Tab** in the central editor area. Side panels (Project tree, Layers, Tile palette, Tools, Console) are dockable panels that dynamically reflect the active document.

---

## 2. Goals & Non-Goals

### 2.1 Core Goals (v1)
- **Single Source of Truth**: The `Project` model in `lc_core` is the definitive source of truth. UI state (pan, zoom, tool, active selection) is kept strictly separate in `ViewState`.
- **Bulletproof Undo/Redo**: Every mutation must pass through a command implementing `Command` (with `apply`, `revert`, and `merge`). UI never directly mutates project data.
- **Strict Architecture Boundaries**: Feature crates are completely decoupled from each other. They communicate solely through `lc_core` data structures, `lc_ui_api` traits, and `EditorEvent`s.
- **Git-Friendly Storage**: Multi-file JSON persistence managed exclusively by `lc_io`, with schema versioning on every file and deterministic formatting.
- **Clean Extensibility**: Independent agent teams can implement individual feature crates (Level, Tileset, Layers, Enums, Entities, World) without merge conflicts or inter-crate dependencies.

### 2.2 Explicit Non-Goals (v1)
Per ADR-0001:
- **No Auto-Layer Rules in v1**: `LayerKind::Auto` and `LayerData::Auto` are reserved in the data model for forward-compatibility, but no rule evaluation engine or auto-tiling logic is implemented in v1.
- **No Foreign File Format Import**: Import of `.ldtk` (LDtk) or `.tmx`/`.json` (Tiled) is out of scope for v1.
- **No Web (WASM) Build in v1**: Desktop only (Linux, macOS, Windows). However, all filesystem operations are strictly isolated to `lc_io`, ensuring future WASM compatibility is not precluded.
- **No Game Engine Runtime Loader Crate in v1**: Export will target engine-agnostic flat JSON; no standalone Bevy/Godot/macroquad runtime crates are included in v1.
- **No Sparse / Chunked Grids in v1**: Layers use dense grids targeting up to 1024x1024 cells per layer. Sparse or chunked storage is deferred until performance profiling demands it.

---

## 3. User Personas & Core User Stories

### 3.1 Personas
- **Alex (Solo Indie Game Developer)**: Needs to quickly sketch levels, configure entity types and game enums in one tool without switching between JSON editors and visual tools.
- **Sam (Level Designer on a Team)**: Works in git branches, merges level changes frequently. Needs git-diffable multi-file project files so level edits do not produce merge conflicts with definition edits.

### 3.2 User Stories
- **US-01 (Project & Asset Management)**: As a designer, I can open, create, and save `.lcproj` workspaces containing multiple worlds, levels, tilesets, layers, and entity definitions.
- **US-02 (Tabbed Document Editing)**: As a designer, I can open any world, level, or definition in its own tab, split tabs side-by-side, and switch between them seamlessly.
- **US-03 (World Layout & Depth)**: As a designer, I can position levels in 2D space (Free, GridVania, Linear layouts) and organize them across discrete z-depths.
- **US-04 (Tile & IntGrid Painting)**: As a designer, I can paint, erase, fill, and eyedrop tiles and integer collision grids with viewport culling and undoable strokes.
- **US-05 (Entity Definition & Placement)**: As a designer, I can define custom entities with rich fields (integers, floats, colors, enums, points, entity references) and place them on entity layers, adjusting properties in a floating canvas inspector.
- **US-06 (Auditable Console & Diagnostics)**: As a developer, I can view real-time structured logs in a dockable console panel and inspect `logs/editor.log` on disk.

---

## 4. Architecture & System Topology

The workspace is divided into strict crate boundaries enforced by the xtask architecture linter:

```
crates/
├── lc_core           # Data model, Command trait, UndoStack, IDs. (NO egui, NO std::fs)
├── lc_io             # Multi-file JSON serialization, migrations. (ONLY crate using std::fs)
├── lc_ui_api         # UI traits, EditorCtx, EditorEvent, Viewport, ViewState. (NO domain UI)
├── lc_canvas         # Reusable canvas helpers: grid render, transform math, floating inspector window
├── lc_feature_level  # Level tab: canvas host, tool dispatch, layer render loop
├── lc_feature_tileset# Tileset tab, tile palette panel, tile/IntGrid painting tools
├── lc_feature_layers # Layer definition tab, Layers hierarchy panel
├── lc_feature_enums  # Enum definition tab, enum values and color palettes
├── lc_feature_entities # Entity def tab, field editors, entity layer handler, inspector section
├── lc_feature_world  # World tab: level placement, depth selector, layout modes
├── lc_shell          # Main window dock layout, menu bar, status bar, console panel, project tree
└── lc_app            # Binary entry point (main.rs)
```

### 4.1 Dependency Rules Matrix
| Crate | Permitted Dependencies (Internal) | Third-Party Rules |
|---|---|---|
| `lc_core` | None | `serde`, `glam` / math, `thiserror`. Forbidden: `egui`, `std::fs` |
| `lc_io` | `lc_core` | `serde`, `serde_json`, `thiserror`. ONLY crate using `std::fs` |
| `lc_ui_api` | `lc_core` | `egui`, `thiserror`. No domain UI |
| `lc_canvas` | `lc_core`, `lc_ui_api` | `egui` |
| `lc_feature_*` | `lc_core`, `lc_ui_api`, `lc_canvas` | `egui`. FORBIDDEN: `lc_io`, `lc_shell`, or other features |
| `lc_shell` | All crates except `lc_app` | `egui`, `egui_dock`, `tracing` |
| `lc_app` | `lc_shell` | `eframe` |

---

## 5. Data Model (lc_core)

### 5.1 Entities and Identifiers
All identifiers are strongly typed newtypes wrapping `u64` (or UUID) with serde support:
- `ProjectId`, `WorldId`, `LevelId`, `LayerDefId`, `EntityDefId`, `EnumDefId`, `TilesetDefId`, `EntityId`.
- No cross-references use `Vec` indices.

### 5.2 Core Data Hierarchy
```rust
pub struct Project {
    pub version: u32,
    pub name: String,
    pub defs: Definitions,
    pub worlds: Vec<World>,
    pub settings: ProjectSettings,
}

pub struct Definitions {
    pub tilesets: Vec<TilesetDef>,
    pub enums: Vec<EnumDef>,
    pub entities: Vec<EntityDef>,
    pub layers: Vec<LayerDef>,
}

pub struct World {
    pub id: WorldId,
    pub name: String,
    pub layout: WorldLayout, // Free | GridVania | LinearHorizontal | LinearVertical
    pub default_grid: Option<UVec2>,
    pub levels: Vec<Level>,
}

pub struct Level {
    pub id: LevelId,
    pub name: String,
    pub world_pos: IVec2,
    pub world_depth: i32,
    pub size_px: UVec2,
    pub layers: Vec<LayerInstance>,
    pub fields: FieldValues,
}

pub enum LayerKind {
    IntGrid,
    Entities,
    Tiles,
    Auto, // Reserved for future auto-tiling
}

pub struct LayerInstance {
    pub def_id: LayerDefId,
    pub visible: bool,
    pub data: LayerData,
}

pub enum LayerData {
    IntGrid(Grid<i32>),
    Tiles(Grid<Option<TileRef>>),
    Entities(Vec<EntityInstance>),
    Auto,
}

pub struct EntityInstance {
    pub id: EntityId,
    pub def_id: EntityDefId,
    pub pos_px: IVec2,
    pub size_px: UVec2,
    pub fields: FieldValues,
}

pub enum FieldType {
    Int,
    Float,
    Bool,
    String,
    Color,
    Enum(EnumDefId),
    Point,
    EntityRef,
    Array(Box<FieldType>),
}
```

### 5.3 Dense Grid Architecture
- `Grid<T>` holds a dense `Vec<T>` with `width: u32` and `height: u32`.
- Indexing helper: `index = y * width + x`.
- Coordinates checked via bounds validation returning `Option<&T>` or `Result`.

---

## 6. Command & Undo System (lc_core)

### 6.1 Command Trait
Every state change occurs via an explicit command:
```rust
pub trait Command: std::fmt::Debug + Send + Sync {
    fn apply(&mut self, project: &mut Project) -> Result<(), CoreError>;
    fn revert(&mut self, project: &mut Project);
    fn merge(&mut self, next: &dyn Command) -> bool;
    fn label(&self) -> String;
    fn focus(&self) -> Option<DocKind>;
}
```

### 6.2 Undo Stack
- Global stack per open project in `lc_core`.
- Pushing an applied command checks `merge()`. If true, the previous command incorporates the new change (e.g. continuous mouse paint stroke or text typing).
- `undo()` calls `revert()` in reverse order.
- `redo()` re-applies commands.
- Invariant: `apply` followed by `revert` must yield byte-identical project data.

---

## 7. Storage & Multi-File Format (lc_io)

### 7.1 Directory Layout
```
<project_dir>/
  project.lcproj                      # Definitions, project metadata, world list
  worlds/
    <world_id>/
      world.json                      # World settings, layout, level placement/depth
      levels/
        <level_id>.json               # Level content (layers, tiles, entities, fields)
  assets/                             # Relative paths to external assets (e.g. PNGs)
```

### 7.2 Format Rules
- Every JSON file contains a `"version": 1` field.
- Missing files or format mismatches yield typed errors (`IoError`), never panics.
- Forward migration is supported via `lc_io::migrate::migrate_project`.
- Golden fixture round-trip tests guarantee format stability.

---

## 8. UI Layout & Extension Points (lc_ui_api & lc_shell)

### 8.1 Desktop Workspace Topology
- **Menu Bar**: File (New, Open, Save, Revert), Edit (Undo, Redo), View (Panels toggles, Reset Layout), Help.
- **Central Document Area**: Tabbed view displaying `DocumentView` instances (Level canvas, World view, Def editors). Can split tabs.
- **Dockable Panels**: Movable panels implementing `PanelView`:
  - *Left Dock*: Project panel (file/document tree).
  - *Right Dock*: Layers panel, Tile palette, Tools.
  - *Bottom Dock*: Godot-style diagnostic Console, expandable/collapsible.
- **Status Bar**: Cursor grid coordinates, zoom level, current tool, dirty indicator.
- **Floating Properties Inspector**: In-canvas egui window hosting registered `InspectorSection`s.

### 8.2 Extension Contracts
- `DocumentView`: Provides title, draws UI, validates presence in project (`is_valid`).
- `PanelView`: Provides panel ID, title, and UI render loop.
- `LayerHandler`: Dispatches rendering and tool selection for a layer kind.
- `CanvasTool`: Handles mouse/keyboard input on the canvas.
- `InspectorSection`: Contextual property editing for selected entities or levels.
- `Registry`: Discovered and populated by each feature crate's `register(&mut Registry)` function.

---

## 9. Phase B Feature Mapping Matrix

Every Phase B task directly implements a specified PRD section:

| Task ID | Task Title | PRD Section | Target Crate |
|---|---|---|---|
| **T-0101** | Level tab: canvas host, pan/zoom, grid, layer render loop, tool dispatch | Section 8.1, 8.2 (Canvas host & Viewport) | `lc_feature_level`, `lc_canvas` |
| **T-0102** | Tileset def tab and tile palette panel | Section 5.2, 8.2 (Tileset defs & Palette) | `lc_feature_tileset` |
| **T-0103** | Tiles and IntGrid layer handlers with paint/erase/rect/fill/eyedropper | Section 5.3, 6.1 (Grid tools & Strokes) | `lc_feature_tileset` |
| **T-0104** | Layer def tab and Layers panel | Section 5.2, 8.1 (Layer defs & Panel) | `lc_feature_layers` |
| **T-0105** | Enum def tab | Section 5.2 (Enum definitions & values) | `lc_feature_enums` |
| **T-0106** | Entity def tab and field system editors | Section 5.2 (Entity defs & FieldTypes) | `lc_feature_entities` |
| **T-0107** | Entity layer handler, tools, and floating inspector section | Section 5.2, 8.1 (Entity placement & Floating inspector) | `lc_feature_entities` |
| **T-0108** | World tab: level placement, depth selector, layouts, multi-world | Section 5.2, 8.1 (World layout & Depth) | `lc_feature_world` |
| **T-0190** | Cross-crate consistency sweep after first feature batch | Section 4.1, 10 (Quality & Consistency) | Integrator sweep across crates |

---

## 10. Quality, Constraints & Testing Bar

- **Zero Tolerance Lints**: `#![forbid(unsafe_code)]`, clippy denies `dbg_macro`, `todo`, `unimplemented`, `print_stdout`, `print_stderr`.
- **Core / IO Safety**: Deny `unwrap_used`, `expect_used`, `panic`, and slice indexing in `lc_core` and `lc_io`.
- **Command Reversibility**: Every command has unit tests proving `apply` then `revert` restores byte-for-byte identical project state.
- **Multi-File Round-Trip**: Golden test verifies loading and re-saving `examples/sample_project` produces identical files.
- **Coordinate Math**: Strict unit testing for screen <-> world <-> grid transformations across zoom levels and negative coordinate planes.
- **Deterministic CI Gate**: `cargo xtask check` runs format checking, clippy (-D warnings), tests, and crate boundary verification.
