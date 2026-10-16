//! egui_dock TabViewer implementation for central DocumentView tabs.

use egui::{Id, Ui, WidgetText};
use egui_dock::TabViewer;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;

/// Renders document tabs inside the central `egui_dock::DockArea`.
pub struct DocTabViewer<'a> {
    pub project: &'a Project,
    pub open_documents: &'a mut HashMap<DocKind, Box<dyn DocumentView>>,
    pub undo_stack: &'a UndoStack,
    pub events: &'a mut Vec<EditorEvent>,
    pub pending_commands: &'a mut Vec<Box<dyn Command>>,
    pub is_dirty: bool,
}

impl<'a> TabViewer for DocTabViewer<'a> {
    type Tab = DocKind;

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        let title_str = if let Some(doc) = self.open_documents.get(tab) {
            doc.title(self.project)
        } else {
            format!("{tab:?}")
        };

        if self.is_dirty {
            format!("{title_str}*").into()
        } else {
            title_str.into()
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(doc) = self.open_documents.get_mut(tab) {
            let mut cx = EditorCtx::new(
                self.project,
                self.undo_stack,
                self.events,
                self.pending_commands,
            );
            doc.ui(ui, &mut cx);
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Document view not found");
            });
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        // Automatically close tab if the underlying document is no longer valid
        if let Some(doc) = self.open_documents.get(tab) {
            !doc.is_valid(self.project)
        } else {
            true
        }
    }
}
