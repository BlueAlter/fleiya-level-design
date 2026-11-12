//! Project panel displaying the Worlds asset tree.
//!
//! Renders worlds as direct asset items without collapsible trees, matching an asset-tree system.
//! Double-clicking any world directly opens that world tab, which displays all of its levels.

use egui::{FontId, RichText, ScrollArea, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, tree_row, ButtonKind};
use lc_core::command::{Command, DocKind};
use lc_core::commands::{AddLevelCommand, AddWorldCommand, RemoveWorldCommand, RenameWorldCommand};
use lc_core::id::{LevelId, WorldId};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_ui_api::{EditorCtx, EditorEvent, PanelId, PanelView};

use crate::shared::ui::context_menu::{show_context_menu, MenuAnchor, MenuItem};

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

/// Project panel displaying worlds/maps as an asset list.
#[derive(Default)]
pub struct ProjectPanel {
    selected_doc: Option<DocKind>,
    renaming_world: Option<(WorldId, String)>,
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

        // Header: Project title and Maps / Worlds count
        ui.horizontal(|ui| {
            ui.label(RichText::new(Icon::World.glyph()).color(t.accent_ink));
            ui.label(
                RichText::new("MAPS")
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_bold_for(ui.ctx())))
                    .color(t.ink_strong),
            );
            ui.label(
                RichText::new(format!("{}", project.worlds.len()))
                    .font(FontId::new(theme::HUD_SIZE, theme::mono_for(ui.ctx())))
                    .color(t.muted),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if button(ui, "+ MAP", ButtonKind::Ghost, true).clicked() {
                    let next_id = project.worlds.iter().map(|w| w.id.0).max().unwrap_or(0) + 1;
                    let new_world_name = format!("Map_{next_id}");
                    let cmd = AddWorldCommand::new(World::new(WorldId(next_id), new_world_name));
                    actions.exec(cmd);
                    actions.status(format!("Added Map_{next_id}"));
                }
            });
        });

        hline(ui);

        ScrollArea::vertical().show(ui, |ui| {
            if project.worlds.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("No maps in project.").color(t.muted));
                    if button(ui, "+ ADD FIRST MAP", ButtonKind::Primary, true).clicked() {
                        let cmd = AddWorldCommand::new(World::new(WorldId(1), "Map_1".to_string()));
                        actions.exec(cmd);
                    }
                });
            } else {
                for world in &project.worlds {
                    self.render_world_asset(ui, &mut actions, world, project);
                }
            }
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
    fn render_world_asset(
        &mut self,
        ui: &mut Ui,
        actions: &mut ActionQueue,
        world: &World,
        project: &Project,
    ) {
        let world_id = world.id;
        let is_selected = self.selected_doc == Some(DocKind::World(world_id));

        // If currently in inline rename mode for this world
        if let Some((renaming_id, ref mut name_buf)) = self.renaming_world {
            if renaming_id == world_id {
                let mut finish_rename = false;
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(Icon::World.glyph()).color(Tokens::get(ui.ctx()).accent_ink),
                    );
                    let resp = ui.add(egui::TextEdit::singleline(name_buf).desired_width(120.0));
                    if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        finish_rename = true;
                    }
                    if ui.small_button("✔").clicked() {
                        finish_rename = true;
                    }
                });

                if finish_rename {
                    let trimmed = name_buf.trim();
                    if !trimmed.is_empty() && trimmed != world.name {
                        actions.exec(RenameWorldCommand::new(world_id, trimmed.to_string()));
                        actions.status(format!("Renamed map to {trimmed}"));
                    }
                    self.renaming_world = None;
                }
                return;
            }
        }

        let level_count = world.levels.len();
        let display_text = format!("{} ({} levels)", world.name, level_count);

        let row_resp = tree_row(ui, &display_text, is_selected, 0);

        // Double-click directly opens the world tab (which displays all levels)
        if row_resp.double_clicked() {
            self.selected_doc = Some(DocKind::World(world_id));
            actions.emit(EditorEvent::OpenDocument(DocKind::World(world_id)));
            actions.status(format!("Opened map {}", world.name));
        } else if row_resp.clicked() {
            self.selected_doc = Some(DocKind::World(world_id));
        }

        // Context menu on world item
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum WorldMenuAction {
            OpenMapTab,
            AddLevel,
            RenameMap,
            DeleteMap,
        }

        let menu_items = [
            MenuItem::action(WorldMenuAction::OpenMapTab, "Open Map Tab", Icon::Folder),
            MenuItem::action(WorldMenuAction::AddLevel, "Add Level", Icon::Add),
            MenuItem::action(WorldMenuAction::RenameMap, "Rename Map", Icon::Pencil),
            MenuItem::separator(),
            MenuItem::destructive(WorldMenuAction::DeleteMap, "Delete Map", Icon::Delete),
        ];

        if let Some(act) = show_context_menu(ui, MenuAnchor::context_menu(&row_resp), &menu_items) {
            match act {
                WorldMenuAction::OpenMapTab => {
                    self.selected_doc = Some(DocKind::World(world_id));
                    actions.emit(EditorEvent::OpenDocument(DocKind::World(world_id)));
                }
                WorldMenuAction::AddLevel => {
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
                }
                WorldMenuAction::RenameMap => {
                    self.renaming_world = Some((world_id, world.name.clone()));
                }
                WorldMenuAction::DeleteMap => {
                    actions.exec(RemoveWorldCommand::new(world_id));
                }
            }
        }
    }
}
