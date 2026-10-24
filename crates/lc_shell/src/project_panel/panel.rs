//! Project panel tree view managing Worlds, Depth groups, Levels, and Definitions.

use egui::{CollapsingHeader, FontId, Frame, Id, RichText, ScrollArea, Ui};
use glam::IVec2;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, tree_row, ButtonKind};
use lc_core::command::{Command, DocKind};
use lc_core::commands::{
    AddLevelCommand, AddWorldCommand, MoveLevelCommand, RemoveLevelCommand, RemoveWorldCommand,
    RenameLevelCommand, RenameWorldCommand,
};
use lc_core::id::{LevelId, WorldId};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_ui_api::{EditorCtx, EditorEvent, PanelId, PanelView};
use std::collections::BTreeSet;

/// Internal action queue to defer event and command emissions during UI rendering,
/// avoiding borrow conflicts with `cx.project()`.
#[derive(Default)]
struct ActionQueue {
    commands: Vec<Box<dyn Command>>,
    events: Vec<EditorEvent>,
    status: Option<String>,
}

impl ActionQueue {
    fn exec(&mut self, cmd: impl Command + 'static) {
        self.commands.push(Box::new(cmd));
    }

    fn emit(&mut self, event: EditorEvent) {
        self.events.push(event);
    }

    fn status(&mut self, msg: impl Into<String>) {
        self.status = Some(msg.into());
    }
}

/// Project panel displaying the hierarchical structure of the active project.
#[derive(Default)]
pub struct ProjectPanel {
    selected_doc: Option<DocKind>,
}

impl ProjectPanel {
    /// Creates a new project panel.
    pub fn new() -> Self {
        Self::default()
    }
}

impl PanelView for ProjectPanel {
    fn id(&self) -> PanelId {
        PanelId::new("project")
    }

    fn title(&self) -> &str {
        "Project"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let mut actions = ActionQueue::default();
        let t = Tokens::get(ui.ctx());
        let project = cx.project();

        ScrollArea::vertical().show(ui, |ui| {
            // 1. Root: Project Info & Settings
            ui.label(
                RichText::new(&project.name)
                    .font(FontId::new(13.0, theme::semibold_for(ui.ctx())))
                    .color(t.ink_strong),
            );

            // Project settings row
            let settings_resp = tree_row(
                ui,
                "PROJECT SETTINGS",
                self.selected_doc == Some(DocKind::ProjectSettings),
                0,
            );
            if settings_resp.double_clicked() || settings_resp.clicked() {
                self.selected_doc = Some(DocKind::ProjectSettings);
                actions.emit(EditorEvent::OpenDocument(DocKind::ProjectSettings));
            }

            hline(ui);

            // 2. Worlds & Levels Tree
            CollapsingHeader::new(
                RichText::new(format!("WORLDS  {}", project.worlds.len()))
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                    .color(t.muted),
            )
            .default_open(true)
            .show_background(false)
            .show(ui, |ui| {
                // Button to add new world
                if button(ui, "+ ADD WORLD", ButtonKind::Ghost, true).clicked() {
                    let next_id = project.worlds.iter().map(|w| w.id.0).max().unwrap_or(0) + 1;
                    let new_world_name = format!("World_{next_id}");
                    let cmd = AddWorldCommand::new(World::new(WorldId(next_id), new_world_name));
                    actions.exec(cmd);
                }

                for world in &project.worlds {
                    self.render_world(ui, &mut actions, world, project);
                }
            });

            hline(ui);

            // 3. Definitions Tree
            CollapsingHeader::new(
                RichText::new("DEFINITIONS")
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                    .color(t.muted),
            )
            .default_open(false)
            .show_background(false)
            .show(ui, |ui| {
                let def_headers = [
                    ("ENTITY DEFS", project.defs.entities.len()),
                    ("ENUM DEFS", project.defs.enums.len()),
                    ("TILESET DEFS", project.defs.tilesets.len()),
                    ("LAYER DEFS", project.defs.layers.len()),
                ];
                for (label, count) in def_headers {
                    CollapsingHeader::new(
                        RichText::new(format!("{label}  {count}"))
                            .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                            .color(t.faint),
                    )
                    .default_open(false)
                    .show_background(false)
                    .show(ui, |ui| match label {
                        "ENTITY DEFS" => {
                            for def in &project.defs.entities {
                                let kind = DocKind::EntityDef(def.id);
                                let is_selected = self.selected_doc == Some(kind);
                                let resp = tree_row(ui, &def.name, is_selected, 1);
                                if resp.double_clicked() {
                                    self.selected_doc = Some(kind);
                                    actions.emit(EditorEvent::OpenDocument(kind));
                                }
                            }
                        }
                        "ENUM DEFS" => {
                            for def in &project.defs.enums {
                                let kind = DocKind::EnumDef(def.id);
                                let is_selected = self.selected_doc == Some(kind);
                                let resp = tree_row(ui, &def.name, is_selected, 1);
                                if resp.double_clicked() {
                                    self.selected_doc = Some(kind);
                                    actions.emit(EditorEvent::OpenDocument(kind));
                                }
                            }
                        }
                        "TILESET DEFS" => {
                            for def in &project.defs.tilesets {
                                let kind = DocKind::TilesetDef(def.id);
                                let is_selected = self.selected_doc == Some(kind);
                                let resp = tree_row(ui, &def.name, is_selected, 1);
                                if resp.double_clicked() {
                                    self.selected_doc = Some(kind);
                                    actions.emit(EditorEvent::OpenDocument(kind));
                                }
                            }
                        }
                        _ => {
                            for def in &project.defs.layers {
                                let kind = DocKind::LayerDef(def.id);
                                let is_selected = self.selected_doc == Some(kind);
                                let resp = tree_row(ui, &def.name, is_selected, 1);
                                if resp.double_clicked() {
                                    self.selected_doc = Some(kind);
                                    actions.emit(EditorEvent::OpenDocument(kind));
                                }
                            }
                        }
                    });
                }
            });
        });

        // Apply deferred actions to EditorCtx
        for event in actions.events {
            cx.emit(event);
        }
        for cmd in actions.commands {
            cx.exec_boxed(cmd);
        }
        if let Some(msg) = actions.status {
            cx.status(msg);
        }
    }
}

