//! DocumentView implementation for levels in Level Craft.

use egui::Ui;
use lc_core::command::DocKind;
use lc_core::id::LevelId;
use lc_core::project::Project;
use lc_ui_api::canvas::{CanvasTool, LayerHandler};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::view::DocumentView;

use crate::canvas::{render_level_canvas, LevelCanvasState};

/// Central level editor document tab view.
pub struct LevelDocumentView {
    level_id: LevelId,
    state: LevelCanvasState,
    layer_handlers: Vec<Box<dyn LayerHandler>>,
    inspector_sections: Vec<Box<dyn InspectorSection>>,
    active_tool: Option<Box<dyn CanvasTool>>,
}

impl LevelDocumentView {
    /// Constructs a new level document view for the specified level ID.
    pub fn new(level_id: LevelId) -> Self {
        Self {
            level_id,
            state: LevelCanvasState::default(),
            layer_handlers: Vec::new(),
            inspector_sections: Vec::new(),
            active_tool: None,
        }
    }

    /// Attaches a layer handler to this level view in a builder pattern.
    pub fn with_handler(mut self, handler: Box<dyn LayerHandler>) -> Self {
        self.layer_handlers.push(handler);
        self
    }

    /// Registers an additional layer handler.
    pub fn add_handler(&mut self, handler: Box<dyn LayerHandler>) {
        self.layer_handlers.push(handler);
    }

    /// Attaches an inspector section in a builder pattern.
    pub fn with_inspector_section(mut self, section: Box<dyn InspectorSection>) -> Self {
        self.inspector_sections.push(section);
        self
    }

    /// Registers an additional inspector section.
    pub fn add_inspector_section(&mut self, section: Box<dyn InspectorSection>) {
        self.inspector_sections.push(section);
    }

    /// Sets the active interactive tool on the canvas.
    pub fn set_tool(&mut self, tool: Box<dyn CanvasTool>) {
        self.active_tool = Some(tool);
    }

    /// Clears the currently active tool.
    pub fn clear_tool(&mut self) {
        self.active_tool = None;
    }

    /// Read-only access to the currently active tool.
    pub fn tool(&self) -> Option<&dyn CanvasTool> {
        self.active_tool.as_deref()
    }

    /// Read-only access to transient level canvas state.
    pub fn state(&self) -> &LevelCanvasState {
        &self.state
    }

    /// Mutable access to transient level canvas state.
    pub fn state_mut(&mut self) -> &mut LevelCanvasState {
        &mut self.state
    }

    /// Target level ID.
    pub fn level_id(&self) -> LevelId {
        self.level_id
    }
}

impl DocumentView for LevelDocumentView {
    fn title(&self, p: &Project) -> String {
        p.level(self.level_id)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "Level".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.level(self.level_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::Level(self.level_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let level_opt = cx.project().level(self.level_id).cloned();
        if let Some(level) = level_opt {
            render_level_canvas(
                ui,
                &level,
                &mut self.state,
                &self.layer_handlers,
                &mut self.inspector_sections,
                &mut self.active_tool,
                cx,
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Level not found in project");
            });
        }
    }
}
