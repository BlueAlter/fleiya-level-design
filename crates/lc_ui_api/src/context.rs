//! Editor context providing read access to the project, session state, assets, and facilities to emit commands and events.

use crate::assets::AssetService;
use crate::events::EditorEvent;
use crate::selection::Selection;
use crate::session::{SessionState, TileSelection};
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::id::{LayerDefId, LevelId};
use lc_core::project::Project;

/// Context passed to UI views and panels during rendering.
///
/// Ensures all mutations go through the command system (`exec`),
/// shared session state is read via `session`, and
/// cross-cutting UI operations go through events (`emit`).
pub struct EditorCtx<'a> {
    project: &'a Project,
    undo_stack: &'a UndoStack,
    session: &'a SessionState,
    events: &'a mut Vec<EditorEvent>,
    pending_commands: &'a mut Vec<Box<dyn Command>>,
    assets: &'a dyn AssetService,
}

impl<'a> EditorCtx<'a> {
    /// Creates a new editor context wrapping references to project state, session state, assets, and queues.
    pub fn new(
        project: &'a Project,
        undo_stack: &'a UndoStack,
        session: &'a SessionState,
        events: &'a mut Vec<EditorEvent>,
        pending_commands: &'a mut Vec<Box<dyn Command>>,
        assets: &'a dyn AssetService,
    ) -> Self {
        Self {
            project,
            undo_stack,
            session,
            events,
            pending_commands,
            assets,
        }
    }

    /// Read-only access to the loaded project data model.
    pub fn project(&self) -> &Project {
        self.project
    }

    /// Read-only access to the undo stack.
    pub fn undo_stack(&self) -> &UndoStack {
        self.undo_stack
    }

    /// Read-only access to shared session state.
    pub fn session(&self) -> &SessionState {
        self.session
    }

    /// Read-only access to asset and texture loading service.
    pub fn assets(&self) -> &dyn AssetService {
        self.assets
    }

    /// Convenience getter for currently focused document kind.
    pub fn active_document(&self) -> Option<DocKind> {
        self.session.active_document
    }

    /// Convenience getter for currently active level ID.
    pub fn active_level(&self) -> Option<LevelId> {
        self.session.active_level
    }

    /// Convenience getter for currently active layer definition ID.
    pub fn active_layer(&self) -> Option<LayerDefId> {
        self.session.active_layer
    }

    /// Convenience getter for currently active selection.
    pub fn selection(&self) -> &Selection {
        &self.session.selection
    }

    /// Convenience getter for currently active tool name.
    pub fn active_tool_name(&self) -> &str {
        &self.session.active_tool_name
    }

    /// Convenience getter for currently selected tiles in palette.
    pub fn tile_selection(&self) -> Option<&TileSelection> {
        self.session.tile_selection.as_ref()
    }

    /// Returns true if an undo operation is currently possible.
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    /// Returns true if a redo operation is currently possible.
    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    /// Submits a command to be executed by the editor undo system.
    pub fn exec(&mut self, cmd: impl Command + 'static) {
        tracing::debug!("exec command: {}", cmd.label());
        self.pending_commands.push(Box::new(cmd));
    }

    /// Submits a pre-boxed command to be executed by the editor undo system.
    pub fn exec_boxed(&mut self, cmd: Box<dyn Command>) {
        tracing::debug!("exec command: {}", cmd.label());
        self.pending_commands.push(cmd);
    }

    /// Emits an event to the shell event queue.
    pub fn emit(&mut self, event: EditorEvent) {
        self.events.push(event);
    }

    /// Emits a status bar message event.
    pub fn status(&mut self, msg: impl Into<String>) {
        self.events.push(EditorEvent::Status(msg.into()));
    }

    /// Logs an informational message via `tracing`.
    pub fn log_info(&self, msg: &str) {
        tracing::info!("{msg}");
    }

    /// Logs a warning message via `tracing`.
    pub fn log_warn(&self, msg: &str) {
        tracing::warn!("{msg}");
    }

    /// Logs an error message via `tracing`.
    pub fn log_error(&self, msg: &str) {
        tracing::error!("{msg}");
    }
}
