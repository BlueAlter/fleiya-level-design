//! LDtk-compatible Auto-Layer Rule data structures and pattern helpers.

use crate::entity::TilesetRect;
use crate::id::{AutoRuleGroupId, AutoRuleId};
use serde::{Deserialize, Serialize};

/// Sentinel pattern value matching any non-zero IntGrid value (`-ANY_NON_ZERO_PATTERN` requires empty `0`).
pub const ANY_NON_ZERO_PATTERN: i32 = 1_000_000;

/// Base offset used to encode `IntGridValueGroupDef` references inside `AutoLayerRuleDef::pattern`.
///
/// Positive `GROUP_PATTERN_BASE + group_uid` requires any value belonging to `group_uid`.
/// Negative `-(GROUP_PATTERN_BASE + group_uid)` forbids any value belonging to `group_uid`.
pub const GROUP_PATTERN_BASE: i32 = 1_000_000;

fn default_true() -> bool {
    true
}

fn default_one_u32() -> u32 {
    1
}

fn default_one_f32() -> f32 {
    1.0
}

fn default_half_f32() -> f32 {
    0.5
}

fn default_rule_size() -> u8 {
    3
}

fn default_perlin_scale() -> f32 {
    0.2
}

fn default_perlin_octaves() -> u32 {
    2
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_true(v: &bool) -> bool {
    *v
}

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}

fn is_one_u32(v: &u32) -> bool {
    *v == 1
}

fn is_one_f32(v: &f32) -> bool {
    (*v - 1.0).abs() <= f32::EPSILON
}

fn is_half_f32(v: &f32) -> bool {
    (*v - 0.5).abs() <= f32::EPSILON
}

/// Logical group of IntGrid values (`IntGridValueGroupDef` in LDtk).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntGridValueGroupDef {
    /// Positive unique group ID (`>= 1`).
    pub uid: u32,
    /// Human-readable group identifier (e.g. "Walls", "Terrain").
    pub name: String,
    /// Optional RGBA badge color for UI display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 4]>,
}

impl IntGridValueGroupDef {
    /// Creates a new IntGrid value group definition.
    pub fn new(uid: u32, name: impl Into<String>) -> Self {
        Self {
            uid,
            name: name.into(),
            color: None,
        }
    }
}

/// How an auto-rule interprets its candidate output tile entries (`TileMode` in LDtk).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AutoRuleTileMode {
    /// Each entry represents a single tile ID picked randomly.
    #[default]
    Single,
    /// Each entry represents a rectangular multi-tile stamp (`w x h`).
    Stamp,
}

/// Checkerboard matching constraint (`CheckerMode` in LDtk).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AutoRuleCheckerMode {
    /// No checkerboard filtering.
    #[default]
    None,
    /// Alternating checkerboard along horizontal rows.
    Horizontal,
    /// Alternating checkerboard along vertical columns.
    Vertical,
}

/// How a rule group's `required_biome_values` are evaluated (`biomeRequirementMode` in LDtk).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BiomeRequirementMode {
    /// Rule group runs only if the level matches at least one listed biome value (when non-empty).
    #[default]
    Require,
    /// Rule group is skipped if the level matches any listed biome value.
    Exclude,
}

/// One candidate output tile or rectangular stamp inside `AutoLayerRuleDef::tile_rects`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AutoTileRect {
    /// Row-major tile IDs (`w * h` elements; 1 element in `Single` mode).
    pub tiles: Vec<u32>,
    /// Stamp width in tiles (`1` in `Single` mode).
    #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
    pub w: u32,
    /// Stamp height in tiles (`1` in `Single` mode).
    #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
    pub h: u32,
}

impl AutoTileRect {
    /// Creates a single-tile candidate.
    pub fn single(tile_id: u32) -> Self {
        Self {
            tiles: vec![tile_id],
            w: 1,
            h: 1,
        }
    }

    /// Creates a rectangular stamp candidate.
    pub fn stamp(tiles: Vec<u32>, w: u32, h: u32) -> Self {
        Self {
            tiles,
            w: w.max(1),
            h: h.max(1),
        }
    }

    /// Returns the tile ID at `(lx, ly)` inside the stamp.
    pub fn tile_at(&self, lx: u32, ly: u32) -> Option<u32> {
        if lx >= self.w || ly >= self.h {
            return None;
        }
        let idx = (ly as usize)
            .checked_mul(self.w as usize)?
            .checked_add(lx as usize)?;
        self.tiles.get(idx).copied()
    }
}

