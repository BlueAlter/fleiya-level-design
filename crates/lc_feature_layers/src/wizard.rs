//! LDtk-style Auto-Layer Rules Assistant (`AUTO-LAYER RULES ASSISTANT`) and 3x3 Wizard.

use std::collections::BTreeMap;

use lc_core::auto_rules::{AutoLayerRuleDef, AutoLayerRuleGroup, AutoTileRect};
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{AutoRuleGroupId, AutoRuleId, LayerDefId};
use lc_core::project::Project;

/// Preset layout template for the Auto-Layer Rules Assistant / Wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoRuleWizardPreset {
    /// Visual slot-mapped 4-bit + inner-corner autotile assistant (LDtk Assistant parity).
    AssistantSlots,
    /// Single center-match fill rule (`[1]` at center).
    SimpleFill,
    /// Horizontal platform strip: left cap, middle, right cap, and isolated 1-wide tile.
    PlatformHorizontal,
    /// Classic 3x3 borders around a solid terrain body (top/bottom/left/right/corners + center fill).
    #[default]
    SolidBorders3x3,
}

impl AutoRuleWizardPreset {
    /// Human-readable preset title.
    pub const fn label(self) -> &'static str {
        match self {
            Self::AssistantSlots => "Visual Autotile Assistant (Mapped Slots)",
            Self::SimpleFill => "Simple Fill",
            Self::PlatformHorizontal => "Platform (Left / Mid / Right)",
            Self::SolidBorders3x3 => "Solid 3x3 Borders + Fill",
        }
    }
}

/// Visual slot definition inside Section 3 (`AUTO-RENDERED TILES`) of the LDtk Rules Assistant.
#[derive(Debug, Clone, Copy)]
pub struct AutotileSlotSpec {
    /// Short slot identifier/tooltip.
    pub label: &'static str,
    /// Visual grid group (`0` = primary 4x4 borders/caps/fill, `1` = secondary 4x4 inner-corners/edges).
    pub panel: u8,
    /// Grid column (`0..4`) inside the panel.
    pub col: u8,
    /// Grid row (`0..4`) inside the panel.
    pub row: u8,
    /// Relative 3x3 conditions `(px, py, is_inner_required)` (`true` = main value, `false` = outer/forbid).
    pub pattern: &'static [(u8, u8, bool)],
}

