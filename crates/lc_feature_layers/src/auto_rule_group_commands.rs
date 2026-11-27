//! Undoable commands for `AutoLayerRuleGroup` management.

use lc_core::auto_rules::AutoLayerRuleGroup;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{AutoRuleGroupId, LayerDefId};
use lc_core::project::Project;

use crate::auto_rule_commands::reorder_by_key;

/// Adds an `AutoLayerRuleGroup` to a `LayerDef`.
#[derive(Debug, Clone)]
pub struct AddAutoRuleGroupCommand {
    layer_id: LayerDefId,
    group: AutoLayerRuleGroup,
    inserted_index: Option<usize>,
}

impl AddAutoRuleGroupCommand {
    /// Creates a command appending `group` to `layer_id`.
    pub fn new(layer_id: LayerDefId, group: AutoLayerRuleGroup) -> Self {
        Self {
            layer_id,
            group,
            inserted_index: None,
        }
    }
}

impl Command for AddAutoRuleGroupCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let idx = layer.auto_rule_groups.len();
        self.inserted_index = Some(idx);
        layer.auto_rule_groups.push(self.group.clone());
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(layer) = p.defs.layer_mut(self.layer_id) {
            if let Some(idx) = self.inserted_index.take() {
                if idx < layer.auto_rule_groups.len()
                    && layer.auto_rule_groups[idx].id == self.group.id
                {
                    layer.auto_rule_groups.remove(idx);
                    return;
                }
            }
            if let Some(pos) = layer
                .auto_rule_groups
                .iter()
                .position(|g| g.id == self.group.id)
            {
                layer.auto_rule_groups.remove(pos);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Rule Group \"{}\"", self.group.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Updates properties of an existing `AutoLayerRuleGroup`.
#[derive(Debug, Clone)]
pub struct UpdateAutoRuleGroupCommand {
    layer_id: LayerDefId,
    updated: AutoLayerRuleGroup,
    previous: Option<AutoLayerRuleGroup>,
    stroke_id: Option<u64>,
}

impl UpdateAutoRuleGroupCommand {
    /// Creates a non-merging update command.
    pub fn new(layer_id: LayerDefId, updated: AutoLayerRuleGroup) -> Self {
        Self {
            layer_id,
            updated,
            previous: None,
            stroke_id: None,
        }
    }

    /// Creates a mergeable update command tagged with a UI stroke identifier.
    pub fn with_stroke(layer_id: LayerDefId, updated: AutoLayerRuleGroup, stroke_id: u64) -> Self {
        Self {
            layer_id,
            updated,
            previous: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for UpdateAutoRuleGroupCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let group = layer.rule_group_mut(self.updated.id).ok_or_else(|| {
            CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.updated.id))
        })?;
        if self.previous.is_none() {
            self.previous = Some(group.clone());
        }
        *group = self.updated.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                if let Some(group) = layer.rule_group_mut(self.updated.id) {
                    *group = prev;
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.layer_id == other.layer_id
                && self.updated.id == other.updated.id
                && self.stroke_id.is_some()
                && self.stroke_id == other.stroke_id
            {
                self.updated = other.updated.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Update Rule Group \"{}\"", self.updated.name)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes an `AutoLayerRuleGroup` from a `LayerDef`.
#[derive(Debug, Clone)]
pub struct RemoveAutoRuleGroupCommand {
    layer_id: LayerDefId,
    group_id: AutoRuleGroupId,
    removed: Option<(usize, AutoLayerRuleGroup)>,
}

impl RemoveAutoRuleGroupCommand {
    /// Creates a command to remove `group_id` from `layer_id`.
    pub fn new(layer_id: LayerDefId, group_id: AutoRuleGroupId) -> Self {
        Self {
            layer_id,
            group_id,
            removed: None,
        }
    }
}

impl Command for RemoveAutoRuleGroupCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let idx = layer
            .auto_rule_groups
            .iter()
            .position(|g| g.id == self.group_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.group_id)))?;
        let group = layer.auto_rule_groups.remove(idx);
        self.removed = Some((idx, group));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some((idx, group)) = self.removed.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                let at = idx.min(layer.auto_rule_groups.len());
                layer.auto_rule_groups.insert(at, group);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Remove Rule Group".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Reorders `AutoLayerRuleGroup`s inside a `LayerDef`.
#[derive(Debug, Clone)]
pub struct ReorderAutoRuleGroupsCommand {
    layer_id: LayerDefId,
    new_order: Vec<AutoRuleGroupId>,
    previous_order: Option<Vec<AutoRuleGroupId>>,
}

impl ReorderAutoRuleGroupsCommand {
    /// Creates a command setting the group order to `new_order`.
    pub fn new(layer_id: LayerDefId, new_order: Vec<AutoRuleGroupId>) -> Self {
        Self {
            layer_id,
            new_order,
            previous_order: None,
        }
    }
}

impl Command for ReorderAutoRuleGroupsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        self.previous_order = Some(layer.auto_rule_groups.iter().map(|g| g.id).collect());
        reorder_by_key(&mut layer.auto_rule_groups, &self.new_order, |g| g.id);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous_order.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                reorder_by_key(&mut layer.auto_rule_groups, &prev, |g| g.id);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Reorder Rule Groups".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
