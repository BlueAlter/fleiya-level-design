//! Inspector section interface for floating properties window.

use crate::context::EditorCtx;
use crate::selection::Selection;

/// A section rendered inside the floating inspector window.
pub trait InspectorSection: Send {
    /// Returns true if this section is applicable to the current selection.
    fn applies(&self, sel: &Selection) -> bool;

    /// Renders the section controls and field editors.
    fn ui(&mut self, ui: &mut egui::Ui, cx: &mut EditorCtx, sel: &Selection);
}
