//! World organization and multi-level layout modes.

use crate::defs::LayerDef;
use crate::id::{LevelId, WorldId};
use crate::layer::LayerKind;
use crate::level::Level;
use crate::marking::tags_eq;
use glam::UVec2;
use serde::{Deserialize, Serialize};

/// Arrangement layout mode for levels within a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldLayout {
    /// Arbitrary 2D positioning (freeform coordinates).
    Free,
    /// Grid-aligned positioning (Metroidvania / room-based grid snap).
    GridVania,
    /// Levels ordered along a horizontal line.
    LinearHorizontal,
    /// Levels ordered along a vertical line.
    LinearVertical,
}

/// A world containing a collection of positioned levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct World {
    /// Stable identifier for this world.
    pub id: WorldId,
    /// Display name (e.g. "Overworld", "Nether").
    pub name: String,
    /// Level arrangement layout style.
    pub layout: WorldLayout,
    /// Default grid sizing for GridVania or level snapping.
    pub default_grid: Option<UVec2>,
    /// Levels belonging to this world.
    pub levels: Vec<Level>,
    /// Optional layer tag whitelist (empty = all layers permitted unless excluded).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub included_layer_tags: Vec<String>,
    /// Optional layer tag blacklist (always excludes matching layers, overriding `included_layer_tags`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_layer_tags: Vec<String>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            id: WorldId(0),
            name: String::new(),
            layout: WorldLayout::Free,
            default_grid: None,
            levels: Vec::new(),
            included_layer_tags: Vec::new(),
            excluded_layer_tags: Vec::new(),
        }
    }
}

impl World {
    /// Creates a new world with default Free layout.
    pub fn new(id: WorldId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            ..Self::default()
        }
    }

    /// Looks up a level by its ID.
    pub fn level(&self, id: LevelId) -> Option<&Level> {
        self.levels.iter().find(|l| l.id == id)
    }

    /// Looks up a mutable reference to a level by its ID.
    pub fn level_mut(&mut self, id: LevelId) -> Option<&mut Level> {
        self.levels.iter_mut().find(|l| l.id == id)
    }

    /// Returns `true` if `def` passes this world's `included_layer_tags` and `excluded_layer_tags` filter.
    pub fn layer_matches(&self, def: &LayerDef) -> bool {
        if self
            .excluded_layer_tags
            .iter()
            .any(|ex| def.tags.iter().any(|t| tags_eq(t, ex)))
        {
            return false;
        }
        if self.included_layer_tags.is_empty() {
            return true;
        }
        self.included_layer_tags
            .iter()
            .any(|inc| def.tags.iter().any(|t| tags_eq(t, inc)))
    }

    /// Returns all project `LayerDef`s active in this world, preserving global render order.
    pub fn matching_layer_defs<'a>(&self, defs: &'a [LayerDef]) -> Vec<&'a LayerDef> {
        defs.iter().filter(|d| self.layer_matches(d)).collect()
    }

    /// Returns pairs `(auto_layer, missing_source_layer)` for active Auto layers whose source IntGrid is excluded.
    pub fn missing_auto_sources<'a>(
        &self,
        defs: &'a [LayerDef],
    ) -> Vec<(&'a LayerDef, &'a LayerDef)> {
        let mut missing = Vec::new();
        for def in defs {
            if def.kind != LayerKind::Auto || !self.layer_matches(def) {
                continue;
            }
            if let Some(src_id) = def.auto_source_layer_def_id {
                if let Some(src_def) = defs.iter().find(|d| d.id == src_id) {
                    if !self.layer_matches(src_def) {
                        missing.push((def, src_def));
                    }
                }
            }
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::LayerDefId;

    #[test]
    fn world_layer_tag_filtering_include_exclude_and_empty() {
        let mut tundra = LayerDef::new(LayerDefId(1), "TundraGround", LayerKind::Tiles);
        tundra.tags = vec!["tundra".to_string()];

        let mut shared = LayerDef::new(LayerDefId(2), "Collisions", LayerKind::IntGrid);
        shared.tags = vec!["shared".to_string(), "tundra".to_string()];

        let mut desert = LayerDef::new(LayerDefId(3), "DesertSand", LayerKind::Tiles);
        desert.tags = vec!["desert".to_string()];

        let untagged = LayerDef::new(LayerDefId(4), "Legacy", LayerKind::Entities);

        // Empty include/exclude -> all match
        let world_all = World::new(WorldId(1), "Default");
        assert!(world_all.layer_matches(&tundra));
        assert!(world_all.layer_matches(&shared));
        assert!(world_all.layer_matches(&desert));
        assert!(world_all.layer_matches(&untagged));

        // Include ["Tundra"] + Exclude ["Shared"] -> tundra passes, shared excluded, desert/untagged skipped
        let mut world_tundra = World::new(WorldId(2), "TundraWorld");
        world_tundra.included_layer_tags = vec!["Tundra".to_string()];
        world_tundra.excluded_layer_tags = vec!["SHARED".to_string()];

        assert!(world_tundra.layer_matches(&tundra));
        assert!(!world_tundra.layer_matches(&shared));
        assert!(!world_tundra.layer_matches(&desert));
        assert!(!world_tundra.layer_matches(&untagged));
    }

    #[test]
    fn world_detects_missing_auto_source_layer() {
        let mut src = LayerDef::new(LayerDefId(1), "Collisions", LayerKind::IntGrid);
        src.tags = vec!["desert".to_string()];

        let mut auto = LayerDef::new(LayerDefId(2), "SnowAuto", LayerKind::Auto);
        auto.tags = vec!["tundra".to_string()];
        auto.auto_source_layer_def_id = Some(LayerDefId(1));

        let mut world = World::new(WorldId(1), "TundraWorld");
        world.included_layer_tags = vec!["tundra".to_string()];

        let defs = vec![src, auto];
        let missing = world.missing_auto_sources(&defs);
        assert_eq!(missing.len(), 1);
        assert_eq!(
            missing.first().map(|(a, s)| (a.id, s.id)),
            Some((LayerDefId(2), LayerDefId(1)))
        );
    }
}
