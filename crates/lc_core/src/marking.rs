//! Tile marking and tagging data model for tilesets.
//! Deterministic data structures for enum value tile associations.

use crate::defs::TilesetDef;
use crate::id::EnumDefId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Mapping from enum value identifiers to sets of tile indices in a tileset.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TileMarking {
    /// Optional enum definition serving as the source for tile marking values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_enum: Option<EnumDefId>,
    /// Map of enum value identifier to set of tile indices tagged with that value.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, BTreeSet<u32>>,
}

impl TileMarking {
    /// Empty tile marking instance with no source enum and no tags.
    pub const EMPTY: Self = Self {
        source_enum: None,
        tags: BTreeMap::new(),
    };

    /// Creates an empty tile marking.
    pub const fn new() -> Self {
        Self::EMPTY
    }

    /// Returns true if there is no source enum and no tags mapped.
    pub fn is_empty(&self) -> bool {
        self.source_enum.is_none() && self.tags.is_empty()
    }

    /// Checks if a tile has a specific enum value tag.
    pub fn has_tag(&self, value: &str, tile: u32) -> bool {
        self.tags
            .get(value)
            .is_some_and(|tiles| tiles.contains(&tile))
    }

    /// Returns all enum value tags assigned to a specific tile index.
    pub fn tags_at(&self, tile: u32) -> Vec<&str> {
        self.tags
            .iter()
            .filter(|(_, tiles)| tiles.contains(&tile))
            .map(|(val, _)| val.as_str())
            .collect()
    }

    /// Sets or clears a tag for a given enum value and tile index.
    /// Returns true if the marking was changed.
    /// Removes empty tag sets so apply/revert round-trips are byte-identical.
    pub fn set_tag(&mut self, value: &str, tile: u32, active: bool) -> bool {
        if active {
            let tiles = self.tags.entry(value.to_string()).or_default();
            tiles.insert(tile)
        } else if let Some(tiles) = self.tags.get_mut(value) {
            let removed = tiles.remove(&tile);
            if tiles.is_empty() {
                self.tags.remove(value);
            }
            removed
        } else {
            false
        }
    }

    /// Removes all tile markings for a specific enum value.
    /// Returns true if the value was present.
    pub fn remove_value(&mut self, value: &str) -> bool {
        self.tags.remove(value).is_some()
    }

    /// Renames an enum value key, unioning tile sets if the destination exists.
    /// Returns true if the old value was present and renamed.
    pub fn rename_value(&mut self, old_value: &str, new_value: &str) -> bool {
        if old_value == new_value {
            return false;
        }
        if let Some(old_tiles) = self.tags.remove(old_value) {
            if let Some(target_tiles) = self.tags.get_mut(new_value) {
                target_tiles.extend(old_tiles);
            } else {
                self.tags.insert(new_value.to_string(), old_tiles);
            }
            true
        } else {
            false
        }
    }

    /// Sets the source enum definition.
    /// Clears existing marking tags when the source enum changes.
    /// Returns true if the source enum changed.
    pub fn set_source(&mut self, source_enum: Option<EnumDefId>) -> bool {
        if self.source_enum == source_enum {
            false
        } else {
            self.source_enum = source_enum;
            self.tags.clear();
            true
        }
    }

    /// Prunes all tile indices greater than or equal to tile_count.
    /// Cleans up empty tag entries.
    /// Returns true if any tile index was removed.
    pub fn prune_tiles_from(&mut self, tile_count: u32) -> bool {
        let mut changed = false;
        self.tags.retain(|_, tiles| {
            let before = tiles.len();
            tiles.retain(|&id| id < tile_count);
            if tiles.len() != before {
                changed = true;
            }
            !tiles.is_empty()
        });
        changed
    }
}

