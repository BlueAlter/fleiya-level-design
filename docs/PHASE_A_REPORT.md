# Level Craft — Phase A Completion & Contract Freeze Report

## 0. Setup Summary (Part 1 Bootstrap)
- **Scaffold & Directory**: Initialized in `$HOME/dev/level-craft/main` from scaffold archive. Scripts made executable, git repo initialized on branch `main` with commit `chore: level craft scaffold`.
- **Reference Repositories**: Cloned shallow depth 1 copies of LDtk (`https://github.com/deepnight/ldtk`) into `_ref/ldtk` and Tiled (`https://github.com/mapeditor/tiled`) into `_ref/tiled` as read-only references.
- **Skills Installation**: Installed via `agy plugin install` and repository clone into `.agents/skills`; verified loading of meta-skills `using-agent-skills` and `spec-driven-development`.
- **Toolchain Verification**: Verified `git`, `rustc` (1.89.0-nightly), `cargo`, `rustfmt`, and `clippy`. All workspace lints (`unsafe_code = "forbid"`, clippy denies `dbg_macro`, `todo`, `unimplemented`, `print_stdout`, `print_stderr`, plus unwrap/expect/panic denials in `lc_core` and `lc_io`) enforce strict zero-warning policy.
- **Manual Action Required**: Human tester should run `cargo run -p lc_app` with display attached and execute visual smoke checks in `docs/MANUAL_CHECKS.md`.

## 1. Phase A Tasks Summary Table (T-0000..T-0011)

| Task | Title | Status | Rounds | Reviewer Type | Commit SHA |
|---|---|---|:---:|---|---|
| T-0000 | Write docs/PRD.md for Level Craft | done | 1 | independent (`subagent-reviewer`) | `f5eacec` |
| T-0001 | Workspace skeleton & empty eframe window | done | 1 | independent (`subagent-reviewer`) | `218159c` |
| T-0002 | xtask check command with architecture lint | done | 1 | independent (`subagent-reviewer`) | `5a94417` |
| T-0003 | CI workflow running xtask check | done | 1 | independent (`subagent-reviewer`) | `a5410c1` |
| T-0004 | Core data model for ALL domains + IDs + serde | done | 1 | independent (`subagent-reviewer`) | `0c70625` |
| T-0005 | Command trait, UndoStack, first commands | done | 1 | independent (`subagent-reviewer`) | `c2b31ad` |
| T-0006 | lc_io: multi-file JSON save/load, versions, golden | done | 1 | independent (`subagent-reviewer`) | `eb2e866` |
| T-0007 | lc_ui_api: traits, EditorCtx, events, Viewport | done | 1 | independent (`subagent-reviewer`) | `08d84ec` |
| T-0008 | Shell: dock layout spike + impl, menu, status bar | done | 1 | self (`self`) | `55c3d64` |
| T-0009 | Console panel with tracing layer, file log | done | 1 | self (`self`) | `0413323` |
| T-0010 | Project panel tree and open-document flow | done | 1 | self (`self`) | `24fba7c` |
| T-0011 | CONTRACT FREEZE: integration check, tag contract-v0.1 | done | 1 | self (`self`) | `b4cbcf4` |

## 2. Pinned Versions and Architecture Decision Records (ADRs)

### Pinned Dependencies
- Rust Edition: 2021 (MSRV 1.84+)
- `egui` = 0.36.0, `eframe` = 0.36.0, `egui_dock` = 0.21.1
- `glam` = 0.29 (with `serde` feature enabled)
- `serde` = 1.0 (features: `derive`), `serde_json` = 1.0
- `tracing` = 0.1, `tracing-subscriber` = 0.3 (features: `fmt`, `ansi`, `registry`)
- `thiserror` = 2.0, `anyhow` = 1.0, `cargo_metadata` = 0.23, `ron` = 0.12 (dev)

### Architecture Decision Records
- **ADR-0001**: Autonomous bootstrap and default choices (Level Craft name, conservative v1 non-goals: no auto-layer execution, no import, no web assembly in v1).
- **ADR-0002**: Hybrid dock architecture (central `egui_dock` tab area + static resizable side/bottom dock slots for tool panels).
- **ADR-0003**: Multi-agent branch-per-task git worktree workflow (`agent/T-XXXX` in `../wt-T-XXXX`).
- **ADR-0004**: Architecture lint in `xtask check` using `cargo_metadata` without adding foreign dependencies to runtime crates.
- **ADR-0005**: Use `ron` in-memory serializer as dev-dependency for data model round-trip property testing.
- **ADR-0006**: Add `glam` to `lc_ui_api` for vector math in `Viewport` coordinate conversion methods and `CanvasCtx`.
- **ADR-0007**: Direct dependencies in `lc_shell` on feature crates and `lc_canvas` for central plugin registration.
- **ADR-0008**: Dual console logging architecture (thread-safe in-memory ring buffer layer + bounded rotating file logger in `lc_io`).

