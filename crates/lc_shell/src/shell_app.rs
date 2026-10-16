//! Central shell application managing docking layout, menu, status bar, and document tabs.

use egui::{CentralPanel, Id, Panel, Ui};
use egui_dock::{DockArea, DockState};
use glam::IVec2;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::project::Project;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::assets::ShellAssetService;
use crate::console::{BoundedRingBuffer, ConsolePanel, SharedRingBuffer};
use crate::doc_tab_viewer::DocTabViewer;
use crate::dock_layout::{DockSlotLocation, ShellDockLayout};
use lc_ui_api::session::SessionState;
use std::sync::Arc;

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
    /// Shared session state readable by panels, tools, and inspector sections.
    pub session: SessionState,
    /// Asset and texture service.
    pub assets: Arc<ShellAssetService>,
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
    /// Last recorded bounding rect of the central document area.
    pub last_central_rect: Option<egui::Rect>,
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

        let assets = Arc::new(ShellAssetService::new());

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
            session: SessionState::default(),
            assets,
            status_message: "Ready".to_string(),
            active_tool_name: "Select".to_string(),
            cursor_grid_pos: None,
            zoom_level: 1.0,
            is_dirty: false,
            console_buffer,
            last_central_rect: None,
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
        self.assets
            .set_project_dir(path.parent().map(|p| p.to_path_buf()));
        self.is_dirty = false;
        self.status_message = format!("Project saved to {}", path.display());
        Ok(())
    }

    /// Loads a project from the given directory path using `lc_io`.
    pub fn open_project_from_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        let loaded = lc_io::load_project(path)?;
        self.project = loaded;
        self.project_path = Some(path.to_path_buf());
        self.assets
            .set_project_dir(path.parent().map(|p| p.to_path_buf()));
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
        if self.session.active_document == Some(*kind) {
            self.session.active_document = None;
            if let DocKind::Level(_) = kind {
                self.session.active_level = None;
            }
        }
    }

    /// Triggers an undo step.
    pub fn undo(&mut self) {
        if let Some((label, focus)) = self.undo_stack.undo(&mut self.project) {
            self.status_message = format!("Undo: {label}");
            if let Some(doc) = focus {
                self.open_document(doc);
                self.session.active_document = Some(doc);
                if let DocKind::Level(lid) = doc {
                    self.session.active_level = Some(lid);
                }
            }
        }
    }

    /// Triggers a redo step.
    pub fn redo(&mut self) {
        match self.undo_stack.redo(&mut self.project) {
            Ok(Some((label, focus))) => {
                self.status_message = format!("Redo: {label}");
                if let Some(doc) = focus {
                    self.open_document(doc);
                    self.session.active_document = Some(doc);
                    if let DocKind::Level(lid) = doc {
                        self.session.active_level = Some(lid);
                    }
                }
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
        // Ensure registered panels are tracked in dock layout
        for panel in self.registry.panels() {
            self.dock_layout
                .ensure_panel(panel.id(), DockSlotLocation::Left);
        }

        crate::menu::handle_global_shortcuts(self, ui.ctx());

        // 1. Top menu bar
        Panel::top("menu_bar").show(ui, |ui| {
            crate::menu::render_menu_bar(self, ui);
        });

        // 2. Bottom status bar
        Panel::bottom("status_bar").show(ui, |ui| {
            crate::status_bar::render_status_bar(self, ui);
        });

        // 3. Left dock panel
        if !self.dock_layout.left.panels.is_empty() {
            Panel::left("left_dock")
                .resizable(true)
                .default_size(220.0)
                .show(ui, |ui| {
                    crate::dock_panels::render_dock_slot(self, ui, DockSlotLocation::Left);
                });
        }

        // 4. Right dock panel
        if !self.dock_layout.right.panels.is_empty() {
            Panel::right("right_dock")
                .resizable(true)
                .default_size(260.0)
                .show(ui, |ui| {
                    crate::dock_panels::render_dock_slot(self, ui, DockSlotLocation::Right);
                });
        }

        // 5. Bottom dock (collapsible)
        crate::dock_panels::render_bottom_dock(self, ui);

        // 6. Central document area (egui_dock)
        CentralPanel::default().show(ui, |ui| {
            self.render_central_documents(ui);
        });

        // 7. Process deferred commands and events
        self.process_commands_and_events();

        // 8. Cancel dragging if pointer released outside dock slots
        if ui.input(|i| i.pointer.any_released()) && self.dock_layout.dragging_panel.is_some() {
            self.dock_layout.cancel_drag();
        }
    }

    fn render_central_documents(&mut self, ui: &mut Ui) {
        self.last_central_rect = Some(ui.max_rect());
        let is_dirty = self.is_dirty();

        if let Some((_, active_tab)) = self.dock_state.find_active_focused() {
            self.session.active_document = Some(*active_tab);
            self.session.active_level = if let DocKind::Level(lid) = *active_tab {
                Some(lid)
            } else {
                None
            };
        } else {
            self.session.active_document = None;
            self.session.active_level = None;
        }

        let mut tab_viewer = DocTabViewer {
            project: &self.project,
            open_documents: &mut self.open_documents,
            undo_stack: &self.undo_stack,
            session: &self.session,
            events: &mut self.events,
            pending_commands: &mut self.pending_commands,
            assets: &*self.assets,
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
            let focus = cmd.focus();
            if let Err(e) = self.undo_stack.execute(cmd, &mut self.project) {
                tracing::error!("Error executing {label}: {e}");
                self.status_message = format!("Error executing {label}: {e}");
            } else {
                tracing::debug!("Applied command: {label}");
                self.is_dirty = true;
                self.status_message = label;
                if let Some(doc) = focus {
                    self.open_document(doc);
                    self.session.active_document = Some(doc);
                    if let DocKind::Level(lid) = doc {
                        self.session.active_level = Some(lid);
                    }
                }
            }
        }

        // 2. Handle emitted events
        let events = std::mem::take(&mut self.events);
        for event in events {
            match event {
                EditorEvent::OpenDocument(kind) => {
                    self.open_document(kind);
                    self.session.active_document = Some(kind);
                    if let DocKind::Level(lid) = kind {
                        self.session.active_level = Some(lid);
                    }
                }
                EditorEvent::FocusDocument(kind) => {
                    self.focus_document(kind);
                    self.session.active_document = Some(kind);
                    if let DocKind::Level(lid) = kind {
                        self.session.active_level = Some(lid);
                    }
                }
                EditorEvent::CloseDocument(kind) => {
                    self.close_document(&kind);
                }
                EditorEvent::Status(msg) => self.status_message = msg,
                EditorEvent::SelectionChanged(sel) => {
                    tracing::debug!("Selection changed: {sel:?}");
                    self.session.selection = sel;
                }
                EditorEvent::SetActiveLevel(id) => {
                    self.session.active_level = id;
                }
                EditorEvent::SetActiveLayer(id) => {
                    self.session.active_layer = id;
                }
                EditorEvent::SetActiveTool(name) => {
                    self.active_tool_name = name.clone();
                    self.session.active_tool_name = name;
                }
                EditorEvent::SetTileSelection(ts) => {
                    self.session.tile_selection = ts;
                }
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
                        tracing::warn!("Cannot save project: no project path configured");
                        self.status_message =
                            "Save failed: no project path configured (specify path or open project first)".to_string();
                    }
                }
            }
        }
    }
}