/// Trims whitespace and rejects empty strings.
pub fn normalize_tag(tag: &str) -> Option<String> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Case-insensitive tag equality comparison.
pub fn tags_eq(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// Derives a unique, case-insensitive sorted list of tags across all tilesets and extra inputs.
pub fn known_tags(tilesets: &[TilesetDef], extra: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let all_tags = tilesets
        .iter()
        .flat_map(|t| t.tags.iter())
        .chain(extra.iter());

    for tag in all_tags {
        if let Some(norm) = normalize_tag(tag) {
            if !result.iter().any(|existing| tags_eq(existing, &norm)) {
                result.push(norm);
            }
        }
    }

    result.sort_by(|a, b| {
        a.to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b))
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{EnumDefId, TilesetDefId};
    use glam::UVec2;

    fn sample_tileset(tags: &[&str]) -> TilesetDef {
        TilesetDef {
            id: TilesetDefId(1),
            name: "TestTileset".to_string(),
            path: "tiles.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            marking: TileMarking::new(),
            custom_data: Default::default(),
            animations: Default::default(),
            collisions: Default::default(),
        }
    }

    #[test]
    fn test_marking_empty_and_state() {
        let mut marking = TileMarking::new();
        assert!(marking.is_empty());

        marking.source_enum = Some(EnumDefId(10));
        assert!(!marking.is_empty());

        marking.source_enum = None;
        assert!(marking.is_empty());

        marking.set_tag("Solid", 5, true);
        assert!(!marking.is_empty());

        marking.set_tag("Solid", 5, false);
        assert!(marking.is_empty());
    }

    #[test]
    fn test_marking_set_tag_and_revert_parity() {
        let mut marking = TileMarking::new();
        marking.source_enum = Some(EnumDefId(10));

        // Adding tag
        assert!(marking.set_tag("Solid", 1, true));
        assert!(!marking.set_tag("Solid", 1, true)); // duplicate is no-op
        assert!(marking.has_tag("Solid", 1));
        assert!(!marking.has_tag("Solid", 2));

        // Multiple tags
        assert!(marking.set_tag("Water", 1, true));
        let mut tags_1 = marking.tags_at(1);
        tags_1.sort();
        assert_eq!(tags_1, vec!["Solid", "Water"]);

        // Reverting removes empty keys
        assert!(marking.set_tag("Water", 1, false));
        assert!(!marking.tags.contains_key("Water"));
        assert_eq!(marking.tags_at(1), vec!["Solid"]);

        assert!(marking.set_tag("Solid", 1, false));
        assert!(marking.tags.is_empty());
    }

    #[test]
    fn test_marking_remove_value() {
        let mut marking = TileMarking::new();
        marking.set_tag("Solid", 1, true);
        marking.set_tag("Solid", 2, true);
        marking.set_tag("Water", 3, true);

        assert!(marking.remove_value("Solid"));
        assert!(!marking.remove_value("Solid"));
        assert!(!marking.has_tag("Solid", 1));
        assert!(marking.has_tag("Water", 3));
    }

    #[test]
    fn test_marking_rename_value() {
        let mut marking = TileMarking::new();
        marking.set_tag("OldVal", 1, true);
        marking.set_tag("OldVal", 2, true);

        assert!(!marking.rename_value("OldVal", "OldVal"));
        assert!(!marking.rename_value("NonExistent", "NewVal"));

        // Rename into empty target
        assert!(marking.rename_value("OldVal", "NewVal"));
        assert!(!marking.has_tag("OldVal", 1));
        assert!(marking.has_tag("NewVal", 1));
        assert!(marking.has_tag("NewVal", 2));

        // Rename into existing target (union)
        marking.set_tag("OtherVal", 2, true);
        marking.set_tag("OtherVal", 3, true);
        assert!(marking.rename_value("OtherVal", "NewVal"));
        assert_eq!(
            marking
                .tags
                .get("NewVal")
                .map(|s| s.iter().copied().collect::<Vec<_>>()),
            Some(vec![1, 2, 3])
        );
    }

    #[test]
    fn test_marking_set_source() {
        let mut marking = TileMarking::new();
        marking.source_enum = Some(EnumDefId(1));
        marking.set_tag("Solid", 1, true);

        // Same source is no-op
        assert!(!marking.set_source(Some(EnumDefId(1))));
        assert!(marking.has_tag("Solid", 1));

        // Changing source clears tags
        assert!(marking.set_source(Some(EnumDefId(2))));
        assert_eq!(marking.source_enum, Some(EnumDefId(2)));
        assert!(marking.tags.is_empty());

        // Clearing source
        assert!(marking.set_source(None));
        assert_eq!(marking.source_enum, None);
    }

    #[test]
    fn test_marking_prune_tiles_from() {
        let mut marking = TileMarking::new();
        marking.set_tag("Solid", 2, true);
        marking.set_tag("Solid", 5, true);
        marking.set_tag("Water", 8, true);

        // Pruning from 10 changes nothing
        assert!(!marking.prune_tiles_from(10));

        // Pruning from 6 drops tile 8 and tile 5
        assert!(marking.prune_tiles_from(5));
        assert!(marking.has_tag("Solid", 2));
        assert!(!marking.has_tag("Solid", 5));
        assert!(!marking.tags.contains_key("Water"));

        // Pruning from 2 drops everything
        assert!(marking.prune_tiles_from(2));
        assert!(marking.tags.is_empty());
    }

    #[test]
    fn test_normalize_tag_and_tags_eq() {
        assert_eq!(normalize_tag(""), None);
        assert_eq!(normalize_tag("   "), None);
        assert_eq!(normalize_tag("  Terrain  "), Some("Terrain".to_string()));

        assert!(tags_eq("Terrain", "terrain"));
        assert!(tags_eq("  solid ", "SOLID  "));
        assert!(!tags_eq("Terrain", "Water"));
    }

    #[test]
    fn test_known_tags_dedup_and_sort() {
        let ts1 = sample_tileset(&["Ground", "Props", "decor"]);
        let ts2 = sample_tileset(&["ground", "WATER"]);
        let extra = vec!["props".to_string(), "Air".to_string(), "   ".to_string()];

        let tags = known_tags(&[ts1, ts2], &extra);
        // Case-insensitive sorted: "Air", "decor", "Ground", "Props", "WATER"
        assert_eq!(tags, vec!["Air", "decor", "Ground", "Props", "WATER"]);
    }
}