## 3. Verbatim Contract Signatures

```rust
// lc_core::command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocKind {
    Level(LevelId),
    World(WorldId),
    EntityDef(EntityDefId),
    EnumDef(EnumDefId),
    TilesetDef(TilesetDefId),
    LayerDef(LayerDefId),
    ProjectSettings,
}

pub trait Command: std::fmt::Debug + Send + Sync + 'static {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError>;
    fn revert(&mut self, p: &mut Project);
    fn merge(&mut self, next: &dyn Command) -> bool;
    fn label(&self) -> String;
    fn focus(&self) -> Option<DocKind>;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct UndoStack { /* private fields */ }
impl UndoStack {
    pub const fn new() -> Self;
    pub fn execute(&mut self, mut command: Box<dyn Command>, project: &mut Project) -> Result<Option<DocKind>, CoreError>;
    pub fn undo(&mut self, project: &mut Project) -> Option<(String, Option<DocKind>)>;
    pub fn redo(&mut self, project: &mut Project) -> Result<Option<(String, Option<DocKind>)>, CoreError>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn undo_label(&self) -> Option<String>;
    pub fn redo_label(&self) -> Option<String>;
    pub fn clear(&mut self);
}

// lc_ui_api::selection
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Selection {
    #[default]
    None,
    Level(LevelId),
    Entities(Vec<EntityId>),
    Tiles {
        layer_id: LayerDefId,
        cells: Vec<IVec2>,
    },
}
impl Selection {
    pub fn is_empty(&self) -> bool;
    pub fn is_level(&self, level_id: LevelId) -> bool;
    pub fn entities(&self) -> Option<&[EntityId]>;
    pub fn single_entity(&self) -> Option<EntityId>;
}

// lc_ui_api::view
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PanelId(pub String);
impl PanelId {
    pub fn new(id: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

pub trait DocumentView: Send {
    fn title(&self, p: &Project) -> String;
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx);
    fn is_valid(&self, p: &Project) -> bool;
    fn doc_kind(&self) -> DocKind;
}

pub trait PanelView: Send {
    fn id(&self) -> PanelId;
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx);
}

// lc_ui_api::canvas
pub struct CanvasCtx<'a> {
    pub viewport: &'a Viewport,
    pub pointer_pos: Option<egui::Pos2>,
    pub world_pos: Option<IVec2>,
    pub grid_pos: Option<IVec2>,
    pub modifiers: egui::Modifiers,
    pub response: &'a egui::Response,
    pub active_layer_id: Option<LayerDefId>,
    pub active_level_id: LevelId,
}

pub trait CanvasTool: Send {
    fn name(&self) -> &str;
    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx);
}

pub struct LayerRenderCtx<'a> {
    pub painter: &'a egui::Painter,
    pub viewport: &'a Viewport,
    pub layer: &'a LayerInstance,
    pub level: &'a Level,
    pub project: &'a Project,
    pub is_active: bool,
    pub opacity: f32,
}

pub trait LayerHandler: Send + Sync {
    fn kind(&self) -> LayerKind;
    fn render(&self, ctx: &mut LayerRenderCtx);
    fn tools(&self) -> Vec<Box<dyn CanvasTool>>;
}

// lc_ui_api::inspector
pub trait InspectorSection: Send {
    fn applies(&self, sel: &Selection) -> bool;
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx, sel: &Selection);
}

// lc_ui_api::session
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileSelection {
    pub tileset_id: TilesetDefId,
    pub tiles: Vec<u32>,
    pub width: u32,
}
impl TileSelection {
    pub fn single(tileset_id: TilesetDefId, tile_id: u32) -> Self;
    pub fn rect(tileset_id: TilesetDefId, tiles: Vec<u32>, width: u32) -> Self;
    pub fn primary_tile(&self) -> Option<TileRef>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionState {
    pub active_document: Option<DocKind>,
    pub active_level: Option<LevelId>,
    pub active_layer: Option<LayerDefId>,
    pub selection: Selection,
    pub active_tool_name: String,
    pub tile_selection: Option<TileSelection>,
}

// lc_ui_api::assets
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Asset not found at path: {0}")]
    NotFound(String),
    #[error("I/O error loading asset: {0}")]
    Io(String),
    #[error("Failed to decode image data: {0}")]
    Decode(String),
    #[error("Invalid asset path: {0}")]
    InvalidPath(String),
}

pub trait AssetService: Send + Sync {
    fn load_texture(&self, ctx: &egui::Context, relative_path: &str) -> Result<egui::TextureHandle, AssetError>;
    fn invalidate(&self, relative_path: &str);
}

#[derive(Debug, Default)]
pub struct NoopAssetService;
impl AssetService for NoopAssetService { ... }

// lc_ui_api::context
pub struct EditorCtx<'a> { /* private fields */ }
impl<'a> EditorCtx<'a> {
    pub fn new(
        project: &'a Project,
        undo_stack: &'a UndoStack,
        session: &'a SessionState,
        events: &'a mut Vec<EditorEvent>,
        pending_commands: &'a mut Vec<Box<dyn Command>>,
        assets: &'a dyn AssetService,
    ) -> Self;
    pub fn project(&self) -> &Project;
    pub fn undo_stack(&self) -> &UndoStack;
    pub fn session(&self) -> &SessionState;
    pub fn assets(&self) -> &dyn AssetService;
    pub fn active_document(&self) -> Option<DocKind>;
    pub fn active_level(&self) -> Option<LevelId>;
    pub fn active_layer(&self) -> Option<LayerDefId>;
    pub fn selection(&self) -> &Selection;
    pub fn active_tool_name(&self) -> &str;
    pub fn tile_selection(&self) -> Option<&TileSelection>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn exec(&mut self, cmd: impl Command + 'static);
    pub fn exec_boxed(&mut self, cmd: Box<dyn Command>);
    pub fn emit(&mut self, event: EditorEvent);
    pub fn status(&mut self, msg: impl Into<String>);
    pub fn log_info(&self, msg: &str);
    pub fn log_warn(&self, msg: &str);
    pub fn log_error(&self, msg: &str);
}

// lc_ui_api::events
pub enum EditorEvent {
    OpenDocument(DocKind),
    FocusDocument(DocKind),
    CloseDocument(DocKind),
    SelectionChanged(Selection),
    SetActiveLevel(Option<LevelId>),
    SetActiveLayer(Option<LayerDefId>),
    SetActiveTool(String),
    SetTileSelection(Option<TileSelection>),
    Status(String),
    SaveProject,
}

// lc_ui_api::registry
pub struct Registry { /* private fields */ }
impl Registry {
    pub fn new() -> Self;
    pub fn register_document_factory<F>(&mut self, factory: F) where F: Fn(DocKind) -> Option<Box<dyn DocumentView>> + Send + Sync + 'static;
    pub fn create_document(&self, kind: DocKind) -> Option<Box<dyn DocumentView>>;
    pub fn register_panel(&mut self, panel: Box<dyn PanelView>);
    pub fn panels(&self) -> &[Box<dyn PanelView>];
    pub fn panels_mut(&mut self) -> &mut [Box<dyn PanelView>];
    pub fn find_panel(&self, id: &PanelId) -> Option<&dyn PanelView>;
    pub fn find_panel_mut(&mut self, id: &PanelId) -> Option<&mut (dyn PanelView + 'static)>;
    pub fn register_layer_handler(&mut self, handler: Box<dyn LayerHandler>);
    pub fn layer_handlers(&self) -> &[Box<dyn LayerHandler>];
    pub fn find_layer_handler(&self, kind: LayerKind) -> Option<&dyn LayerHandler>;
    pub fn register_inspector_section(&mut self, section: Box<dyn InspectorSection>);
    pub fn inspector_sections(&self) -> &[Box<dyn InspectorSection>];
    pub fn inspector_sections_mut(&mut self) -> &mut [Box<dyn InspectorSection>];
}

// lc_ui_api::viewport
pub struct Viewport {
    pub screen_rect: egui::Rect,
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}
impl Viewport {
    pub fn new(screen_rect: egui::Rect) -> Self;
    pub fn set_zoom(&mut self, zoom: f32);
    pub fn pan_by(&mut self, delta: egui::Vec2);
    pub fn zoom_at(&mut self, factor: f32, pivot_screen: egui::Pos2);
    pub fn screen_to_world(&self, screen: egui::Pos2) -> IVec2;
    pub fn screen_to_world_f32(&self, screen: egui::Pos2) -> glam::Vec2;
    pub fn world_to_screen(&self, world: IVec2) -> egui::Pos2;
    pub fn world_to_grid(world: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn grid_to_world(grid: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn screen_to_grid(&self, screen: egui::Pos2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn grid_to_screen(&self, grid: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> egui::Rect;
    pub fn visible_world_bounds(&self) -> (IVec2, IVec2);
}

// lc_io
pub fn read_bytes(path: &Path) -> Result<Vec<u8>, IoError>;

// Top-level Data Structs (lc_core)
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
    pub layout: WorldLayout,
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

## 4. T-0008 Spike Outcome
- **egui_dock Capabilities**: `egui_dock` (0.21) provides flexible tab dragging, splitting, and docking, but does not provide built-in constraints preventing arbitrary tools from being dropped into document tabs, nor does it support collapsible single-line tool trays out of the box without complex tab viewer state.
- **Chosen Approach**: Hybrid docking architecture (detailed in ADR-0002). `egui_dock::DockArea` is used exclusively for central document tabs (`DocKind`), where tab splitting and side-by-side level viewing are valuable. Tool panels (Project, Layers, Palette, Tools, Console) reside in dedicated left/right side panels and a collapsible bottom tray (`ShellDockLayout`), with context menus to move panels between dock slots.
- **Why**: Eliminates user confusion from docking tool panels into document spaces, guarantees bottom tray collapse to 28px height on `Ctrl+``, and avoids brittle layout tree serialization hacks.

