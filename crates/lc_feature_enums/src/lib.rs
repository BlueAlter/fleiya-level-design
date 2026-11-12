//! lc_feature_enums: Enum definition tab, commands, and reference protection.

#![forbid(unsafe_code)]

pub mod commands;
pub mod view;

pub use commands::{
    find_enum_def_references, find_enum_value_references, AddEnumDefCommand, AddEnumValueCommand,
    EnumReferenceLocation, RemoveEnumDefCommand, RemoveEnumValueCommand, RenameEnumValueIdCommand,
    ReorderEnumDefsCommand, ReorderEnumValuesCommand, UpdateEnumDefCommand, UpdateEnumValueCommand,
};
pub use view::EnumDefDocumentView;

use lc_core::command::DocKind;
use lc_ui_api::registry::Registry;

/// Registers the Enum definition document view factory with the registry.
pub fn register(registry: &mut Registry) {
    registry.register_document_factory(|kind| match kind {
        DocKind::EnumDef(enum_id) => Some(Box::new(EnumDefDocumentView::new(enum_id))),
        _ => None,
    });
}
