# 02 — Architecture and crate boundaries

```
crates/
  lc_core           data model + Commands + undo. No egui. No fs.
  lc_io             load/save, JSON, versions, migration. The ONLY crate using std::fs.
  lc_ui_api         egui-facing contracts: traits, EditorCtx, events, Viewport, ViewState. No domain UI.
  lc_canvas         shared canvas helpers: grid drawing, floating-window helper, selection rect.
  lc_feature_level    level tab: canvas host, tool dispatch, layer rendering loop
  lc_feature_tileset  tileset tab, tile palette panel, Tiles/IntGrid layer handlers (paint/erase/rect/fill/eyedropper)
  lc_feature_layers   layer-def tab, Layers panel (visibility, order)
  lc_feature_enums    enum-def tab
  lc_feature_entities entity-def tab, field editors, entity layer handler (place/move/resize), floating inspector content
  lc_feature_world    world tab (level placement, depth selector, layouts), multi-world
  lc_shell          dock layout, menu bar, status bar, console, project panel, feature registration
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
Third-party: egui/eframe only in ui_api, canvas, features, shell, app. egui_dock only in shell. serde_json only in io.

## Extension points (defined in lc_ui_api, T-0007; this is what lets agents work in parallel)
```rust
pub enum DocKind { Level(LevelId), World(WorldId), EntityDef(EntityDefId), EnumDef(EnumDefId),
                   TilesetDef(TilesetDefId), LayerDef(LayerDefId), ProjectSettings }
pub trait DocumentView { fn title(&self, p:&Project)->String; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx);
                         fn is_valid(&self, p:&Project)->bool; /* false => shell closes the tab */ }
pub trait PanelView    { fn id(&self)->PanelId; fn title(&self)->&str; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx); }
pub trait LayerHandler { fn kind(&self)->LayerKind; fn render(&self, ...); fn tools(&self)->Vec<Box<dyn CanvasTool>>; }
pub trait CanvasTool   { fn name(&self)->&str; fn on_input(&mut self, cv:&mut CanvasCtx, cx:&mut EditorCtx); }
pub trait InspectorSection { fn applies(&self, sel:&Selection)->bool; fn ui(&mut self, ui:&mut egui::Ui, cx:&mut EditorCtx, sel:&Selection); }
```
- `EditorCtx` gives read access to `Project`, `cx.exec(Box<dyn Command>)` (the only way to mutate), `cx.emit(EditorEvent)`, and logging helpers.
- `EditorEvent` examples: `OpenDocument(DocKind)`, `FocusDocument(DocKind)`, `SelectionChanged`, `Status(String)`.
- Each feature crate exposes `pub fn register(reg: &mut Registry)`. `lc_shell` calls every `register`. Adding a feature touches ONE line in shell.
- A feature needing something from another feature goes through core data or an `EditorEvent`. If impossible, write a QUESTIONS.md entry.

## Command system (lc_core)
```rust
pub trait Command { fn apply(&mut self, p:&mut Project)->Result<(),CoreError>; fn revert(&mut self, p:&mut Project);
                    fn merge(&mut self, next:&dyn Command)->bool; fn label(&self)->String; fn focus(&self)->Option<DocKind>; }
```
One global undo stack per project. A paint stroke = one command merged while the mouse is down. `focus()` lets undo jump to the affected tab.
