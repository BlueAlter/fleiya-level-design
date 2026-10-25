//! lc_feature_layers: Layer definition tab, Layers panel, and layer commands.

#![forbid(unsafe_code)]

pub mod commands;
pub mod panel;
pub mod view;

pub use commands::{
    AddLayerDefCommand, RemoveLayerDefCommand, ReorderLayerDefsCommand,
    ToggleLayerVisibilityCommand, UpdateLayerDefCommand,
};
pub use panel::LayersPanel;
pub use view::LayerDefDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the layer definition document view factory and layers dockable panel with the registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::LayerDef(layer_def_id) => Some(Box::new(LayerDefDocumentView::new(layer_def_id))),
        _ => None,
    });

    registry.register_panel(Box::new(LayersPanel::new()));
}