/// Definition of a single LDtk-compatible Auto-Layer rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayerRuleDef {
    /// Stable rule identifier.
    pub id: AutoRuleId,
    /// Whether this rule is evaluated.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub active: bool,
    /// Odd pattern matrix dimension (`1`, `3`, `5`, or `7`).
    #[serde(default = "default_rule_size")]
    pub size: u8,
    /// Row-major `size * size` pattern conditions (`0` = ignore, `>0` = require, `<0` = forbid).
    #[serde(default)]
    pub pattern: Vec<i32>,
    /// Candidate output tiles or stamps chosen randomly on match.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tile_rects: Vec<AutoTileRect>,
    /// Single tile vs multi-tile stamp mode.
    #[serde(default)]
    pub tile_mode: AutoRuleTileMode,
    /// Stamp horizontal pivot (`0.0..=1.0`).
    #[serde(default = "default_half_f32", skip_serializing_if = "is_half_f32")]
    pub pivot_x: f32,
    /// Stamp vertical pivot (`0.0..=1.0`).
    #[serde(default = "default_half_f32", skip_serializing_if = "is_half_f32")]
    pub pivot_y: f32,
    /// Output tile opacity (`0.0..=1.0`).
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub alpha: f32,
    /// Deterministic probability (`0.0..=1.0`) for applying the rule when matched.
    #[serde(default = "default_one_f32", skip_serializing_if = "is_one_f32")]
    pub chance: f32,
    /// When `true`, stops evaluating subsequent rules for the matched cell.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub break_on_match: bool,
    /// Allow matching horizontally mirrored pattern variants.
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_x: bool,
    /// Allow matching vertically mirrored pattern variants.
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_y: bool,
    /// Horizontal cell evaluation stride (`>= 1`).
    #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
    pub x_modulo: u32,
    /// Vertical cell evaluation stride (`>= 1`).
    #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
    pub y_modulo: u32,
    /// Horizontal modulo phase offset.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub x_offset: i32,
    /// Vertical modulo phase offset.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub y_offset: i32,
    /// Checkerboard filtering mode.
    #[serde(default)]
    pub checker: AutoRuleCheckerMode,
    /// Assumed IntGrid value when pattern cells sample outside level boundaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_of_bounds_value: Option<i32>,
    /// Enable Perlin noise spatial filtering.
    #[serde(default, skip_serializing_if = "is_false")]
    pub perlin_active: bool,
    /// Perlin noise seed.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub perlin_seed: i32,
    /// Perlin noise spatial frequency scale.
    #[serde(default = "default_perlin_scale")]
    pub perlin_scale: f32,
    /// Perlin noise octave count (`>= 1`).
    #[serde(default = "default_perlin_octaves")]
    pub perlin_octaves: u32,
}

impl Default for AutoLayerRuleDef {
    fn default() -> Self {
        Self {
            id: AutoRuleId(0),
            active: true,
            size: 3,
            pattern: vec![0; 9],
            tile_rects: Vec::new(),
            tile_mode: AutoRuleTileMode::Single,
            pivot_x: 0.5,
            pivot_y: 0.5,
            alpha: 1.0,
            chance: 1.0,
            break_on_match: true,
            flip_x: false,
            flip_y: false,
            x_modulo: 1,
            y_modulo: 1,
            x_offset: 0,
            y_offset: 0,
            checker: AutoRuleCheckerMode::None,
            out_of_bounds_value: None,
            perlin_active: false,
            perlin_seed: 0,
            perlin_scale: 0.2,
            perlin_octaves: 2,
        }
    }
}

