//! egui_dock TabViewer implementation for central DocumentView tabs.

use egui::{Id, Ui, WidgetText};
use egui_dock::TabViewer;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::project::Project;
use lc_ui_api::assets::AssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;
use std::collections::HashMap;

/// Renders document tabs inside the central `egui_dock::DockArea`.
pub struct DocTabViewer<'a> {
    pub project: &'a Project,
    pub open_documents: &'a mut HashMap<DocKind, Box<dyn DocumentView>>,
    pub undo_stack: &'a UndoStack,
    pub session: &'a SessionState,
    pub events: &'a mut Vec<EditorEvent>,
    pub pending_commands: &'a mut Vec<Box<dyn Command>>,
    pub assets: &'a dyn AssetService,
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
                self.session,
                self.events,
                self.pending_commands,
                self.assets,
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

/// Fallback placeholder DocumentView implementation for shell tabs.
pub struct PlaceholderDocumentView {
    /// Document kind this view represents.
    pub kind: DocKind,
}

impl DocumentView for PlaceholderDocumentView {
    fn doc_kind(&self) -> DocKind {
        self.kind
    }

    fn title(&self, project: &Project) -> String {
        match self.kind {
            DocKind::Level(id) => project
                .level(id)
                .map(|l| l.name.clone())
                .unwrap_or_else(|| format!("Level({id})")),
            DocKind::World(id) => project
                .world(id)
                .map(|w| w.name.clone())
                .unwrap_or_else(|| format!("World({id})")),
            DocKind::EntityDef(id) => project
                .defs
                .entities
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("EntityDef({id})")),
            DocKind::EnumDef(id) => project
                .defs
                .enums
                .iter()
                .find(|e| e.id == id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("EnumDef({id})")),
            DocKind::TilesetDef(id) => project
                .defs
                .tilesets
                .iter()
                .find(|t| t.id == id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("TilesetDef({id})")),
            DocKind::LayerDef(id) => project
                .defs
                .layers
                .iter()
                .find(|l| l.id == id)
                .map(|l| l.name.clone())
                .unwrap_or_else(|| format!("LayerDef({id})")),
            DocKind::ProjectSettings => "Project Settings".to_string(),
        }
    }

    fn is_valid(&self, project: &Project) -> bool {
        match self.kind {
            DocKind::Level(id) => project.level(id).is_some(),
            DocKind::World(id) => project.world(id).is_some(),
            DocKind::EntityDef(id) => project.defs.entities.iter().any(|e| e.id == id),
            DocKind::EnumDef(id) => project.defs.enums.iter().any(|e| e.id == id),
            DocKind::TilesetDef(id) => project.defs.tilesets.iter().any(|t| t.id == id),
            DocKind::LayerDef(id) => project.defs.layers.iter().any(|l| l.id == id),
            DocKind::ProjectSettings => true,
        }
    }

    fn ui(&mut self, ui: &mut Ui, _cx: &mut EditorCtx) {
        ui.heading(format!("{:?}", self.kind));
        ui.label("Document view placeholder.");
    }
}
