//! Central shell application managing docking layout, menu, status bar, and document tabs.

use egui::{CentralPanel, Context, Id, Key, Panel, Ui};
use egui_dock::{DockArea, DockState};
use glam::IVec2;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::console::{BoundedRingBuffer, ConsolePanel, SharedRingBuffer};
use crate::doc_tab_viewer::DocTabViewer;
use crate::dock_layout::{BottomDockState, DockSlotLocation, ShellDockLayout};

/// Main application shell orchestrating panels, document tabs, commands, and layout.
pub struct ShellApp {
    /// Active project data model.
    pub project: Project,
    /// Path on disk where project was loaded/saved, if any.
    pub project_path: Option<PathBuf>,
    /// Global command undo/redo stack.
    pub undo_stack: UndoStack,
    /// Registry of feature providers, panels, and document factories.
    pub registry: Registry,
    /// Docking state for central document tabs.
    pub dock_state: DockState<DocKind>,
    /// Active open document views keyed by document kind.
    pub open_documents: HashMap<DocKind, Box<dyn DocumentView>>,
    /// Layout configuration for left, right, and bottom panel docks.
    pub dock_layout: ShellDockLayout,
    /// Queue of editor events emitted during frame rendering.
    pub events: Vec<EditorEvent>,
    /// Queue of commands submitted for execution during frame rendering.
    pub pending_commands: Vec<Box<dyn Command>>,
    /// Current status bar notification message.
    pub status_message: String,
    /// Active canvas tool name shown in status bar.
    pub active_tool_name: String,
    /// Active cursor grid position shown in status bar.
    pub cursor_grid_pos: Option<IVec2>,
    /// Current canvas zoom level (1.0 = 100%).
    pub zoom_level: f32,
    /// Tracks if there are unsaved modifications.
    pub is_dirty: bool,
    /// Shared console log ring buffer.
    pub console_buffer: SharedRingBuffer,
}

impl Default for ShellApp {
    fn default() -> Self {
        Self::new(Project::new("Level Craft Project"))
    }
}

impl ShellApp {
    /// Creates a new shell instance for a given project with a default console buffer.
    pub fn new(project: Project) -> Self {
        let buffer = SharedRingBuffer::new(BoundedRingBuffer::DEFAULT_CAPACITY);
        Self::new_with_console(project, buffer)
    }

    /// Creates a shell instance with default project and specified console buffer.
    pub fn from_console(console_buffer: SharedRingBuffer) -> Self {
        Self::new_with_console(Project::new("Level Craft Project"), console_buffer)
    }

    /// Creates a new shell instance using an existing console ring buffer.
    pub fn new_with_console(project: Project, console_buffer: SharedRingBuffer) -> Self {
        let mut registry = Registry::new();
        registry.register_panel(Box::new(ConsolePanel::new(console_buffer.clone())));
        registry.register_panel(Box::new(crate::project_panel::ProjectPanel::new()));

        Self {
            project,
            project_path: None,
            undo_stack: UndoStack::new(),
            registry,
            dock_state: DockState::new(Vec::new()),
            open_documents: HashMap::new(),
            dock_layout: ShellDockLayout::default(),
            events: Vec::new(),
            pending_commands: Vec::new(),
            status_message: "Ready".to_string(),
            active_tool_name: "Select".to_string(),
            cursor_grid_pos: None,
            zoom_level: 1.0,
            is_dirty: false,
            console_buffer,
        }
    }

    /// Access to the console ring buffer.
    pub fn console_buffer(&self) -> &SharedRingBuffer {
        &self.console_buffer
    }

    /// Read-only access to registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Mutable access to registry for feature registration.
    pub fn registry_mut(&mut self) -> &mut Registry {
        &mut self.registry
    }

