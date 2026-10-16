//! Command trait, DocKind, and UndoStack implementation.

use crate::error::CoreError;
use crate::id::{EntityDefId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use crate::project::Project;
use serde::{Deserialize, Serialize};

/// Target document kind that can be opened or focused in the central editor tab bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocKind {
    /// Level canvas tab.
    Level(LevelId),
    /// World overview tab.
    World(WorldId),
    /// Entity definition editor tab.
    EntityDef(EntityDefId),
    /// Enum definition editor tab.
    EnumDef(EnumDefId),
    /// Tileset definition editor tab.
    TilesetDef(TilesetDefId),
    /// Layer definition editor tab.
    LayerDef(LayerDefId),
    /// Global project settings tab.
    ProjectSettings,
}

/// An undoable mutation to the project state.
pub trait Command: std::fmt::Debug + Send + Sync + 'static {
    /// Applies the mutation to the project.
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError>;

    /// Reverts the mutation, restoring the project to its prior state.
    fn revert(&mut self, p: &mut Project);

    /// Attempts to merge a consecutive command into this command.
    /// Returns `true` if merged successfully, in which case `next` will not be stored.
    fn merge(&mut self, next: &dyn Command) -> bool;

    /// Human-readable action label (e.g., "Add World", "Move Level").
    fn label(&self) -> String;

    /// Target document to focus when applied or undone.
    fn focus(&self) -> Option<DocKind>;

    /// Downcasting helper for command type matching in merge logic.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Global undo/redo stack managing project mutations.
pub struct UndoStack {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for UndoStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UndoStack")
            .field("undo_count", &self.undo_stack.len())
            .field("redo_count", &self.redo_stack.len())
            .finish()
    }
}

impl UndoStack {
    /// Creates a new, empty undo stack.
    pub const fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Executes and records a command. Merges with the previous command if supported.
    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        project: &mut Project,
    ) -> Result<Option<DocKind>, CoreError> {
        tracing::debug!(command = %command.label(), "Executing command");
        command.apply(project)?;

        let focus = command.focus();

        if let Some(last) = self.undo_stack.last_mut() {
            if last.merge(command.as_ref()) {
                tracing::debug!(command = %last.label(), "Merged command into existing history entry");
                self.redo_stack.clear();
                return Ok(focus);
            }
        }

        self.undo_stack.push(command);
        self.redo_stack.clear();
        Ok(focus)
    }

    /// Undoes the topmost command.
    pub fn undo(&mut self, project: &mut Project) -> Option<(String, Option<DocKind>)> {
        let mut command = self.undo_stack.pop()?;
        tracing::debug!(command = %command.label(), "Reverting command (undo)");
        command.revert(project);
        let label = command.label();
        let focus = command.focus();
        self.redo_stack.push(command);
        Some((label, focus))
    }

    /// Redoes the topmost undone command.
    pub fn redo(
        &mut self,
        project: &mut Project,
    ) -> Result<Option<(String, Option<DocKind>)>, CoreError> {
        let mut command = match self.redo_stack.pop() {
            Some(cmd) => cmd,
            None => return Ok(None),
        };
        tracing::debug!(command = %command.label(), "Re-applying command (redo)");
        if let Err(e) = command.apply(project) {
            // Put it back on redo stack if apply failed
            self.redo_stack.push(command);
            return Err(e);
        }
        let label = command.label();
        let focus = command.focus();
        self.undo_stack.push(command);
        Ok(Some((label, focus)))
    }

    /// Returns `true` if there are operations to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are operations to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the label of the next command that would be undone.
    pub fn undo_label(&self) -> Option<String> {
        self.undo_stack.last().map(|c| c.label())
    }

    /// Returns the label of the next command that would be redone.
    pub fn redo_label(&self) -> Option<String> {
        self.redo_stack.last().map(|c| c.label())
    }

    /// Clears both undo and redo histories (e.g. on project load or revert).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