impl ProjectPanel {
    fn render_world(
        &mut self,
        ui: &mut Ui,
        actions: &mut ActionQueue,
        world: &World,
        project: &Project,
    ) {
        let t = Tokens::get(ui.ctx());
        let world_id = world.id;
        let is_selected = self.selected_doc == Some(DocKind::World(world_id));

        let header = CollapsingHeader::new(
            RichText::new(&world.name)
                .font(FontId::new(12.5, theme::semibold_for(ui.ctx())))
                .color(t.ink),
        )
        .id_salt(format!("world_{}", world.id.0))
        .default_open(true)
        .show_background(false);

        let header_resp = header.show(ui, |ui| {
            // Group levels by depth
            let mut depths = BTreeSet::new();
            depths.insert(0); // Ensure depth 0 is always available
            depths.insert(1); // Common candidate depths for drag-and-drop
            depths.insert(-1);
            for level in &world.levels {
                depths.insert(level.world_depth);
            }

            for depth in depths {
                self.render_depth_group(ui, actions, world, depth, project);
            }
        });

        // Double-click world header to open world tab
        if header_resp.header_response.double_clicked() {
            self.selected_doc = Some(DocKind::World(world_id));
            actions.emit(EditorEvent::OpenDocument(DocKind::World(world_id)));
        }

        // Context menu on world header
        header_resp.header_response.context_menu(|ui| {
            if ui.button("Open World Tab").clicked() {
                self.selected_doc = Some(DocKind::World(world_id));
                actions.emit(EditorEvent::OpenDocument(DocKind::World(world_id)));
                ui.close();
            }
            if ui.button("Add Level").clicked() {
                let next_id = project
                    .worlds
                    .iter()
                    .flat_map(|w| &w.levels)
                    .map(|l| l.id.0)
                    .max()
                    .unwrap_or(0)
                    + 1;
                let level_name = format!("Level_{next_id}");
                let level = Level::new(LevelId(next_id), level_name, glam::UVec2::new(32, 32));
                actions.exec(AddLevelCommand::new(world_id, level));
                ui.close();
            }
            if ui.button("Rename World").clicked() {
                let new_name = format!("{}_renamed", world.name);
                actions.exec(RenameWorldCommand::new(world_id, new_name));
                ui.close();
            }
            ui.separator();
            if ui.button("Delete World").clicked() {
                actions.exec(RemoveWorldCommand::new(world_id));
                ui.close();
            }
        });

        if is_selected {
            header_resp.header_response.highlight();
        }
    }