/// Standard 15 primary + 4 inner-corner slot templates matching LDtk's `AUTO-LAYER RULES ASSISTANT`.
pub const AUTOTILE_ASSISTANT_SLOTS: &[AutotileSlotSpec] = &[
    // Panel 0 (Primary 4x4: outer borders, 3x3 fill, vertical & horizontal caps)
    AutotileSlotSpec {
        label: "Top-Left Corner",
        panel: 0,
        col: 0,
        row: 0,
        pattern: &[
            (1, 0, false),
            (0, 1, false),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Top Edge",
        panel: 0,
        col: 1,
        row: 0,
        pattern: &[
            (1, 0, false),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Top-Right Corner",
        panel: 0,
        col: 2,
        row: 0,
        pattern: &[
            (1, 0, false),
            (0, 1, true),
            (1, 1, true),
            (2, 1, false),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Vertical Top Cap",
        panel: 0,
        col: 3,
        row: 0,
        pattern: &[
            (1, 0, false),
            (0, 1, false),
            (1, 1, true),
            (2, 1, false),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Left Edge",
        panel: 0,
        col: 0,
        row: 1,
        pattern: &[
            (1, 0, true),
            (0, 1, false),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Center Fill",
        panel: 0,
        col: 1,
        row: 1,
        pattern: &[(1, 1, true)],
    },
    AutotileSlotSpec {
        label: "Right Edge",
        panel: 0,
        col: 2,
        row: 1,
        pattern: &[
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, false),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Vertical Middle",
        panel: 0,
        col: 3,
        row: 1,
        pattern: &[
            (1, 0, true),
            (0, 1, false),
            (1, 1, true),
            (2, 1, false),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Bottom-Left Corner",
        panel: 0,
        col: 0,
        row: 2,
        pattern: &[
            (1, 0, true),
            (0, 1, false),
            (1, 1, true),
            (2, 1, true),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Bottom Edge",
        panel: 0,
        col: 1,
        row: 2,
        pattern: &[
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Bottom-Right Corner",
        panel: 0,
        col: 2,
        row: 2,
        pattern: &[
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, false),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Vertical Bottom Cap",
        panel: 0,
        col: 3,
        row: 2,
        pattern: &[
            (1, 0, true),
            (0, 1, false),
            (1, 1, true),
            (2, 1, false),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Horizontal Left Cap",
        panel: 0,
        col: 0,
        row: 3,
        pattern: &[
            (1, 0, false),
            (0, 1, false),
            (1, 1, true),
            (2, 1, true),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Horizontal Middle",
        panel: 0,
        col: 1,
        row: 3,
        pattern: &[
            (1, 0, false),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Horizontal Right Cap",
        panel: 0,
        col: 2,
        row: 3,
        pattern: &[
            (1, 0, false),
            (0, 1, true),
            (1, 1, true),
            (2, 1, false),
            (1, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Single Isolated Block",
        panel: 0,
        col: 3,
        row: 3,
        pattern: &[
            (1, 0, false),
            (0, 1, false),
            (1, 1, true),
            (2, 1, false),
            (1, 2, false),
        ],
    },
    // Panel 1 (Secondary: Inner Corners & Corner Accents)
    AutotileSlotSpec {
        label: "Inner Corner Bottom-Right",
        panel: 1,
        col: 0,
        row: 0,
        pattern: &[
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
            (2, 2, false),
        ],
    },
    AutotileSlotSpec {
        label: "Inner Corner Bottom-Left",
        panel: 1,
        col: 1,
        row: 0,
        pattern: &[
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (0, 2, false),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Inner Corner Top-Right",
        panel: 1,
        col: 0,
        row: 1,
        pattern: &[
            (1, 0, true),
            (2, 0, false),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
        ],
    },
    AutotileSlotSpec {
        label: "Inner Corner Top-Left",
        panel: 1,
        col: 1,
        row: 1,
        pattern: &[
            (0, 0, false),
            (1, 0, true),
            (0, 1, true),
            (1, 1, true),
            (2, 1, true),
            (1, 2, true),
        ],
    },
];

/// Parameters for generating an `AutoLayerRuleGroup` from the Assistant / Wizard.
#[derive(Debug, Clone)]
pub struct AutoRuleWizardSpec {
    /// Target group name.
    pub group_name: String,
    /// Main IntGrid value matched by the generated rules (`>= 1`).
    pub int_grid_value: i32,
    /// Optional outer IntGrid value required outside borders (`None` = forbid `int_grid_value`).
    pub outer_int_grid_value: Option<i32>,
    /// Preset layout strategy.
    pub preset: AutoRuleWizardPreset,
    /// Base tile IDs used by non-slot presets (center/fill, top, side, corner, etc.).
    pub sample_tiles: Vec<u32>,
    /// Explicit mapped tiles per `AUTOTILE_ASSISTANT_SLOTS` index (for `AssistantSlots`).
    pub slot_tiles: BTreeMap<usize, u32>,
    /// Whether generated rules stack instead of stopping cascade (`!break_on_match`).
    pub allow_stack: bool,
}

impl Default for AutoRuleWizardSpec {
    fn default() -> Self {
        Self {
            group_name: "AutoTerrain".to_string(),
            int_grid_value: 1,
            outer_int_grid_value: None,
            preset: AutoRuleWizardPreset::SolidBorders3x3,
            sample_tiles: Vec::new(),
            slot_tiles: BTreeMap::new(),
            allow_stack: false,
        }
    }
}

/// Builds an `AutoLayerRuleGroup` from `spec` using deterministic IDs starting after `first_rule_id`.
pub fn build_wizard_group(
    group_id: AutoRuleGroupId,
    first_rule_id: u64,
    spec: &AutoRuleWizardSpec,
) -> AutoLayerRuleGroup {
    let v = spec.int_grid_value.max(1);
    let outer_cond = spec.outer_int_grid_value.unwrap_or(-v);
    let tile_at = |idx: usize| -> u32 {
        spec.sample_tiles
            .get(idx)
            .or_else(|| spec.sample_tiles.first())
            .copied()
            .unwrap_or(0)
    };

    let mut group = AutoLayerRuleGroup::new(group_id, spec.group_name.clone());
    group.uses_wizard = true;
    let mut next_rid = first_rule_id.max(1);
    let mut make_rule = |coords: &[(u8, u8, i32)], tile_id: u32, flip_x: bool, flip_y: bool| {
        let mut r = AutoLayerRuleDef::new(AutoRuleId(next_rid));
        next_rid = next_rid.saturating_add(1);
        for &(px, py, cond) in coords {
            r.set_pattern_at(px, py, cond);
        }
        r.flip_x = flip_x;
        r.flip_y = flip_y;
        r.break_on_match = !spec.allow_stack;
        r.out_of_bounds_value = Some(0);
        r.tile_rects.push(AutoTileRect::single(tile_id));
        r
    };

    match spec.preset {
        AutoRuleWizardPreset::AssistantSlots => {
            // Emit mapped slots in reverse order (specific corners/caps before general Center Fill)
            // Wait: Center Fill is at index 5 in AUTOTILE_ASSISTANT_SLOTS, so push specific rules first and Center Fill last!
            let mut center_fill: Option<AutoLayerRuleDef> = None;
            for (slot_idx, slot) in AUTOTILE_ASSISTANT_SLOTS.iter().enumerate() {
                let Some(&tile_id) = spec.slot_tiles.get(&slot_idx) else {
                    continue;
                };
                let coords: Vec<(u8, u8, i32)> = slot
                    .pattern
                    .iter()
                    .map(|&(px, py, is_inner)| (px, py, if is_inner { v } else { outer_cond }))
                    .collect();
                let rule = make_rule(&coords, tile_id, false, false);
                if slot.pattern.len() == 1 {
                    center_fill = Some(rule);
                } else {
                    group.rules.push(rule);
                }
            }
            if let Some(fill) = center_fill {
                group.rules.push(fill);
            }
        }
        AutoRuleWizardPreset::SimpleFill => {
            group
                .rules
                .push(make_rule(&[(1, 1, v)], tile_at(0), false, false));
        }
        AutoRuleWizardPreset::PlatformHorizontal => {
            group.rules.push(make_rule(
                &[(0, 1, outer_cond), (1, 1, v), (2, 1, outer_cond)],
                tile_at(0),
                false,
                false,
            ));
            group.rules.push(make_rule(
                &[(0, 1, outer_cond), (1, 1, v), (2, 1, v)],
                tile_at(1),
                true,
                false,
            ));
            group.rules.push(make_rule(
                &[(0, 1, v), (1, 1, v), (2, 1, v)],
                tile_at(2),
                false,
                false,
            ));
        }
        AutoRuleWizardPreset::SolidBorders3x3 => {
            group.rules.push(make_rule(
                &[(1, 0, outer_cond), (0, 1, outer_cond), (1, 1, v)],
                tile_at(1),
                true,
                true,
            ));
            group.rules.push(make_rule(
                &[(1, 0, outer_cond), (1, 1, v), (1, 2, v)],
                tile_at(2),
                false,
                true,
            ));
            group.rules.push(make_rule(
                &[(0, 1, outer_cond), (1, 1, v), (2, 1, v)],
                tile_at(3),
                true,
                false,
            ));
            group
                .rules
                .push(make_rule(&[(1, 1, v)], tile_at(0), false, false));
        }
    }

    group
}

/// Applies a generated Wizard rule group onto a `LayerDef` as one undoable step.
#[derive(Debug, Clone)]
pub struct ApplyAutoRuleWizardCommand {
    layer_id: LayerDefId,
    group: AutoLayerRuleGroup,
    inserted_index: Option<usize>,
}

impl ApplyAutoRuleWizardCommand {
    /// Creates a command appending a wizard-generated rule group to `layer_id`.
    pub fn new(layer_id: LayerDefId, group: AutoLayerRuleGroup) -> Self {
        Self {
            layer_id,
            group,
            inserted_index: None,
        }
    }
}

impl Command for ApplyAutoRuleWizardCommand {
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
        "Apply Auto-Rule Assistant".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
