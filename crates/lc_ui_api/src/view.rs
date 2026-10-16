//! Document and Panel views hosted in the shell docking layout.

use crate::context::EditorCtx;
use crate::selection::Selection;
use crate::viewport::Viewport;
use lc_core::command::DocKind;
use lc_core::project::Project;

use serde::{Deserialize, Serialize};

/// Unique identifier for dockable panels (e.g. Project, Layers, Console, Palette).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PanelId(pub String);

impl PanelId {
    /// Creates a new panel identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a string slice of the panel identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PanelId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for PanelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A document tab hosted in the central editor dock area.
pub trait DocumentView: Send {
    /// Formatted tab title shown in the document header.
    fn title(&self, p: &Project) -> String;

    /// Renders the document view contents.
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx);

    /// Checks if the underlying document still exists in the project.
    ///
    /// If this returns `false`, the shell automatically closes the tab.
    fn is_valid(&self, p: &Project) -> bool;

    /// Returns the corresponding target document kind.
    fn doc_kind(&self) -> DocKind;
}

/// A dockable side or bottom panel view.
pub trait PanelView: Send {
    /// Unique identifier of the panel.
    fn id(&self) -> PanelId;

    /// User-visible title of the panel tab.
    fn title(&self) -> &str;

    /// Renders the panel contents.
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx);
}

/// Transient per-document view state that is not persisted in the project file.
#[derive(Debug, Clone)]
pub struct ViewState {
    /// Canvas viewport containing pan and zoom.
    pub viewport: Viewport,
    /// Currently active tool name, if any.
    pub active_tool: Option<String>,
    /// Active selection in this view.
    pub selection: Selection,
    /// Position of the floating inspector window, if open.
    pub floating_window_pos: Option<egui::Pos2>,
    /// Whether the floating inspector window is open.
    pub floating_window_open: bool,
    /// Whether the floating inspector window is pinned.
    pub floating_window_pinned: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            viewport: Viewport::default(),
            active_tool: None,
            selection: Selection::default(),
            floating_window_pos: None,
            floating_window_open: true,
            floating_window_pinned: false,
        }
    }
}
