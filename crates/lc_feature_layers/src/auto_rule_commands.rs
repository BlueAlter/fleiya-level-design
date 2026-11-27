//! Undoable commands for individual `AutoLayerRuleDef` management.

use lc_core::auto_rules::AutoLayerRuleDef;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{AutoRuleGroupId, AutoRuleId, LayerDefId};
use lc_core::project::Project;

/// Adds an `AutoLayerRuleDef` to a specific `AutoLayerRuleGroup`.
#[derive(Debug, Clone)]
pub struct AddAutoRuleCommand {
    layer_id: LayerDefId,
    group_id: AutoRuleGroupId,
    rule: AutoLayerRuleDef,
    inserted_index: Option<usize>,
}

impl AddAutoRuleCommand {
    /// Creates a command appending `rule` to `group_id` on `layer_id`.
    pub fn new(layer_id: LayerDefId, group_id: AutoRuleGroupId, rule: AutoLayerRuleDef) -> Self {
        Self {
            layer_id,
            group_id,
            rule,
            inserted_index: None,
        }
    }
}

impl Command for AddAutoRuleCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let group = layer
            .rule_group_mut(self.group_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.group_id)))?;
        let idx = group.rules.len();
        self.inserted_index = Some(idx);
        group.rules.push(self.rule.clone());
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(layer) = p.defs.layer_mut(self.layer_id) {
            if let Some(group) = layer.rule_group_mut(self.group_id) {
                if let Some(idx) = self.inserted_index.take() {
                    if idx < group.rules.len() && group.rules[idx].id == self.rule.id {
                        group.rules.remove(idx);
                        return;
                    }
                }
                if let Some(pos) = group.rules.iter().position(|r| r.id == self.rule.id) {
                    group.rules.remove(pos);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Add Auto-Tile Rule".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Updates an existing `AutoLayerRuleDef` inside a rule group.
#[derive(Debug, Clone)]
pub struct UpdateAutoRuleCommand {
    layer_id: LayerDefId,
    group_id: AutoRuleGroupId,
    updated: AutoLayerRuleDef,
    previous: Option<AutoLayerRuleDef>,
    stroke_id: Option<u64>,
}

impl UpdateAutoRuleCommand {
    /// Creates a non-merging rule update command.
    pub fn new(layer_id: LayerDefId, group_id: AutoRuleGroupId, updated: AutoLayerRuleDef) -> Self {
        Self {
            layer_id,
            group_id,
            updated,
            previous: None,
            stroke_id: None,
        }
    }

    /// Creates a mergeable rule update command tagged with a UI stroke identifier.
    pub fn with_stroke(
        layer_id: LayerDefId,
        group_id: AutoRuleGroupId,
        updated: AutoLayerRuleDef,
        stroke_id: u64,
    ) -> Self {
        Self {
            layer_id,
            group_id,
            updated,
            previous: None,
            stroke_id: Some(stroke_id),
        }
    }
}

impl Command for UpdateAutoRuleCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let group = layer
            .rule_group_mut(self.group_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.group_id)))?;
        let rule = group
            .rule_mut(self.updated.id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRule {:?}", self.updated.id)))?;
        if self.previous.is_none() {
            self.previous = Some(rule.clone());
        }
        *rule = self.updated.clone();
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                if let Some(group) = layer.rule_group_mut(self.group_id) {
                    if let Some(rule) = group.rule_mut(self.updated.id) {
                        *rule = prev;
                    }
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.layer_id == other.layer_id
                && self.group_id == other.group_id
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
        "Update Auto-Tile Rule".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes an `AutoLayerRuleDef` from a rule group.
#[derive(Debug, Clone)]
pub struct RemoveAutoRuleCommand {
    layer_id: LayerDefId,
    group_id: AutoRuleGroupId,
    rule_id: AutoRuleId,
    removed: Option<(usize, AutoLayerRuleDef)>,
}

impl RemoveAutoRuleCommand {
    /// Creates a command removing `rule_id` from `group_id`.
    pub fn new(layer_id: LayerDefId, group_id: AutoRuleGroupId, rule_id: AutoRuleId) -> Self {
        Self {
            layer_id,
            group_id,
            rule_id,
            removed: None,
        }
    }
}

impl Command for RemoveAutoRuleCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let group = layer
            .rule_group_mut(self.group_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.group_id)))?;
        let idx = group
            .rules
            .iter()
            .position(|r| r.id == self.rule_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRule {:?}", self.rule_id)))?;
        let rule = group.rules.remove(idx);
        self.removed = Some((idx, rule));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some((idx, rule)) = self.removed.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                if let Some(group) = layer.rule_group_mut(self.group_id) {
                    let at = idx.min(group.rules.len());
                    group.rules.insert(at, rule);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Remove Auto-Tile Rule".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Reorders `AutoLayerRuleDef`s inside a rule group.
#[derive(Debug, Clone)]
pub struct ReorderAutoRulesCommand {
    layer_id: LayerDefId,
    group_id: AutoRuleGroupId,
    new_order: Vec<AutoRuleId>,
    previous_order: Option<Vec<AutoRuleId>>,
}

impl ReorderAutoRulesCommand {
    /// Creates a command setting rule order inside `group_id`.
    pub fn new(
        layer_id: LayerDefId,
        group_id: AutoRuleGroupId,
        new_order: Vec<AutoRuleId>,
    ) -> Self {
        Self {
            layer_id,
            group_id,
            new_order,
            previous_order: None,
        }
    }
}

impl Command for ReorderAutoRulesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let layer = p
            .defs
            .layer_mut(self.layer_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        let group = layer
            .rule_group_mut(self.group_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("AutoRuleGroup {:?}", self.group_id)))?;
        self.previous_order = Some(group.rules.iter().map(|r| r.id).collect());
        reorder_by_key(&mut group.rules, &self.new_order, |r| r.id);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.previous_order.take() {
            if let Some(layer) = p.defs.layer_mut(self.layer_id) {
                if let Some(group) = layer.rule_group_mut(self.group_id) {
                    reorder_by_key(&mut group.rules, &prev, |r| r.id);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Reorder Auto-Tile Rules".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(crate) fn reorder_by_key<T, K: PartialEq>(
    items: &mut Vec<T>,
    order: &[K],
    key_fn: impl Fn(&T) -> K,
) {
    let mut reordered = Vec::with_capacity(items.len());
    for key in order {
        if let Some(pos) = items.iter().position(|item| key_fn(item) == *key) {
            reordered.push(items.remove(pos));
        }
    }
    reordered.append(items);
    *items = reordered;
}