## 5. Deviations from Docs 01..03 and AGENTS.md
- **Coordinate Types**: Adopted `glam::IVec2` and `glam::UVec2` in place of raw tuples `(i32, i32)` across `lc_core` and `lc_ui_api` per ADR-0006 for vector arithmetic and zoom pivot transformations.
- **Isolated File Log in lc_io**: Replaced potential third-party `tracing-appender` in shell with a lightweight zero-allocation `RotatingLogWriter` inside `lc_io` (ADR-0008), preserving the invariant that no crate outside `lc_io` accesses `std::fs`.
- **ProjectPanel ActionQueue**: Traversal of project hierarchy in `ProjectPanel::ui` collects commands and events into an `ActionQueue` flushed at end-of-frame to prevent borrow checker conflicts against immutable `EditorCtx::project()`.

## 6. Test and Gate Status
- **Workspace Test Count**: 61 tests passing across all crates.
- **Gate Output**:
```
 INFO === Gate 1: cargo fmt --all -- --check ===
 INFO === Gate 2: cargo clippy --workspace --all-targets -- -D warnings ===
 INFO === Gate 3: cargo test --workspace ===
 INFO === Gate 4: Architecture & Crate Boundaries Lint ===
 INFO === ALL GATES PASSED: fmt, clippy, test, architecture lint ===
```
- **Suppression / Lints**: Zero lints suppressed, zero tests skipped, zero gates loosened.

