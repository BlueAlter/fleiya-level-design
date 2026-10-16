//! Editor events emitted by UI panels and views to trigger shell actions.

use crate::selection::Selection;
use lc_core::command::DocKind;

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
    /// Status bar notification message.
    Status(String),
    /// Request to trigger project save.
    SaveProject,
}
