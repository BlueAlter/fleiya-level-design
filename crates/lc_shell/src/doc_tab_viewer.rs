//! egui_dock TabViewer implementation for central DocumentView tabs.

use egui::{Id, Ui, WidgetText};
use egui_dock::TabViewer;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::{canvas_preview, empty_state, hud, meta_row, page, page_header};
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::project::Project;
use lc_ui_api::assets::AssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;

/// Renders document tabs inside the central `egui_dock::DockArea`.
pub struct DocTabViewer<'a> {
    pub project: &'a Project,
    pub open_documents: &'a mut HashMap<DocKind, Box<dyn DocumentView>>,
    pub undo_stack: &'a UndoStack,
    pub session: &'a SessionState,
    pub events: &'a mut Vec<EditorEvent>,
    pub pending_commands: &'a mut Vec<Box<dyn Command>>,
    pub assets: &'a dyn AssetService,
    pub is_dirty: bool,
}

impl<'a> TabViewer for DocTabViewer<'a> {
    type Tab = DocKind;

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        let title_str = if let Some(doc) = self.open_documents.get(tab) {
            doc.title(self.project)
        } else {
            format!("{tab:?}")
        };

        let suffix = if self.is_dirty { " *" } else { "" };
        egui::RichText::new(format!("{title_str}{suffix}"))
            .font(egui::FontId::new(11.0, egui::FontFamily::Monospace))
            .into()
    }

    fn context_menu(&mut self, ui: &mut Ui, tab: &mut Self::Tab, _path: egui_dock::NodePath) {
        if ui.button("Close").clicked() {
            self.events.push(EditorEvent::CloseDocument(*tab));
            ui.close();
        }
        if ui.button("Close Others").clicked() {
            let current = *tab;
            let others: Vec<DocKind> = self
                .open_documents
                .keys()
                .copied()
                .filter(|kind| *kind != current)
                .collect();
            for kind in others {
                self.events.push(EditorEvent::CloseDocument(kind));
            }
            ui.close();
        }
        if ui.button("Close All").clicked() {
            let all: Vec<DocKind> = self.open_documents.keys().copied().collect();
            for kind in all {
                self.events.push(EditorEvent::CloseDocument(kind));
            }
            ui.close();
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(doc) = self.open_documents.get_mut(tab) {
            let mut cx = EditorCtx::new(
                self.project,
                self.undo_stack,
                self.session,
                self.events,
                self.pending_commands,
                self.assets,
            );
            doc.ui(ui, &mut cx);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Document view not found");
            });
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        // Automatically close tab if the underlying document is no longer valid
        if let Some(doc) = self.open_documents.get(tab) {
            !doc.is_valid(self.project)
        } else {
            true
        }
    }
}

/// Fallback placeholder DocumentView implementation for shell tabs.
pub struct PlaceholderDocumentView {
    /// Document kind this view represents.
    pub kind: DocKind,
}

impl DocumentView for PlaceholderDocumentView {
    fn doc_kind(&self) -> DocKind {
        self.kind
    }