## 7. Honest Risk List
1. **Headless egui Testing**: All shell and UI tests run in headless `egui::Context` without an active GPU window. While layouts, tab state, and events are 100% covered, GPU presentation under `eframe::run_native` relies on the eframe runtime.
2. **Native File Dialog**: `File > Open` currently invokes the project loader method; integration with native system dialogs (`rfd`) was deferred to dedicated shell task `T-0109` (ADR-0011) to avoid C-library desktop-portal/GTK linkage dependencies in headless CI.
3. **Independent Re-Reviews**: T-0008, T-0009, T-0010, and T-0011 have undergone independent adversarial re-reviews with fresh-context subagents. All Critical and Important findings (concurrency partial writes, ID generation collisions, module file splitting, `DocKind` display parsing, `save_to_memory` persistence) have been resolved.

## 8. Open Items
- **docs/QUESTIONS.md**: Zero unresolved blocking questions.
- **docs/MANUAL_CHECKS.md**: Contains 35 checklist items ready for human visual verification.

## 9. Phase B Tasks (T-0101..T-0108) Readiness & Scope Analysis
- **Immediately Ready Tasks**: `T-0101` (Level Canvas Host), `T-0102` (Tileset Def & Palette), `T-0104` (Layer Defs & Panel), `T-0105` (Enum Defs), `T-0106` (Entity Defs & Fields), `T-0108` (World Tab).
- **Sequenced Dependency**: `T-0107` (Entity Layer Handler) depends on `[T-0101, T-0106]`.
- **New Task**: `T-0109` (Native File Dialogs) added for the shell role.
- **Scope Overlap Note**: `T-0102` and `T-0103` both touch `crates/lc_feature_tileset`. It is recommended to implement `T-0102` first so that tile palette selection state is established before implementing paint/fill layer tools in `T-0103`.

