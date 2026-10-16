//! Editor context providing read access to the project and facilities to emit commands and events.

use crate::events::EditorEvent;
use lc_core::command::{Command, UndoStack};
use lc_core::project::Project;

/// Context passed to UI views and panels during rendering.
///
/// Ensures all mutations go through the command system (`exec`) and
/// cross-cutting UI operations go through events (`emit`).
pub struct EditorCtx<'a> {
    project: &'a Project,
    undo_stack: &'a UndoStack,
    events: &'a mut Vec<EditorEvent>,
    pending_commands: &'a mut Vec<Box<dyn Command>>,
}

impl<'a> EditorCtx<'a> {
    /// Creates a new editor context wrapping references to project state and queues.
    pub fn new(
        project: &'a Project,
        undo_stack: &'a UndoStack,
        events: &'a mut Vec<EditorEvent>,
        pending_commands: &'a mut Vec<Box<dyn Command>>,
    ) -> Self {
        Self {
            project,
            undo_stack,
            events,
            pending_commands,
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
        self.pending_commands.push(Box::new(cmd));
    }

    /// Submits a pre-boxed command to be executed by the editor undo system.
    pub fn exec_boxed(&mut self, cmd: Box<dyn Command>) {
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