    /// Returns true if there are uncommitted or unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Saves the active project to the given directory path using `lc_io`.
    pub fn save_project_to_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        lc_io::save_project(&self.project, path)?;
        self.project_path = Some(path.to_path_buf());
        self.is_dirty = false;
        self.status_message = format!("Project saved to {}", path.display());
        Ok(())
    }

    /// Loads a project from the given directory path using `lc_io`.
    pub fn open_project_from_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        let loaded = lc_io::load_project(path)?;
        self.project = loaded;
        self.project_path = Some(path.to_path_buf());
        self.undo_stack.clear();
        self.open_documents.clear();
        self.dock_state = DockState::new(Vec::new());
        self.is_dirty = false;
        self.status_message = format!("Opened project from {}", path.display());
        Ok(())
    }

    /// Opens or focuses a document tab. Guarantees the same `DocKind` never opens twice.
    pub fn open_document(&mut self, kind: DocKind) {
        if self.open_documents.contains_key(&kind) {
            // Already open: focus the tab in dock_state
            self.focus_document(kind);
            return;
        }

        let doc = self
            .registry
            .create_document(kind)
            .unwrap_or_else(|| Box::new(crate::doc_tab_viewer::PlaceholderDocumentView { kind }));
        self.open_documents.insert(kind, doc);
        self.dock_state.push_to_first_leaf(kind);
        self.status_message = format!("Opened {kind:?}");
    }

    /// Focuses an existing open document tab in the dock state.
    pub fn focus_document(&mut self, kind: DocKind) {
        if let Some(pos) = self.dock_state.find_tab(&kind) {
            let _ = self.dock_state.set_active_tab(pos);
        }
    }

    /// Closes an open document tab.
    pub fn close_document(&mut self, kind: &DocKind) {
        self.open_documents.remove(kind);
        self.dock_state.retain_tabs(|t| t != kind);
    }

    /// Triggers an undo step.
    pub fn undo(&mut self) {
        if let Some((label, _focus)) = self.undo_stack.undo(&mut self.project) {
            self.status_message = format!("Undo: {label}");
        }
    }

    /// Triggers a redo step.
    pub fn redo(&mut self) {
        match self.undo_stack.redo(&mut self.project) {
            Ok(Some((label, _focus))) => {
                self.status_message = format!("Redo: {label}");
            }
            Ok(None) => {}
            Err(e) => {
                self.status_message = format!("Redo failed: {e}");
            }
        }
    }

    /// Restores default docking layout.
    pub fn reset_layout(&mut self) {
        self.dock_layout = ShellDockLayout::default();
        self.dock_state = DockState::new(self.open_documents.keys().copied().collect());
        self.status_message = "Layout reset to defaults".to_string();
    }

    /// Renders the complete shell UI inside a top-level egui `Ui`.
    pub fn ui(&mut self, ui: &mut Ui) {
        self.handle_shortcuts(ui.ctx());

        // 1. Top menu bar
        Panel::top("menu_bar").show(ui, |ui| {
            self.render_menu_bar(ui);
        });

        // 2. Bottom status bar
        Panel::bottom("status_bar").show(ui, |ui| {
            self.render_status_bar(ui);
        });

        // 3. Bottom dock (collapsible)
        self.render_bottom_dock(ui);

        // 4. Left dock panel
        if !self.dock_layout.left.panels.is_empty() {
            Panel::left("left_dock")
                .resizable(true)
                .default_size(220.0)
                .show(ui, |ui| {
                    self.render_dock_slot(ui, DockSlotLocation::Left);
                });
        }

        // 5. Right dock panel
        if !self.dock_layout.right.panels.is_empty() {
            Panel::right("right_dock")
                .resizable(true)
                .default_size(260.0)
                .show(ui, |ui| {
                    self.render_dock_slot(ui, DockSlotLocation::Right);
                });
        }

        // 6. Central document area (egui_dock)
        CentralPanel::default().show(ui, |ui| {
            self.render_central_documents(ui);
        });

        // 7. Process deferred commands and events
        self.process_commands_and_events();
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            // Ctrl+` (Backtick): Toggle bottom dock
            if i.modifiers.ctrl && i.key_pressed(Key::Backtick) {
                self.dock_layout.toggle_bottom_dock();
            }

            // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y
            if i.modifiers.ctrl && i.key_pressed(Key::Z) {
                if i.modifiers.shift {
                    self.redo();
                } else {
                    self.undo();
                }
            } else if i.modifiers.ctrl && i.key_pressed(Key::Y) {
                self.redo();
            }

            // Ctrl+S: Save
            if i.modifiers.ctrl && i.key_pressed(Key::S) {
                self.events.push(EditorEvent::SaveProject);
            }
        });
    }

    fn render_menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    self.project = Project::new("Untitled");
                    self.undo_stack.clear();
                    self.open_documents.clear();
                    self.dock_state = DockState::new(Vec::new());
                    self.is_dirty = false;
                    self.status_message = "New project created".to_string();
                    ui.close();
                }
                if ui.button("Save (Ctrl+S)").clicked() {
                    self.events.push(EditorEvent::SaveProject);
                    ui.close();
                }
            });

            ui.menu_button("Edit", |ui| {
                let undo_label = self
                    .undo_stack
                    .undo_label()
                    .map(|l| format!("Undo {l} (Ctrl+Z)"))
                    .unwrap_or_else(|| "Undo (Ctrl+Z)".to_string());
                if ui
                    .add_enabled(self.undo_stack.can_undo(), egui::Button::new(undo_label))
                    .clicked()
                {
                    self.undo();
                    ui.close();
                }

                let redo_label = self
                    .undo_stack
                    .redo_label()
                    .map(|l| format!("Redo {l} (Ctrl+Y)"))
                    .unwrap_or_else(|| "Redo (Ctrl+Y)".to_string());
                if ui
                    .add_enabled(self.undo_stack.can_redo(), egui::Button::new(redo_label))
                    .clicked()
                {
                    self.redo();
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                let bottom_label = if self.dock_layout.bottom.collapsed {
                    "Expand Bottom Dock (Ctrl+`)"
                } else {
                    "Collapse Bottom Dock (Ctrl+`)"
                };
                if ui.button(bottom_label).clicked() {
                    self.dock_layout.toggle_bottom_dock();
                    ui.close();
                }

                if ui.button("Reset Layout").clicked() {
                    self.reset_layout();
                    ui.close();
                }
            });
        });
    }

    fn render_status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // 1. Grid coordinates
            if let Some(pos) = self.cursor_grid_pos {
                ui.label(format!("Grid: [{}, {}]", pos.x, pos.y));
            } else {
                ui.label("Grid: -");
            }
            ui.separator();

            // 2. Zoom percentage
            ui.label(format!("{:.0}%", self.zoom_level * 100.0));
            ui.separator();

            // 3. Active tool
            ui.label(&self.active_tool_name);
            ui.separator();

            // 4. Dirty indicator
            if self.is_dirty() {
                ui.colored_label(egui::Color32::from_rgb(255, 180, 50), "* Unsaved changes");
            } else {
                ui.colored_label(egui::Color32::GRAY, "Saved");
            }
            ui.separator();

            // 5. Status message
            ui.label(&self.status_message);
        });
    }

    fn render_bottom_dock(&mut self, ui: &mut Ui) {
        if self.dock_layout.bottom.panels.is_empty() {
            return;
        }

        let is_collapsed = self.dock_layout.bottom.collapsed;
        let height = if is_collapsed {
            BottomDockState::COLLAPSED_HEIGHT
        } else {
            self.dock_layout.bottom.height
        };

        Panel::bottom("bottom_dock")
            .resizable(!is_collapsed)
            .default_size(height)
            .show(ui, |ui| {
                // Header bar
                ui.horizontal(|ui| {
                    let collapse_icon = if is_collapsed { "▲" } else { "▼" };
                    if ui.button(collapse_icon).clicked() {
                        self.dock_layout.bottom.toggle_collapsed();
                    }

                    let panels = self.dock_layout.bottom.panels.clone();
                    let active = self.dock_layout.bottom.active.clone();

                    for panel_id in panels {
                        let is_selected = active.as_ref() == Some(&panel_id);
                        let title = self
                            .registry
                            .find_panel(&panel_id)
                            .map(|p| p.title())
                            .unwrap_or_else(|| panel_id.as_str());

                        let btn = ui.selectable_label(is_selected, title);
                        if btn.clicked() {
                            if is_selected {
                                // Clicking active tab toggles collapse
                                self.dock_layout.bottom.toggle_collapsed();
                            } else {
                                // Clicking inactive tab selects it and expands
                                self.dock_layout.bottom.active = Some(panel_id.clone());
                                self.dock_layout.bottom.collapsed = false;
                            }
                        }

                        // Context menu to move dock
                        btn.context_menu(|ui| {
                            if ui.button("Move to Left Dock").clicked() {
                                self.dock_layout
                                    .move_panel(&panel_id, DockSlotLocation::Left);
                                ui.close();
                            }
                            if ui.button("Move to Right Dock").clicked() {
                                self.dock_layout
                                    .move_panel(&panel_id, DockSlotLocation::Right);
                                ui.close();
                            }
                        });
                    }
                });

                // Render content if expanded
                if !is_collapsed {
                    ui.separator();
                    if let Some(active_id) = self.dock_layout.bottom.active.clone() {
                        if let Some(panel) = self.registry.find_panel_mut(&active_id) {
                            let mut cx = EditorCtx::new(
                                &self.project,
                                &self.undo_stack,
                                &mut self.events,
                                &mut self.pending_commands,
                            );
                            panel.ui(ui, &mut cx);
                        }
                    }
                }
            });
    }

    fn render_dock_slot(&mut self, ui: &mut Ui, location: DockSlotLocation) {
        let (panels, active) = match location {
            DockSlotLocation::Left => (
                self.dock_layout.left.panels.clone(),
                self.dock_layout.left.active.clone(),
            ),
            DockSlotLocation::Right => (
                self.dock_layout.right.panels.clone(),
                self.dock_layout.right.active.clone(),
            ),
            DockSlotLocation::Bottom => (
                self.dock_layout.bottom.panels.clone(),
                self.dock_layout.bottom.active.clone(),
            ),
        };

        // Tab strip
        ui.horizontal(|ui| {
            for panel_id in &panels {
                let is_selected = active.as_ref() == Some(panel_id);
                let title = self
                    .registry
                    .find_panel(panel_id)
                    .map(|p| p.title())
                    .unwrap_or_else(|| panel_id.as_str());

                let btn = ui.selectable_label(is_selected, title);
                if btn.clicked() {
                    match location {
                        DockSlotLocation::Left => {
                            self.dock_layout.left.set_active(panel_id.clone())
                        }
                        DockSlotLocation::Right => {
                            self.dock_layout.right.set_active(panel_id.clone())
                        }
                        DockSlotLocation::Bottom => {
                            self.dock_layout.bottom.set_active(panel_id.clone())
                        }
                    }
                }

                // Context menu to move panel
                btn.context_menu(|ui| {
                    if location != DockSlotLocation::Left
                        && ui.button("Move to Left Dock").clicked()
                    {
                        self.dock_layout
                            .move_panel(panel_id, DockSlotLocation::Left);
                        ui.close();
                    }
                    if location != DockSlotLocation::Right
                        && ui.button("Move to Right Dock").clicked()
                    {
                        self.dock_layout
                            .move_panel(panel_id, DockSlotLocation::Right);
                        ui.close();
                    }
                    if location != DockSlotLocation::Bottom
                        && ui.button("Move to Bottom Dock").clicked()
                    {
                        self.dock_layout
                            .move_panel(panel_id, DockSlotLocation::Bottom);
                        ui.close();
                    }
                });
            }
        });

        ui.separator();

        // Render active panel content
        if let Some(active_id) = active {
            if let Some(panel) = self.registry.find_panel_mut(&active_id) {
                let mut cx = EditorCtx::new(
                    &self.project,
                    &self.undo_stack,
                    &mut self.events,
                    &mut self.pending_commands,
                );
                panel.ui(ui, &mut cx);
            }
        }
    }

    fn render_central_documents(&mut self, ui: &mut Ui) {
        let is_dirty = self.is_dirty();
        let mut tab_viewer = DocTabViewer {
            project: &self.project,
            open_documents: &mut self.open_documents,
            undo_stack: &self.undo_stack,
            events: &mut self.events,
            pending_commands: &mut self.pending_commands,
            is_dirty,
        };

        DockArea::new(&mut self.dock_state)
            .id(Id::new("central_document_dock_area"))
            .show_inside(ui, &mut tab_viewer);

        // Prune any documents that were closed or became invalid
        let active_tabs: Vec<DocKind> = self.dock_state.iter_all_tabs().map(|(_, t)| *t).collect();
        self.open_documents
            .retain(|k, v| active_tabs.contains(k) && v.is_valid(&self.project));
        let valid_docs = self.open_documents.keys().copied().collect::<Vec<_>>();
        self.dock_state.retain_tabs(|t| valid_docs.contains(t));
    }

    fn process_commands_and_events(&mut self) {
        // 1. Apply queued commands
        let commands = std::mem::take(&mut self.pending_commands);
        for cmd in commands {
            let label = cmd.label();
            if let Err(e) = self.undo_stack.execute(cmd, &mut self.project) {
                tracing::error!("Error executing {label}: {e}");
                self.status_message = format!("Error executing {label}: {e}");
            } else {
                tracing::debug!("Applied command: {label}");
                self.is_dirty = true;
                self.status_message = label;
            }
        }

        // 2. Handle emitted events
        let events = std::mem::take(&mut self.events);
        for event in events {
            match event {
                EditorEvent::OpenDocument(kind) => self.open_document(kind),
                EditorEvent::FocusDocument(kind) => self.focus_document(kind),
                EditorEvent::CloseDocument(kind) => self.close_document(&kind),
                EditorEvent::Status(msg) => self.status_message = msg,
                EditorEvent::SelectionChanged(_) => {}
                EditorEvent::SaveProject => {
                    if let Some(ref path) = self.project_path {
                        if let Err(e) = lc_io::save_project(&self.project, path) {
                            tracing::error!("Failed to save project: {e}");
                            self.status_message = format!("Failed to save project: {e}");
                        } else {
                            self.is_dirty = false;
                            self.status_message = format!("Saved project to {}", path.display());
                        }
                    } else {
                        self.is_dirty = false;
                        self.status_message = "Project saved".to_string();
                    }
                }
            }
        }
    }
}
