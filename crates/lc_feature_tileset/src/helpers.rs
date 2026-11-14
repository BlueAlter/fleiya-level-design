//! Pure helper functions for tileset grid metrics, usage counts, mode availability, and naming.

use glam::UVec2;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;

/// Computed grid metrics for an atlas image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridMetrics {
    /// Total number of tile columns.
    pub cols: u32,
    /// Total number of tile rows.
    pub rows: u32,
    /// Total number of tiles in the grid (`cols * rows`).
    pub total_tiles: u32,
    /// Unused pixels on the horizontal axis.
    pub leftover_x: u32,
    /// Unused pixels on the vertical axis.
    pub leftover_y: u32,
}

/// Computes the grid metrics (cols, rows, total, leftover pixels) for an image.
pub fn compute_tileset_grid(
    image_size: [u32; 2],
    tile_size: UVec2,
    spacing: u32,
    padding: u32,
) -> GridMetrics {
    let [img_w, img_h] = image_size;
    let tile_w = tile_size.x;
    let tile_h = tile_size.y;

    if tile_w == 0 || tile_h == 0 {
        return GridMetrics {
            cols: 0,
            rows: 0,
            total_tiles: 0,
            leftover_x: img_w,
            leftover_y: img_h,
        };
    }

    let eff_w = img_w.saturating_sub(padding * 2);
    let eff_h = img_h.saturating_sub(padding * 2);
    let stride_x = tile_w + spacing;
    let stride_y = tile_h + spacing;

    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(0);
    let rows = (eff_h + spacing).checked_div(stride_y).unwrap_or(0);
    let total_tiles = cols * rows;

    let used_w = if cols > 0 {
        padding * 2 + cols * tile_w + (cols - 1) * spacing
    } else {
        0
    };
    let used_h = if rows > 0 {
        padding * 2 + rows * tile_h + (rows - 1) * spacing
    } else {
        0
    };

    let leftover_x = img_w.saturating_sub(used_w);
    let leftover_y = img_h.saturating_sub(used_h);

    GridMetrics {
        cols,
        rows,
        total_tiles,
        leftover_x,
        leftover_y,
    }
}

/// Counts how many LayerDefs and EnumDefs reference the given tileset definition.
///
/// Returns `(layers_count, enums_count)`.
pub fn tileset_usage_counts(project: &Project, id: TilesetDefId) -> (usize, usize) {
    let layers = project
        .defs
        .layers
        .iter()
        .filter(|l| l.tileset_id == Some(id))
        .count();

    let enums = project
        .defs
        .enums
        .iter()
        .filter(|e| {
            e.values
                .iter()
                .any(|v| v.tile.as_ref().map(|t| t.tileset_id) == Some(id))
        })
        .count();

    (layers, enums)
}

/// Entry for the tileset right panel mode dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeEntry {
    /// Internal mode identifier.
    pub id: &'static str,
    /// Localized or display label.
    pub label: &'static str,
    /// Whether this mode is currently active and selectable.
    pub available: bool,
    /// Tooltip or explanation hint (e.g. "Coming in a later update").
    pub hint: &'static str,
}

/// Returns the available editor modes for a given tileset definition.
pub fn mode_availability(_tileset: &TilesetDef) -> Vec<ModeEntry> {
    vec![
        ModeEntry {
            id: "None",
            label: "None",
            available: true,
            hint: "",
        },
        ModeEntry {
            id: "Marking",
            label: "Marking",
            available: false,
            hint: "Coming in a later update",
        },
        ModeEntry {
            id: "Animation",
            label: "Animation",
            available: false,
            hint: "Coming in a later update",
        },
    ]
}

/// Generates a unique tileset name, appending `" copy"` or `" copy N"` if the name already exists.
pub fn generate_unique_tileset_name(existing_names: &[String], base: &str) -> String {
    if !existing_names.iter().any(|n| n == base) {
        return base.to_string();
    }

    let copy_base = if base.ends_with(" copy") {
        base.to_string()
    } else {
        format!("{base} copy")
    };

    if !existing_names.iter().any(|n| n == &copy_base) {
        return copy_base;
    }

    let mut i = 2;
    loop {
        let candidate = format!("{copy_base} {i}");
        if !existing_names.iter().any(|n| n == &candidate) {
            return candidate;
        }
        i += 1;
    }
}