    fn render_depth_group(
        &mut self,
        ui: &mut Ui,
        actions: &mut ActionQueue,
        world: &World,
        depth: i32,
        project: &Project,
    ) {
        let t = Tokens::get(ui.ctx());
        let levels_at_depth: Vec<&Level> = world
            .levels
            .iter()
            .filter(|l| l.world_depth == depth)
            .collect();

        let group_id = Id::new(("depth_drop_zone", world.id, depth));
        let (_inner, dropped_payload) = ui.dnd_drop_zone::<LevelId, _>(Frame::new(), |ui| {
            CollapsingHeader::new(
                RichText::new(format!("DEPTH {}  {}", depth, levels_at_depth.len()))
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                    .color(t.faint),
            )
            .id_salt(group_id)
            .default_open(true)
            .show_background(false)
            .show(ui, |ui| {
                if levels_at_depth.is_empty() {
                    hud(ui, "DROP LEVEL HERE TO MOVE TO THIS DEPTH", t.faint);
                }

                for level in levels_at_depth {
                    self.render_level_item(ui, actions, world.id, level, project);
                }
            });
        });

        // If a level was dropped onto this depth group, issue MoveLevelCommand
        if let Some(dropped_level_id) = dropped_payload {
            let dropped_id = *dropped_level_id;
            if let Some(level) = world.levels.iter().find(|l| l.id == dropped_id) {
                if level.world_depth != depth {
                    let cmd = MoveLevelCommand::new(dropped_id, level.world_pos, depth);
                    actions.exec(cmd);
                    actions.status(format!("Moved level to depth {depth}"));
                }
            }
        }
    }

    fn render_level_item(
        &mut self,
        ui: &mut Ui,
        actions: &mut ActionQueue,
        world_id: WorldId,
        level: &Level,
        project: &Project,
    ) {
        let level_id = level.id;
        let is_selected = self.selected_doc == Some(DocKind::Level(level_id));
        let item_id = Id::new(("level_drag", level_id));

        let drag_resp = ui
            .dnd_drag_source(item_id, level_id, |ui| {
                let label = tree_row(ui, &level.name, is_selected, 1);

                if label.double_clicked() {
                    self.selected_doc = Some(DocKind::Level(level_id));
                    actions.emit(EditorEvent::OpenDocument(DocKind::Level(level_id)));
                } else if label.clicked() {
                    self.selected_doc = Some(DocKind::Level(level_id));
                }

                label
            })
            .response;

        // Context menu for operations on Level
        drag_resp.context_menu(|ui| {
            if ui.button("Open Level").clicked() {
                self.selected_doc = Some(DocKind::Level(level_id));
                actions.emit(EditorEvent::OpenDocument(DocKind::Level(level_id)));
                ui.close();
            }
            if ui.button("Rename Level").clicked() {
                let new_name = format!("{}_renamed", level.name);
                actions.exec(RenameLevelCommand::new(level_id, new_name));
                ui.close();
            }
            if ui.button("Duplicate Level").clicked() {
                let mut duplicated = level.clone();
                let next_id = project
                    .worlds
                    .iter()
                    .flat_map(|w| &w.levels)
                    .map(|l| l.id.0)
                    .max()
                    .unwrap_or(0)
                    + 1;
                duplicated.id = LevelId(next_id);
                duplicated.name = format!("{}_copy", level.name);
                duplicated.world_pos += IVec2::new(32, 0);
                actions.exec(AddLevelCommand::new(world_id, duplicated));
                ui.close();
            }
            ui.menu_button("Move to Depth...", |ui| {
                let mut target_depths =
                    vec![level.world_depth - 1, level.world_depth + 1, 0, 1, -1];
                target_depths.retain(|&d| d != level.world_depth);
                target_depths.sort_unstable();
                target_depths.dedup();
                for target_depth in target_depths {
                    if ui.button(format!("Depth {target_depth}")).clicked() {
                        actions.exec(MoveLevelCommand::new(
                            level_id,
                            level.world_pos,
                            target_depth,
                        ));
                        ui.close();
                    }
                }
            });
            ui.separator();
            if ui.button("Delete Level").clicked() {
                actions.exec(RemoveLevelCommand::new(world_id, level_id));
                ui.close();
            }
        });
    }
}
