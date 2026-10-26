//! Pure geometry and selection helpers shared by the entity tools.
//!
//! These functions do not contain egui widgets; they only translate between
//! world/level coordinates, snap to the layer grid and query entity instances.

use glam::{IVec2, UVec2};
use lc_core::defs::LayerDef;
use lc_core::id::{EntityId, LayerDefId};
use lc_core::layer::LayerData;
use lc_core::level::Level;
use lc_core::project::Project;
use lc_ui_api::canvas::CanvasCtx;
use lc_ui_api::selection::Selection;

/// Grid size in pixels for a layer definition.
pub fn grid_size(project: &Project, _level: &Level, layer_def_id: LayerDefId) -> IVec2 {
    project
        .defs
        .layer(layer_def_id)
        .map(|def: &LayerDef| IVec2::new(def.grid_size.x as i32, def.grid_size.y as i32))
        .unwrap_or_else(|| IVec2::splat(16))
}

/// Level-local pointer position, if the pointer is over the canvas.
pub fn local_pos(cv: &CanvasCtx, level: &Level) -> Option<IVec2> {
    cv.world_pos.map(|world| world - level.world_pos)
}

/// Snaps a level-local point to the grid.
pub fn snap_to_grid(point: IVec2, grid: IVec2) -> IVec2 {
    let grid = grid.max(IVec2::ONE);
    IVec2::new(
        (point.x as f32 / grid.x as f32).round() as i32 * grid.x,
        (point.y as f32 / grid.y as f32).round() as i32 * grid.y,
    )
}

pub(crate) fn snap_size(size: IVec2, grid: IVec2) -> IVec2 {
    let grid = grid.max(IVec2::ONE);
    IVec2::new(
        (size.x as f32 / grid.x as f32).round().max(1.0) as i32 * grid.x,
        (size.y as f32 / grid.y as f32).round().max(1.0) as i32 * grid.y,
    )
}

/// Clamps a top-left position so the entity stays inside the level.
pub fn clamp_position(position: IVec2, size: UVec2, level_size: UVec2) -> IVec2 {
    let max_x = level_size.x.saturating_sub(size.x) as i32;
    let max_y = level_size.y.saturating_sub(size.y) as i32;
    IVec2::new(position.x.clamp(0, max_x), position.y.clamp(0, max_y))
}

/// Topmost visible entity under a level-local point.
pub fn pick_entity_at(level: &Level, point: IVec2) -> Option<EntityId> {
    for layer in level.layers.iter().rev() {
        if !layer.visible {
            continue;
        }
        if let LayerData::Entities(entities) = &layer.data {
            for entity in entities.iter().rev() {
                let min = entity.pos_px;
                let max = min + entity.size_px.as_ivec2();
                if point.x >= min.x && point.y >= min.y && point.x < max.x && point.y < max.y {
                    return Some(entity.id);
                }
            }
        }
    }
    None
}

/// Entities layer containing `id`, if any.
pub(crate) fn entity_layer_of(level: &Level, id: EntityId) -> Option<LayerDefId> {
    level
        .layers
        .iter()
        .find(|layer| {
            matches!(&layer.data, LayerData::Entities(entities) if entities.iter().any(|e| e.id == id))
        })
        .map(|layer| layer.def_id)
}

/// Selected ids that actually exist in `level`.
pub fn selected_in_level(selection: &Selection, level: &Level) -> Vec<EntityId> {
    selection
        .entities()
        .map(|ids| {
            ids.iter()
                .copied()
                .filter(|id| level.entity(*id).is_some())
                .collect()
        })
        .unwrap_or_default()
}