    fn title(&self, project: &Project) -> String {
        match self.kind {
            DocKind::Level(id) => project
                .level(id)
                .map(|l| l.name.clone())
                .unwrap_or_else(|| format!("Level({id})")),
            DocKind::World(id) => project
                .world(id)
                .map(|w| w.name.clone())
                .unwrap_or_else(|| format!("World({id})")),
            DocKind::EntityDef(id) => project
                .defs
                .entities
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("EntityDef({id})")),
            DocKind::EnumDef(id) => project
                .defs
                .enums
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("EnumDef({id})")),
            DocKind::TilesetDef(id) => project
                .defs
                .tilesets
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("TilesetDef({id})")),
            DocKind::LayerDef(id) => project
                .defs
                .layers
                .iter()
                .find(|l| l.id == id)
                .map(|l| l.name.clone())
                .unwrap_or_else(|| format!("LayerDef({id})")),
            DocKind::ProjectSettings => "Project Settings".to_string(),
        }
    }

    fn is_valid(&self, project: &Project) -> bool {
        match self.kind {
            DocKind::Level(id) => project.level(id).is_some(),
            DocKind::World(id) => project.world(id).is_some(),
            DocKind::EntityDef(id) => project.defs.entities.iter().any(|e| e.id == id),
            DocKind::EnumDef(id) => project.defs.enums.iter().any(|e| e.id == id),
            DocKind::TilesetDef(id) => project.defs.tilesets.iter().any(|t| t.id == id),
            DocKind::LayerDef(id) => project.defs.layers.iter().any(|l| l.id == id),
            DocKind::ProjectSettings => true,
        }
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let project = cx.project();
        let kind = self.kind;
        let title = self.title(project);
        let (tag, description) = match kind {
            DocKind::Level(_) => ("LEVEL", "Level canvas, layers and entity placement."),
            DocKind::World(_) => ("WORLD", "World layout and level placement overview."),
            DocKind::EntityDef(_) => (
                "ENTITY DEF",
                "Entity template, default size and custom fields.",
            ),
            DocKind::EnumDef(_) => ("ENUM DEF", "Named value set used by custom fields."),
            DocKind::TilesetDef(_) => ("TILESET DEF", "Image source, tile size and atlas layout."),
            DocKind::LayerDef(_) => ("LAYER DEF", "Layer template, grid size and value palette."),
            DocKind::ProjectSettings => (
                "PROJECT",
                "Project defaults, definitions and file metadata.",
            ),
        };
        let t = Tokens::get(ui.ctx());

        page(ui, |ui| {
            page_header(ui, tag, &title, description);
            ui.add_space(16.0);

            match kind {
                DocKind::Level(id) => {
                    if let Some(level) = project.level(id) {
                        meta_row(ui, "LEVEL ID", &format!("#{}", level.id.0));
                        meta_row(
                            ui,
                            "SIZE",
                            &format!("{} × {} px", level.size_px.x, level.size_px.y),
                        );
                        meta_row(
                            ui,
                            "WORLD POS",
                            &format!("{}, {}", level.world_pos.x, level.world_pos.y),
                        );
                        meta_row(ui, "DEPTH", &level.world_depth.to_string());
                        meta_row(ui, "LAYERS", &level.layers.len().to_string());
                        ui.add_space(16.0);
                        hud(ui, "CANVAS PREVIEW / READ ONLY", t.faint);
                        ui.add_space(6.0);
                        let grid = project.settings.default_grid_size.x.max(1) as i32;
                        canvas_preview(ui, 260.0, grid);
                        ui.add_space(12.0);
                        empty_state(
                            ui,
                            "CANVAS HOST",
                            "Level canvas is not wired yet",
                            "Pan, zoom, layer selection and tools arrive with T-0101. This preview uses the shared grid and checkerboard only.",
                            "READ-ONLY PREVIEW",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::World(id) => {
                    if let Some(world) = project.world(id) {
                        meta_row(ui, "WORLD ID", &format!("#{}", world.id.0));
                        meta_row(ui, "LAYOUT", &format!("{:?}", world.layout));
                        meta_row(
                            ui,
                            "DEFAULT GRID",
                            &world
                                .default_grid
                                .map(|g| format!("{} × {}", g.x, g.y))
                                .unwrap_or_else(|| "AUTO".to_string()),
                        );
                        meta_row(ui, "LEVELS", &world.levels.len().to_string());
                        ui.add_space(16.0);
                        empty_state(
                            ui,
                            "WORLD OVERVIEW",
                            "World placement is not wired yet",
                            "Level placement, depth selection and layouts arrive with T-0108.",
                            "OPEN A LEVEL FROM THE PROJECT PANEL",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::EntityDef(id) => {
                    if let Some(entity) = project.defs.entity(id) {
                        meta_row(ui, "ENTITY ID", &format!("#{}", entity.id.0));
                        meta_row(
                            ui,
                            "SIZE",
                            &format!("{} × {} px", entity.size_px.x, entity.size_px.y),
                        );
                        meta_row(
                            ui,
                            "PIVOT",
                            &format!("{:.2}, {:.2}", entity.pivot.x, entity.pivot.y),
                        );
                        meta_row(ui, "FIELDS", &entity.fields.len().to_string());
                        meta_row(
                            ui,
                            "TILE",
                            &entity
                                .tile
                                .map(|tile| format!("#{} / {}", tile.tileset_id.0, tile.tile_id))
                                .unwrap_or_else(|| "NONE".to_string()),
                        );
                        ui.add_space(16.0);
                        empty_state(
                            ui,
                            "ENTITY EDITOR",
                            "Entity definition editing is not wired yet",
                            "Properties, default fields and tile previews arrive with T-0106.",
                            "READ-ONLY DEFINITION",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::EnumDef(id) => {
                    if let Some(enum_def) = project.defs.enum_def(id) {
                        meta_row(ui, "ENUM ID", &format!("#{}", enum_def.id.0));
                        meta_row(ui, "VALUES", &enum_def.values.len().to_string());
                        ui.add_space(16.0);
                        empty_state(
                            ui,
                            "ENUM EDITOR",
                            "Enum definition editing is not wired yet",
                            "Values, colors and tile icons arrive with T-0105.",
                            "READ-ONLY DEFINITION",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::TilesetDef(id) => {
                    if let Some(tileset) = project.defs.tileset(id) {
                        meta_row(ui, "TILESET ID", &format!("#{}", tileset.id.0));
                        meta_row(ui, "PATH", &tileset.path);
                        meta_row(
                            ui,
                            "TILE SIZE",
                            &format!("{} × {} px", tileset.tile_size.x, tileset.tile_size.y),
                        );
                        meta_row(ui, "SPACING", &format!("{} px", tileset.spacing));
                        meta_row(ui, "PADDING", &format!("{} px", tileset.padding));
                        ui.add_space(16.0);
                        empty_state(
                            ui,
                            "TILESET EDITOR",
                            "Tileset editing is not wired yet",
                            "Image loading, atlas slicing and tile previews arrive with T-0102.",
                            "READ-ONLY DEFINITION",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::LayerDef(id) => {
                    if let Some(layer) = project.defs.layer(id) {
                        meta_row(ui, "LAYER ID", &format!("#{}", layer.id.0));
                        meta_row(ui, "KIND", &format!("{:?}", layer.kind));
                        meta_row(
                            ui,
                            "GRID",
                            &format!("{} × {} px", layer.grid_size.x, layer.grid_size.y),
                        );
                        meta_row(
                            ui,
                            "TILESET",
                            &layer
                                .tileset_id
                                .map(|id| format!("#{}", id.0))
                                .unwrap_or_else(|| "NONE".to_string()),
                        );
                        meta_row(ui, "INT VALUES", &layer.int_grid_values.len().to_string());
                        ui.add_space(16.0);
                        empty_state(
                            ui,
                            "LAYER EDITOR",
                            "Layer definition editing is not wired yet",
                            "Grid size, tileset binding and value palettes arrive with T-0104.",
                            "READ-ONLY DEFINITION",
                        );
                    } else {
                        missing_document(ui);
                    }
                }
                DocKind::ProjectSettings => {
                    meta_row(ui, "PROJECT", &project.name);
                    meta_row(ui, "VERSION", &project.version.to_string());
                    meta_row(
                        ui,
                        "DEFAULT GRID",
                        &format!(
                            "{} × {} px",
                            project.settings.default_grid_size.x,
                            project.settings.default_grid_size.y
                        ),
                    );
                    meta_row(ui, "WORLDS", &project.worlds.len().to_string());
                    meta_row(ui, "ENTITIES", &project.defs.entities.len().to_string());
                    meta_row(ui, "ENUMS", &project.defs.enums.len().to_string());
                    meta_row(ui, "TILESETS", &project.defs.tilesets.len().to_string());
                    meta_row(ui, "LAYERS", &project.defs.layers.len().to_string());
                    ui.add_space(16.0);
                    empty_state(
                        ui,
                        "PROJECT SETTINGS",
                        "Settings editing is not wired yet",
                        "Defaults, validation and the save flow arrive with the project settings work. This page is a read-only summary.",
                        "READ-ONLY SUMMARY",
                    );
                }
            }
        });
    }
}

fn missing_document(ui: &mut Ui) {
    empty_state(
        ui,
        "MISSING",
        "Document not found",
        "This item was removed from the project. The tab closes on the next frame.",
        "NO DOCUMENT",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::ShellAssetService;
    use glam::{UVec2, Vec2};
    use lc_core::defs::{EntityDef, EnumDef, LayerDef, TilesetDef};
    use lc_core::id::{EntityDefId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
    use lc_core::layer::LayerKind;
    use lc_core::level::Level;
    use lc_core::world::World;
    use lc_ui_api::session::SessionState;

    fn populated_project() -> Project {
        let mut project = Project::new("Test");
        let mut world = World::new(WorldId(1), "Overworld");
        world
            .levels
            .push(Level::new(LevelId(1), "Level_0", UVec2::new(1920, 1080)));
        project.worlds.push(world);
        project.defs.tilesets.push(TilesetDef {
            id: TilesetDefId(1),
            name: "Tiles".to_string(),
            path: "tiles.png".to_string(),
            tile_size: UVec2::splat(16),
            spacing: 0,
            padding: 0,
        });
        project.defs.enums.push(EnumDef {
            id: EnumDefId(1),
            name: "Biome".to_string(),
            values: Vec::new(),
        });
        project.defs.entities.push(EntityDef {
            id: EntityDefId(1),
            name: "Player".to_string(),
            size_px: UVec2::splat(16),
            color: [255, 255, 255, 255],
            pivot: Vec2::splat(0.5),
            tile: None,
            fields: Vec::new(),
        });
        project.defs.layers.push(LayerDef {
            id: LayerDefId(1),
            name: "Collisions".to_string(),
            kind: LayerKind::IntGrid,
            grid_size: UVec2::splat(16),
            tileset_id: None,
            int_grid_values: Vec::new(),
        });
        project
    }

    #[test]
    fn placeholder_pages_render_for_every_document_kind() {
        let project = populated_project();
        let undo = UndoStack::new();
        let session = SessionState::default();
        let assets = ShellAssetService::new();
        let kinds = [
            DocKind::Level(LevelId(1)),
            DocKind::World(WorldId(1)),
            DocKind::EntityDef(EntityDefId(1)),
            DocKind::EnumDef(EnumDefId(1)),
            DocKind::TilesetDef(TilesetDefId(1)),
            DocKind::LayerDef(LayerDefId(1)),
            DocKind::ProjectSettings,
        ];
        let ctx = egui::Context::default();

        for kind in kinds {
            let mut view = PlaceholderDocumentView { kind };
            let mut events = Vec::new();
            let mut commands = Vec::new();
            let mut output = ctx.run_ui(Default::default(), |ui| {
                let mut cx = EditorCtx::new(
                    &project,
                    &undo,
                    &session,
                    &mut events,
                    &mut commands,
                    &assets,
                );
                view.ui(ui, &mut cx);
            });
            output.textures_delta.clear();
        }
    }
}
