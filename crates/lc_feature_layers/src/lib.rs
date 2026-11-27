//! lc_feature_layers: Layer definition tab, Layers panel, and layer commands.

#![forbid(unsafe_code)]

pub mod auto_rule_commands;
pub mod auto_rule_group_commands;
pub mod commands;
pub mod helpers;
pub mod layer_ops_commands;
pub mod panel;
pub mod view;
pub mod wizard;

pub use auto_rule_commands::{
    AddAutoRuleCommand, RemoveAutoRuleCommand, ReorderAutoRulesCommand, UpdateAutoRuleCommand,
};
pub use auto_rule_group_commands::{
    AddAutoRuleGroupCommand, RemoveAutoRuleGroupCommand, ReorderAutoRuleGroupsCommand,
    UpdateAutoRuleGroupCommand,
};
pub use commands::{
    AddLayerDefCommand, RemoveLayerDefCommand, ReorderLayerDefsCommand,
    ToggleLayerVisibilityCommand, UpdateLayerDefCommand,
};
pub use helpers::{
    count_layer_def_usage, generate_unique_layer_name, next_int_grid_value_id, LayerUsageStats,
};
pub use layer_ops_commands::{BakeAutoLayerCommand, MoveEntitiesToLayerCommand};
pub use panel::LayersPanel;
pub use view::LayerDefDocumentView;
pub use wizard::{
    build_wizard_group, ApplyAutoRuleWizardCommand, AutoRuleWizardPreset, AutoRuleWizardSpec,
    AutotileSlotSpec, AUTOTILE_ASSISTANT_SLOTS,
};

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
