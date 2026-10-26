//! lc_feature_entities: Entity definition tab, reusable field editors, and
//! entity layer support.

#![forbid(unsafe_code)]

pub mod commands;
pub mod field_commands;
pub mod field_defs;
pub mod field_editors;
pub mod helpers;
pub mod stroke;
pub mod view;

pub use commands::{
    entity_def_usage, AddEntityDefCommand, RemoveEntityDefCommand, UpdateEntityDefCommand,
};
pub use field_commands::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, UpdateEntityFieldCommand,
};
pub use field_defs::{coerce_field_value, default_field_value, format_field_value};
pub use field_editors::{
    entity_choice_label, entity_choices, entity_def_label, field_type_editor, field_type_label,
    field_value_editor, normalized_value, EntityChoice, FieldEditorContext, FieldEditorResponse,
};
pub use helpers::{count_entity_def_instances, find_entity_def_instances, EntityLocation};
pub use stroke::{next_stroke_id, StrokeTracker};
pub use view::EntityDefDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the entity definition document view factory with the registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::EntityDef(entity_def_id) => {
            Some(Box::new(EntityDefDocumentView::new(entity_def_id)))
        }
        _ => None,
    });
}
