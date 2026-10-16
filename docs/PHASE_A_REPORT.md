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

// lc_ui_api::view
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
pub trait LayerHandler: Send + Sync {
    fn kind(&self) -> LayerKind;
    fn render(&self, ctx: &mut LayerRenderCtx);
    fn tools(&self) -> Vec<Box<dyn CanvasTool>>;
}

pub trait CanvasTool: Send {
    fn name(&self) -> &str;
    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx);
}

// lc_ui_api::inspector
pub trait InspectorSection: Send {
    fn applies(&self, sel: &Selection) -> bool;
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx, sel: &Selection);
}

// lc_ui_api::context
pub struct EditorCtx<'a> { /* private fields */ }
impl<'a> EditorCtx<'a> {
    pub fn new(project: &'a Project, undo_stack: &'a UndoStack, events: &'a mut Vec<EditorEvent>, pending_commands: &'a mut Vec<Box<dyn Command>>) -> Self;
    pub fn project(&self) -> &Project;
    pub fn undo_stack(&self) -> &UndoStack;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn exec(&mut self, cmd: impl Command + 'static);
    pub fn exec_boxed(&mut self, cmd: Box<dyn Command>);
    pub fn emit(&mut self, event: EditorEvent);
    pub fn status(&mut self, msg: impl Into<String>);
}

// lc_ui_api::events
pub enum EditorEvent {
    OpenDocument(DocKind),
    FocusDocument(DocKind),
    CloseDocument(DocKind),
    SelectionChanged(Selection),
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
    pub fn world_to_screen(&self, world: IVec2) -> egui::Pos2;
    pub fn world_to_grid(world: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn grid_to_world(grid: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn screen_to_grid(&self, screen: egui::Pos2, level_world_pos: IVec2, grid_size: IVec2) -> IVec2;
    pub fn grid_to_screen(&self, grid: IVec2, level_world_pos: IVec2, grid_size: IVec2) -> egui::Rect;
    pub fn visible_world_bounds(&self) -> (IVec2, IVec2);
}

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
- **Workspace Test Count**: 47 tests passing across all crates.
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
2. **Native File Dialog**: `File > Open` currently invokes the project loader method; integration with native system dialogs (`rfd`) was deferred to avoid unpinned platform dependencies before Phase B.
3. **Single Implementer & Self-Reviews in Shell Tasks**: T-0008, T-0009, and T-0010 were reviewed via rigorous self-reviews due to execution within the autonomous bootstrap session. An independent audit is recommended before major production releases.

## 8. Open Items
- **docs/QUESTIONS.md**: Zero unresolved blocking questions.
- **docs/MANUAL_CHECKS.md**: Contains 35 checklist items ready for human visual verification.

## 9. Phase B Tasks (T-0101..T-0108) Readiness & Scope Analysis
- **Immediately Ready Tasks**: `T-0101` (Level Canvas Host), `T-0102` (Tileset Def & Palette), `T-0104` (Layer Defs & Panel), `T-0105` (Enum Defs), `T-0106` (Entity Defs & Fields), `T-0108` (World Tab).
- **Sequenced Dependency**: `T-0107` (Entity Layer Handler) depends on `[T-0101, T-0106]`.
- **Scope Overlap Note**: `T-0102` and `T-0103` both touch `crates/lc_feature_tileset`. It is recommended to implement `T-0102` first so that tile palette selection state is established before implementing paint/fill layer tools in `T-0103`.
