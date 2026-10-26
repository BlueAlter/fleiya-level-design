//! lc_feature_entities: Entity definition tab, reusable field editors, entity
//! layer support, and the floating entity inspector.

#![forbid(unsafe_code)]

pub mod commands;
pub mod field_commands;
pub mod field_defs;
pub mod field_editors;
pub mod field_instance_commands;
pub mod geometry;
pub mod handler;
pub mod helpers;
pub mod inspector;
pub mod instance_commands;
pub mod panel;
pub mod refs;
pub mod stroke;
pub mod tools;
pub mod view;

pub use commands::{
    entity_def_usage, AddEntityDefCommand, RemoveEntityDefCommand, UpdateEntityDefCommand,
};
pub use field_commands::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, UpdateEntityFieldCommand,
};
pub use field_defs::{
    coerce_field_value, default_field_value, default_field_values, format_field_value,
};
pub use field_editors::{
    entity_choice_label, entity_choices, entity_def_label, field_type_editor, field_type_label,
    field_value_editor, normalized_value, EntityChoice, FieldEditorContext, FieldEditorResponse,
};
pub use field_instance_commands::{EntityFieldUpdate, SetEntityFieldValuesCommand};
pub use handler::EntitiesLayerHandler;
pub use helpers::{count_entity_def_instances, find_entity_def_instances, EntityLocation};
pub use inspector::EntityInspectorSection;
pub use instance_commands::{
    AddEntityInstancesCommand, RemoveEntityInstancesCommand, SetEntityPositionsCommand,
    SetEntitySizesCommand,
};
pub use panel::EntitiesPanel;
pub use stroke::{next_stroke_id, FocusStrokeTracker, StrokeTracker};
pub use tools::{
    active_entities_layer, clamp_position, grid_size, local_pos, pick_entity_at, placement_def_id,
    selected_in_level, snap_to_grid, EntityCanvasTool,
};
pub use view::EntityDefDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the entity definition document factory, the entity panel, the
/// entity layer handler, and the entity inspector section.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::EntityDef(entity_def_id) => {
            Some(Box::new(EntityDefDocumentView::new(entity_def_id)))
        }
        _ => None,
    });

    registry.register_panel(Box::new(EntitiesPanel::new()));
    registry.register_layer_handler(Box::new(EntitiesLayerHandler::new()));
    registry.register_inspector_section(Box::new(EntityInspectorSection::new()));
}
