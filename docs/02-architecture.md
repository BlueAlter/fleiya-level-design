# 02 — Architecture and crate boundaries

```
crates/
  lc_core           data model + Commands + undo + DocKind. No egui. No fs.
  lc_io             load/save, binary reading, JSON, versions, migration. The ONLY crate using std::fs.
  lc_ui_api         egui-facing contracts: traits, EditorCtx, SessionState, AssetService, events, Viewport, ViewState. No domain UI.
  lc_canvas         shared canvas helpers: grid drawing, floating-window helper, selection rect.
  lc_feature_level    level tab: canvas host, tool dispatch, layer rendering loop
  lc_feature_tileset  tileset tab, tile palette panel, Tiles/IntGrid layer handlers (paint/erase/rect/fill/eyedropper)
  lc_feature_layers   layer-def tab, Layers panel (visibility, order)
  lc_feature_enums    enum-def tab
  lc_feature_entities entity-def tab, field editors, entity layer handler (place/move/resize), floating inspector content
  lc_feature_world    world tab (level placement, depth selector, layouts), multi-world
  lc_shell          dock layout, menu bar, status bar, console, project panel, feature registration, asset manager
  lc_app            main.rs only
xtask/                  cargo xtask check|board
```

## Dependency matrix (enforced by xtask arch-lint)
| crate | may depend on (workspace crates) |
|---|---|
| lc_core | none |
| lc_io | core |
| lc_ui_api | core |
| lc_canvas | core, ui_api |
| lc_feature_* | core, ui_api, canvas — NEVER io, shell, or another feature |
| lc_shell | everything except lc_app |
| lc_app | shell |
Third-party: egui/eframe only in ui_api, canvas, features, shell, app. egui_dock only in shell. egui_commonmark only in shell (ADR-0016). serde_json only in io.
Toolchain: Pinned to stable `1.99.0` in `rust-toolchain.toml` (newest stable compiling egui 0.36 without nightly flags; see ADR-0012).

## Extension points (defined in lc_ui_api, with DocKind defined in lc_core::command)
```rust
// Defined in lc_core::command so Command::focus() can return it without circular dependency; re-exported in lc_ui_api:
pub enum DocKind { Level(LevelId), World(WorldId), EntityDef(EntityDefId), EnumDef(EnumDefId),
                   TilesetDef(TilesetDefId), LayerDef(LayerDefId), ProjectSettings }

pub trait DocumentView { fn title(&self, p:&Project)->String; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx);
                         fn is_valid(&self, p:&Project)->bool; fn doc_kind(&self)->DocKind; /* false => shell closes the tab */ }
pub trait PanelView    { fn id(&self)->PanelId; fn title(&self)->&str; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx); }
pub trait LayerHandler { fn kind(&self)->LayerKind; fn render(&self, ...); fn tools(&self)->Vec<Box<dyn CanvasTool>>; }
pub trait CanvasTool   { fn name(&self)->&str; fn on_input(&mut self, cv:&mut CanvasCtx, cx:&mut EditorCtx); }
pub trait InspectorSection { fn applies(&self, sel:&Selection)->bool; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx, sel:&Selection); }

// Shared session state and asset service:
pub struct SessionState { pub active_document: Option<DocKind>, pub active_level: Option<LevelId>,
                          pub active_layer: Option<LayerDefId>, pub selection: Selection,
                          pub active_tool_name: String, pub tile_selection: Option<TileSelection> }

pub trait AssetService: Send + Sync {
    fn load_texture(&self, ctx: &egui::Context, relative_path: &str) -> Result<egui::TextureHandle, AssetError>;
    fn invalidate(&self, relative_path: &str);
}
```

- `EditorCtx` gives read-only access to `Project`, `UndoStack`, `SessionState` (`cx.session()`, `cx.active_document()`, etc.), and `AssetService` (`cx.assets().load_texture(...)`).
- Mutations go strictly through commands (`cx.exec(cmd)`), and cross-cutting UI operations go through events (`cx.emit(EditorEvent)`).
- `EditorEvent` variants: `OpenDocument(DocKind)`, `FocusDocument(DocKind)`, `CloseDocument(DocKind)`, `SelectionChanged(Selection)`, `SetActiveLevel`, `SetActiveLayer`, `SetActiveTool`, `SetTileSelection`, `Status(String)`, `SaveProject`.
- Each feature crate exposes `pub fn register(reg: &mut Registry)`. `lc_shell` calls every `register`; that covers document factories, panels, layer
  handlers and inspector sections. Exception: level tabs are still assembled explicitly in `ShellApp::open_document` (level view + tile/entity
  handlers + `LevelCanvasToolRouter`), so a new level-layer feature currently also touches shell (tracked in `docs/QUESTIONS.md`, T-0107 note).

## Command system & Timing (lc_core / lc_shell)
```rust
pub trait Command { fn apply(&mut self, p:&mut Project)->Result<(),CoreError>; fn revert(&mut self, p:&mut Project);
                    fn merge(&mut self, next:&dyn Command)->bool; fn label(&self)->String; fn focus(&self)->Option<DocKind>;
                    fn as_any(&self)->&dyn std::any::Any; }
```
- **Deferred Execution Timing**: Commands submitted via `cx.exec(cmd)` are queued during frame rendering and applied at the end of the frame by `ShellApp::process_commands_and_events()`. Views and tools in frame N observe the pre-mutation project; frame N+1 observes the mutated project.
- **Stroke Merging**: Multi-frame interactions (e.g. continuous paint strokes or dragging a level) emit consecutive mergeable commands across successive frames. `UndoStack::execute` merges each frame's command into the top command via `top.merge(&*cmd)`, resulting in exactly ONE undo step for the entire interaction.
- **Error Surfacing**: If a deferred command fails during `apply()`, the failure is logged via `tracing::error!` and surfaced immediately in the bottom status bar with error details.
- `focus()` lets undo jump back to the affected document tab.

## Viewport & Coordinates (lc_ui_api)
- `screen_to_world` converts screen pixels to integer world coordinates using `.floor()` for both positive and negative spaces.
- **Zoom < 1.0 Note**: When zoomed out (zoom < 1.0), 1 screen pixel spans `1.0 / zoom` world pixels, creating quantization where multiple screen pixels map to the same integer coordinate.
- `screen_to_world_f32` is provided for continuous sub-pixel placement (e.g. vector gizmos or rulers).