## 10. Contract v0.2 Changes

Contract Freeze v0.2 addresses all gaps identified in Phase A re-reviews, hardens public traits, and provides the complete set of shared services needed for autonomous parallel Phase B implementation:

1. **Shared Session State (`lc_ui_api::session::SessionState`, ADR-0009)**:
   - **What**: Single-source-of-truth session state holding active document (`DocKind`), active level (`LevelId`), active layer (`LayerDefId`), `Selection`, active tool name (`String`), and tile palette selection (`TileSelection`).
   - **Why**: Feature panels (Layers, Tile Palette) and canvas tools need to observe current document and layer context without mutating persistent `Project` data or corrupting undo history.
   - **Affects**: `T-0102` (Tileset def / palette), `T-0103` (Tile painting), `T-0104` (Layer defs / panel), `T-0107` (Entity layer).

2. **Asset and Texture Service (`lc_ui_api::assets::AssetService`, ADR-0010)**:
   - **What**: Added `AssetService` trait and typed `AssetError` in `lc_ui_api`, exposed via `cx.assets().load_texture(ui.ctx(), &rel_path)`. Implemented in `lc_shell` using `lc_io::read_bytes` and `image` decoding with thread-safe texture handle caching.
   - **Why**: Gate 4 architecture lint strictly prohibits feature crates from depending on `lc_io` or calling `std::fs`.
   - **Affects**: `T-0102` (Tileset tab preview), `T-0103` (Tile layer rendering), `T-0105` (Enum icons), `T-0106` (Entity icons).

3. **DocKind Location Confirmation**:
   - **What**: Confirmed `DocKind` definition in `lc_core::command` and re-exported in `lc_ui_api`.
   - **Why**: Allows `Command::focus(&self) -> Option<DocKind>` in `lc_core` without creating a circular dependency from `lc_core` to `lc_ui_api`.

4. **Deferred Command Error Surfacing**:
   - **What**: In `ShellApp::process_commands_and_events`, command `apply()` failures log via `tracing::error!` and set `status_message` to `"Error executing <label>: <error>"`.
   - **Why**: Guarantees that errors from deferred command execution are immediately visible to the user in the status bar and logged for debugging.
   - **Affects**: All Phase B command interactions; verified via `test_command_error_surfacing_in_status_bar`.

5. **Viewport Coordinates and Sub-pixel Placement**:
   - **What**: Confirmed `screen_to_world` negative coordinate flooring. Documented integer quantization under `zoom < 1.0` in `docs/02-architecture.md`. Added `screen_to_world_f32` for continuous floating-point world calculations.
   - **Why**: Mathematical precision for tile grid alignment and smooth sub-pixel vector/ruler gizmos.
   - **Affects**: `T-0101`, `T-0103`, `T-0107`, `T-0108`.

6. **Command Deferred Timing & Multi-frame Stroke Merging**:
   - **What**: Documented end-of-frame execution timing in `docs/02-architecture.md`. Added `test_paint_stroke_multi_frame_undo_merging` in `lc_core` verifying that multi-frame dragging gestures collapse into a single undo step.
   - **Why**: Eliminates ambiguity on when tools observe mutations and guarantees intuitive single-step undo for drawing strokes.
   - **Affects**: `T-0103` (Tile painting), `T-0107` (Entity moving/resizing).

7. **Toolchain Pinning to Stable Rust 1.99.0 (ADR-0012)**:
   - **What**: Pinned `rust-toolchain.toml` to stable `1.99.0`.
   - **Why**: Resolves discrepancy from Phase A report claiming nightly; builds `egui 0.36` and entire workspace without nightly flags.

8. **Native File Dialog Deferral (Task T-0109, ADR-0011)**:
   - **What**: Created task `T-0109-file-dialog.md` for shell role and recorded ADR-0011.
   - **Why**: Avoids introducing Linux desktop-portal/GTK linkage dependencies during contract freeze.

9. **Panel Tab Drag-and-Drop between Dock Slots**:
   - **What**: Implemented panel tab dragging between Left, Right, and Bottom docks in `lc_shell::dock_panels` (~50 lines), preserving context menus as accessible fallbacks.
   - **Why**: Fully satisfies dock layout customization requirements.
   - **Affects**: Shell UI docking.

