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
use crate::file_dialog::{
    normalize_project_path, FileDialogHandle, FileDialogOutcome, FileDialogService,
    NativeFileDialogService,
};
use lc_canvas::theme::{self, ThemeChoice};
use lc_ui_api::session::SessionState;
use std::sync::Arc;

/// Which action a pending file dialog will perform when it completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileDialogAction {
    OpenProject,
    SaveProjectAs,
}

/// A file dialog whose worker thread has not reported back yet.
struct PendingFileDialog {
    action: FileDialogAction,
    handle: FileDialogHandle,
}

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
    /// Selected appearance preference.
    pub theme_choice: ThemeChoice,
    /// Shared console log ring buffer.
    pub console_buffer: SharedRingBuffer,
    /// Last recorded bounding rect of the central document area.
    pub last_central_rect: Option<egui::Rect>,
    /// Backend used to request native file dialogs.
    file_dialogs: Arc<dyn FileDialogService>,
    /// File dialogs waiting for a user decision.
    pending_file_dialogs: Vec<PendingFileDialog>,
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

        lc_feature_level::register(&mut registry);
        lc_feature_tileset::register(&mut registry);
        lc_feature_layers::register(&mut registry);
        lc_feature_enums::register(&mut registry);
        lc_feature_entities::register(&mut registry);
        lc_feature_world::register(&mut registry);

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
            theme_choice: ThemeChoice::Dark,
            console_buffer,
            last_central_rect: None,
            file_dialogs: Arc::new(NativeFileDialogService::new()),
            pending_file_dialogs: Vec::new(),
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

    /// Read-only access to the loaded project model.
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Mutable access to the project model.
    pub fn project_mut(&mut self) -> &mut Project {
        &mut self.project
    }

    /// Read-only access to shared session state.
    pub fn session(&self) -> &SessionState {
        &self.session
    }

    /// Mutable access to shared session state.
    pub fn session_mut(&mut self) -> &mut SessionState {
        &mut self.session
    }

    /// Returns true if there are uncommitted or unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Applies a new appearance preference immediately.
    pub fn set_theme(&mut self, ctx: &egui::Context, choice: ThemeChoice) {
        self.theme_choice = choice;
        theme::apply(ctx, choice);
    }

    /// Saves the active project to the given directory path using `lc_io`.
    pub fn save_project_to_path(&mut self, path: &std::path::Path) -> Result<(), lc_io::IoError> {
        lc_io::save_project(&self.project, path)?;
        self.project_path = Some(path.to_path_buf());
        self.assets
            .set_project_dir(path.parent().map(|p| p.to_path_buf()));
        self.is_dirty = false;
        self.status_message = format!("Project saved to {}", path.display());
        tracing::info!(path = %path.display(), "Saved project");
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
        tracing::info!(path = %path.display(), "Opened project");
        Ok(())
    }

    /// Replaces the backend used for native file dialogs.
    ///
    /// Tests and headless harnesses use this to inject a mock service.
    pub fn set_file_dialog_service(&mut self, service: Arc<dyn FileDialogService>) {
        self.file_dialogs = service;
    }

    /// Starts an asynchronous "Open project" dialog.
    pub fn request_open_project_dialog(&mut self) {
        if self.has_pending_dialog(FileDialogAction::OpenProject) {
            self.status_message = "Open dialog is already active".to_string();
            return;
        }

        tracing::info!("Requested open project dialog");
        self.pending_file_dialogs.push(PendingFileDialog {
            action: FileDialogAction::OpenProject,
            handle: self.file_dialogs.open_project(),
        });
        self.status_message = "Select a project file...".to_string();
    }

    /// Starts an asynchronous "Save project as" dialog.
    pub fn request_save_project_as_dialog(&mut self) {
        if self.has_pending_dialog(FileDialogAction::SaveProjectAs) {
            self.status_message = "Save As dialog is already active".to_string();
            return;
        }

        tracing::info!("Requested save project dialog");
        self.pending_file_dialogs.push(PendingFileDialog {
            action: FileDialogAction::SaveProjectAs,
            handle: self.file_dialogs.save_project_as(),
        });
        self.status_message = "Choose a destination for the project...".to_string();
    }

    /// Polls pending file dialogs and applies any completed results.
    ///
    /// Returns `true` when at least one dialog finished during this call.
    pub fn poll_file_dialogs(&mut self) -> bool {
        if self.pending_file_dialogs.is_empty() {
            return false;
        }

        let pending = std::mem::take(&mut self.pending_file_dialogs);
        let mut completed = Vec::new();
        let mut still_pending = Vec::with_capacity(pending.len());
        for dialog in pending {
            match dialog.handle.try_take() {
                Some(outcome) => completed.push((dialog.action, outcome)),
                None => still_pending.push(dialog),
            }
        }
        self.pending_file_dialogs = still_pending;

        let any_completed = !completed.is_empty();
        for (action, outcome) in completed {
            self.apply_file_dialog_outcome(action, outcome);
        }
        any_completed
    }

    fn has_pending_dialog(&self, action: FileDialogAction) -> bool {
        self.pending_file_dialogs
            .iter()
            .any(|dialog| dialog.action == action)
    }

    fn apply_file_dialog_outcome(&mut self, action: FileDialogAction, outcome: FileDialogOutcome) {
        match outcome {
            FileDialogOutcome::Selected(path) => match action {
                FileDialogAction::OpenProject => {
                    if let Err(error) = self.open_project_from_path(&path) {
                        tracing::error!(
                            %error,
                            path = %path.display(),
                            "Failed to open project from dialog"
                        );
                        self.status_message = format!("Failed to open project: {error}");
                    }
                }
                FileDialogAction::SaveProjectAs => {
                    let path = normalize_project_path(path);
                    if let Err(error) = self.save_project_to_path(&path) {
                        tracing::error!(
                            %error,
                            path = %path.display(),
                            "Failed to save project from dialog"
                        );
                        self.status_message = format!("Failed to save project: {error}");
                    }
                }
            },
            FileDialogOutcome::Cancelled => {
                tracing::info!("File dialog cancelled");
                self.status_message = "File dialog cancelled".to_string();
            }
            FileDialogOutcome::Unavailable => {
                tracing::warn!("File dialog unavailable in this environment");
                self.status_message =
                    "File dialog unavailable in this environment (headless mode)".to_string();
            }
        }
    }

    /// Opens or focuses a document tab. Guarantees the same `DocKind` never opens twice.
    pub fn open_document(&mut self, kind: DocKind) {
        if self.open_documents.contains_key(&kind) {
            // Already open: focus the tab in dock_state
            self.focus_document(kind);
            return;
        }

        let doc = match kind {
            DocKind::Level(level_id) => {
                let mut view = lc_feature_level::LevelDocumentView::new(level_id);
                view.add_handler(Box::new(lc_feature_tileset::TilesLayerHandler::new()));
                view.add_handler(Box::new(lc_feature_tileset::IntGridLayerHandler::new()));
                view.add_handler(Box::new(lc_feature_entities::EntitiesLayerHandler::new()));
                view.add_inspector_section(Box::new(
                    lc_feature_entities::EntityInspectorSection::new(),
                ));
                view.set_tool(Box::new(crate::tool_router::LevelCanvasToolRouter::new()));
                Box::new(view) as Box<dyn DocumentView>
            }
            _ => self.registry.create_document(kind).unwrap_or_else(|| {
                Box::new(crate::doc_tab_viewer::PlaceholderDocumentView { kind })
            }),
        };
        self.open_documents.insert(kind, doc);
        self.dock_state.push_to_first_leaf(kind);
        self.status_message = format!("Opened {kind:?}");
    }

    /// Automatically opens the first level of the project into a document tab and focuses it.
    pub fn open_first_level(&mut self) {
        if let Some(level_id) = self
            .project
            .worlds
            .first()
            .and_then(|w| w.levels.first())
            .map(|l| l.id)
        {
            self.open_document(DocKind::Level(level_id));
            self.session.active_level = Some(level_id);
            self.session.active_document = Some(DocKind::Level(level_id));

            if let Some(first_layer) = self.project.level(level_id).and_then(|l| l.layers.first()) {
                self.session.active_layer = Some(first_layer.def_id);
            }
        }
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
        // Apply results from asynchronous file dialogs without blocking this frame.
        self.poll_file_dialogs();
        if !self.pending_file_dialogs.is_empty() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Ensure registered panels are tracked in dock layout
        for panel in self.registry.panels() {
            self.dock_layout
                .ensure_panel(panel.id(), DockSlotLocation::Left);
        }

        crate::menu::handle_global_shortcuts(self, ui.ctx());

        let tokens = theme::Tokens::get(ui.ctx());

        // 1. Top menu bar
        Panel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(tokens.bg)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| {
                crate::menu::render_menu_bar(self, ui);
            });
        // 2. Bottom status bar
        Panel::bottom("status_bar")
            .exact_size(theme::STATUS_H)
            .frame(
                egui::Frame::new()
                    .fill(tokens.bg)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| {
                crate::status_bar::render_status_bar(self, ui);
            });
        let mut dock_geometry = crate::dock_panels::DockGeometry::default();

        // 3. Left dock panel
        if !self.dock_layout.left.panels.is_empty() {
            Panel::left("left_dock")
                .resizable(true)
                .default_size(220.0)
                .frame(
                    egui::Frame::new()
                        .fill(tokens.panel)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ui, |ui| {
                    dock_geometry.left = Some(crate::dock_panels::render_dock_slot(
                        self,
                        ui,
                        DockSlotLocation::Left,
                    ));
                });
        }
        // 4. Right dock panel
        if !self.dock_layout.right.panels.is_empty() {
            Panel::right("right_dock")
                .resizable(true)
                .default_size(260.0)
                .frame(
                    egui::Frame::new()
                        .fill(tokens.panel)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ui, |ui| {
                    dock_geometry.right = Some(crate::dock_panels::render_dock_slot(
                        self,
                        ui,
                        DockSlotLocation::Right,
                    ));
                });
        }
        // 5. Bottom dock (collapsible)
        dock_geometry.bottom = crate::dock_panels::render_bottom_dock(self, ui);
        // 6. Central document area (egui_dock)
        CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(tokens.bg)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ui, |ui| {
                self.render_central_documents(ui);
            });

        // 7. Drag preview, drop indicator and drop completion.
        crate::dock_panels::handle_drag_and_drop(self, ui, &dock_geometry);

        // 8. Process deferred commands and events
        self.process_commands_and_events();
    }

    fn render_central_documents(&mut self, ui: &mut Ui) {
        self.last_central_rect = Some(ui.max_rect());

        if self.open_documents.is_empty() && self.dock_state.iter_all_tabs().next().is_none() {
            self.session.active_document = None;
            self.session.active_level = None;
            crate::workspace::render_workspace_empty(self, ui);
            return;
        }

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

        let dock_style = crate::dock_style::style(&theme::Tokens::get(ui.ctx()));
        DockArea::new(&mut self.dock_state)
            .id(Id::new("central_document_dock_area"))
            .style(dock_style)
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
