//! Project-wide layer helpers for naming, IntGrid value allocation, and usage inspection.

use lc_core::defs::IntGridValueDef;
use lc_core::id::LayerDefId;
use lc_core::layer::LayerData;
use lc_core::project::Project;

/// Summary of where a layer definition is used across project levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayerUsageStats {
    /// Total number of level layer instances referencing this `LayerDef`.
    pub total_instances: usize,
    /// Number of levels containing at least one non-empty cell, tile, or entity in this layer.
    pub non_empty_levels: usize,
    /// Total count of non-zero IntGrid cells, placed tiles, or placed entities across all levels.
    pub placed_items: usize,
}

/// Counts level instances and placed data for `layer_def_id` across all worlds and levels.
pub fn count_layer_def_usage(project: &Project, layer_def_id: LayerDefId) -> LayerUsageStats {
    let mut stats = LayerUsageStats::default();
    for world in &project.worlds {
        for level in &world.levels {
            if let Some(instance) = level.layer(layer_def_id) {
                stats.total_instances += 1;
                let count = match &instance.data {
                    LayerData::IntGrid(grid) => grid.cells().iter().filter(|&&v| v != 0).count(),
                    LayerData::Tiles(grid) => grid.cells().iter().filter(|t| t.is_some()).count(),
                    LayerData::Entities(entities) => entities.len(),
                    LayerData::Auto => 0,
                };
                if count > 0 {
                    stats.non_empty_levels += 1;
                    stats.placed_items += count;
                }
            }
        }
    }
    stats
}

/// Generates a unique layer identifier not present in `existing_names`.
pub fn generate_unique_layer_name(existing_names: &[String], base: &str) -> String {
    let trimmed = base.trim();
    let seed = if trimmed.is_empty() { "Layer" } else { trimmed };
    let candidate = format!("{seed}_copy");
    if !existing_names
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&candidate))
    {
        return candidate;
    }

    let mut idx = 2usize;
    loop {
        let next = format!("{seed}_copy_{idx}");
        if !existing_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&next))
        {
            return next;
        }
        idx += 1;
    }
}

/// Returns the next positive integer ID for a new `IntGridValueDef` (minimum 1; 0 is empty).
pub fn next_int_grid_value_id(values: &[IntGridValueDef]) -> i32 {
    values
        .iter()
        .map(|v| v.value)
        .filter(|&v| v > 0)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}
