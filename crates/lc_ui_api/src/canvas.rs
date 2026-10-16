//! Canvas interfaces for layer rendering and interactive tools.

use crate::context::EditorCtx;
use crate::viewport::Viewport;
use glam::IVec2;
use lc_core::id::{LayerDefId, LevelId};
use lc_core::layer::{LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;

/// Transient context provided to a `CanvasTool` on interactive input.
pub struct CanvasCtx<'a> {
    /// Active viewport for converting between screen, world, and grid.
    pub viewport: &'a Viewport,
    /// Screen position of the pointer, if inside canvas rect.
    pub pointer_pos: Option<egui::Pos2>,
    /// World pixel coordinates under the pointer, if any.
    pub world_pos: Option<IVec2>,
    /// Grid cell coordinates under the pointer, if any.
    pub grid_pos: Option<IVec2>,
    /// Current keyboard modifiers.
    pub modifiers: egui::Modifiers,
    /// egui response of the canvas interaction rect.
    pub response: &'a egui::Response,
    /// Currently active layer definition ID, if any.
    pub active_layer_id: Option<LayerDefId>,
    /// ID of the level being edited.
    pub active_level_id: LevelId,
}

/// An interactive tool operating on the level canvas (e.g., paint, erase, move, select).
pub trait CanvasTool: Send {
    /// Unique name of the tool (e.g. "Paint", "Erase", "Select").
    fn name(&self) -> &str;

    /// Dispatches pointer and keyboard input events to the tool.
    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx);
}

/// Context provided to a `LayerHandler` to render layer graphics on canvas.
pub struct LayerRenderCtx<'a> {
    /// egui painter to draw primitives onto.
    pub painter: &'a egui::Painter,
    /// Active viewport for coordinate projection.
    pub viewport: &'a Viewport,
    /// The layer instance being rendered.
    pub layer: &'a LayerInstance,
    /// The parent level containing this layer.
    pub level: &'a Level,
    /// Full project data model for resolving tilesets, enums, or entity definitions.
    pub project: &'a Project,
    /// Whether this layer is the currently selected active layer.
    pub is_active: bool,
    /// Visual opacity multiplier in `0.0..=1.0`.
    pub opacity: f32,
}

/// Handler responsible for rendering a specific `LayerKind` and supplying its available canvas tools.
pub trait LayerHandler: Send + Sync {
    /// The kind of layer handled by this implementor.
    fn kind(&self) -> LayerKind;

    /// Renders the visual representation of this layer onto the canvas.
    fn render(&self, ctx: &mut LayerRenderCtx);

    /// Instantiates the tools provided by this layer handler.
    fn tools(&self) -> Vec<Box<dyn CanvasTool>>;
}