impl AutoLayerRuleDef {
    /// Creates a new `3x3` active rule with the given ID.
    pub fn new(id: AutoRuleId) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }

    /// Normalizes `size` to supported odd values (`1`, `3`, `5`, `7`).
    pub const fn normalized_size(size: u8) -> u8 {
        match size {
            0..=1 => 1,
            2..=3 => 3,
            4..=5 => 5,
            _ => 7,
        }
    }

    /// Reads the pattern condition at `(px, py)` in `0..self.size`.
    pub fn pattern_at(&self, px: u8, py: u8) -> i32 {
        let sz = Self::normalized_size(self.size) as usize;
        if (px as usize) >= sz || (py as usize) >= sz {
            return 0;
        }
        let idx = (py as usize) * sz + (px as usize);
        self.pattern.get(idx).copied().unwrap_or(0)
    }

    /// Sets the pattern condition at `(px, py)` in `0..self.size`, ensuring buffer length matches.
    pub fn set_pattern_at(&mut self, px: u8, py: u8, value: i32) {
        let sz = Self::normalized_size(self.size);
        self.size = sz;
        let expected = (sz as usize) * (sz as usize);
        if self.pattern.len() != expected {
            self.pattern.resize(expected, 0);
        }
        if px < sz && py < sz {
            let idx = (py as usize) * (sz as usize) + (px as usize);
            if let Some(cell) = self.pattern.get_mut(idx) {
                *cell = value;
            }
        }
    }

    /// Resizes the pattern matrix (`1`, `3`, `5`, or `7`) while preserving center alignment.
    pub fn resize_pattern(&mut self, target_size: u8) {
        let old_sz = Self::normalized_size(self.size) as i32;
        let new_sz = Self::normalized_size(target_size) as i32;
        if old_sz == new_sz && self.pattern.len() == (new_sz as usize) * (new_sz as usize) {
            self.size = new_sz as u8;
            return;
        }

        let old_half = old_sz / 2;
        let new_half = new_sz / 2;
        let mut next = vec![0i32; (new_sz as usize) * (new_sz as usize)];

        for ny in 0..new_sz {
            for nx in 0..new_sz {
                let ox = nx - new_half + old_half;
                let oy = ny - new_half + old_half;
                if ox >= 0 && oy >= 0 && ox < old_sz && oy < old_sz {
                    let val = self.pattern_at(ox as u8, oy as u8);
                    let n_idx = (ny as usize) * (new_sz as usize) + (nx as usize);
                    if let Some(slot) = next.get_mut(n_idx) {
                        *slot = val;
                    }
                }
            }
        }

        self.size = new_sz as u8;
        self.pattern = next;
    }
}

/// Logical collapsible folder/group of auto-layer rules (`AutoLayerRuleGroup` in LDtk).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayerRuleGroup {
    /// Stable group identifier.
    pub id: AutoRuleGroupId,
    /// Display name of the group.
    pub name: String,
    /// Optional RGBA group accent color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 4]>,
    /// Optional tileset icon rect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<TilesetRect>,
    /// Whether this rule group is active.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub active: bool,
    /// Marks this group as optional (can be toggled per level).
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_optional: bool,
    /// Marks whether this group was generated/managed by the 3x3 Rule Wizard.
    #[serde(default, skip_serializing_if = "is_false")]
    pub uses_wizard: bool,
    /// Biome enum value names controlling when this group executes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_biome_values: Vec<String>,
    /// Whether `required_biome_values` acts as a whitelist (`Require`) or blacklist (`Exclude`).
    #[serde(default)]
    pub biome_requirement_mode: BiomeRequirementMode,
    /// Ordered list of rules inside this group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<AutoLayerRuleDef>,
}

impl Default for AutoLayerRuleGroup {
    fn default() -> Self {
        Self {
            id: AutoRuleGroupId(0),
            name: "New Group".to_string(),
            color: None,
            icon: None,
            active: true,
            is_optional: false,
            uses_wizard: false,
            required_biome_values: Vec::new(),
            biome_requirement_mode: BiomeRequirementMode::Require,
            rules: Vec::new(),
        }
    }
}

impl AutoLayerRuleGroup {
    /// Creates a new active rule group with the given ID and name.
    pub fn new(id: AutoRuleGroupId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            ..Self::default()
        }
    }

    /// Finds a rule by ID.
    pub fn rule(&self, id: AutoRuleId) -> Option<&AutoLayerRuleDef> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Finds a mutable rule by ID.
    pub fn rule_mut(&mut self, id: AutoRuleId) -> Option<&mut AutoLayerRuleDef> {
        self.rules.iter_mut().find(|r| r.id == id)
    }
}

/// Output tile produced by evaluating `AutoLayerRuleGroup`s against an IntGrid.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedAutoTile {
    /// Destination grid X coordinate.
    pub x: i32,
    /// Destination grid Y coordinate.
    pub y: i32,
    /// Resolved tile ID in the layer's tileset.
    pub tile_id: u32,
    /// Horizontal flip flag.
    pub flip_x: bool,
    /// Vertical flip flag.
    pub flip_y: bool,
    /// Opacity (`0.0..=1.0`).
    pub alpha: f32,
    /// Rule ID that generated this tile.
    pub rule_id: AutoRuleId,
}
