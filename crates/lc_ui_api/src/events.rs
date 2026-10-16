//! Editor events emitted by UI panels and views to trigger shell actions.

use crate::selection::Selection;
use crate::session::TileSelection;
use lc_core::command::DocKind;
use lc_core::id::{LayerDefId, LevelId};

/// High-level events emitted by document views or panels to the shell loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorEvent {
    /// Request to open a document in the central tab bar.
    OpenDocument(DocKind),
    /// Request to focus an existing open document.
    FocusDocument(DocKind),
    /// Request to close an open document tab.
    CloseDocument(DocKind),
    /// Notification that the active selection changed.
    SelectionChanged(Selection),
    /// Updates the currently active level ID.
    SetActiveLevel(Option<LevelId>),
    /// Updates the currently active layer definition ID.
    SetActiveLayer(Option<LayerDefId>),
    /// Updates the currently active tool name.
    SetActiveTool(String),
    /// Updates the tile palette selection.
    SetTileSelection(Option<TileSelection>),
    /// Status bar notification message.
    Status(String),
    /// Request to trigger project save.
    SaveProject,
}
