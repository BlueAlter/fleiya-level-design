//! File schema format definitions for multi-file JSON storage.

use glam::{IVec2, UVec2};
use lc_core::{
    Definitions, FieldValues, LayerInstance, LevelId, ProjectSettings, WorldId, WorldLayout,
};
use serde::{Deserialize, Serialize};

/// Root file representation: `project.lcproj`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Schema format version.
    pub version: u32,
    /// Project title.
    pub name: String,
    /// Project-wide global settings.
    pub settings: ProjectSettings,
    /// Definitions and templates (tilesets, enums, entities, layers).
    pub defs: Definitions,
    /// Manifest of worlds contained in this project.
    pub worlds: Vec<WorldManifestEntry>,
}

/// World reference entry stored inside `project.lcproj`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldManifestEntry {
    /// World ID.
    pub id: WorldId,
    /// World name.
    pub name: String,
    /// Relative path to `world.json` from project root directory.
    pub file: String,
}

/// World file representation: `worlds/<world>/world.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldFile {
    /// Schema format version.
    pub version: u32,
    /// World identifier.
    pub id: WorldId,
    /// World title.
    pub name: String,
    /// World layout arrangement style.
    pub layout: WorldLayout,
    /// Default room grid size in pixels, if configured.
    pub default_grid: Option<UVec2>,
    /// Manifest of levels belonging to this world.
    pub levels: Vec<LevelManifestEntry>,
}

/// Level reference entry stored inside `world.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelManifestEntry {
    /// Level identifier.
    pub id: LevelId,
    /// Level title.
    pub name: String,
    /// Position within the world coordinate plane.
    pub world_pos: IVec2,
    /// Z-depth ordering in the world.
    pub world_depth: i32,
    /// Relative path to level JSON from the containing `world.json` directory.
    pub file: String,
}

/// Level file representation: `worlds/<world>/levels/<lvl>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelFile {
    /// Schema format version.
    pub version: u32,
    /// Level identifier.
    pub id: LevelId,
    /// Level title.
    pub name: String,
    /// Position within world.
    pub world_pos: IVec2,
    /// Z-depth ordering.
    pub world_depth: i32,
    /// Level dimensions in pixels.
    pub size_px: UVec2,
    /// Ordered layer instances.
    pub layers: Vec<LayerInstance>,
    /// Custom level field values.
    pub fields: FieldValues,
}
