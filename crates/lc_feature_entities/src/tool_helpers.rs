//! Pick-mode and resize-rect helpers for `EntityCanvasTool`.

use glam::{IVec2, UVec2};
use lc_core::defs::EntityDef;
use lc_core::fields::FieldValue;
use lc_core::id::{EntityDefId, LayerDefId};
use lc_core::layer::{LayerData, LayerKind};
use lc_core::level::Level;
use lc_ui_api::canvas::CanvasCtx;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::CanvasPickMode;

use crate::field_instance_commands::{EntityFieldUpdate, SetEntityFieldValuesCommand};
use crate::geometry::{
    clamp_position_for_def, entity_layer_of, grid_size, local_pos, pick_entity_at_with_defs,
    snap_size, snap_to_grid, ResizeHandle,
};
use crate::helpers::filter_entity_choices;

pub(crate) fn handle_pick_click(
    cv: &mut CanvasCtx,
    cx: &mut EditorCtx,
    pick: &CanvasPickMode,
) -> bool {
    let level_id = cv.active_level_id;
    let Some(level) = cx.project().level(level_id) else {
        return false;
    };
    let Some(local) = local_pos(cv, level) else {
        return false;
    };

    match pick {
        CanvasPickMode::Point {
            level_id: target_lvl,
            entity_id,
            field_name,
            array_index,
        } if *target_lvl == level_id => {
            let grid = entity_layer_of(level, *entity_id)
                .map(|lid| grid_size(cx.project(), level, lid))
                .unwrap_or_else(|| IVec2::splat(16));
            let snapped = if cv.modifiers.shift {
                local
            } else {
                snap_to_grid(local, grid)
            };
            if let Some(ent) = level.entity(*entity_id) {
                let next_val = match array_index {
                    None => FieldValue::Point(snapped),
                    Some(idx) => {
                        let mut items = match ent.fields.get(field_name) {
                            Some(FieldValue::Array(v)) => v.clone(),
                            _ => Vec::new(),
                        };
                        if *idx < items.len() {
                            items[*idx] = FieldValue::Point(snapped);
                        } else {
                            items.push(FieldValue::Point(snapped));
                        }
                        FieldValue::Array(items)
                    }
                };
                cx.exec(SetEntityFieldValuesCommand::new(
                    level_id,
                    vec![EntityFieldUpdate {
                        entity_id: *entity_id,
                        field_name: field_name.clone(),
                        value: next_val,
                    }],
                ));
                cx.emit(EditorEvent::SetCanvasPickMode(None));
                cx.status(format!("Picked point ({}, {})", snapped.x, snapped.y));
                return true;
            }
        }
        CanvasPickMode::EntityRef {
            level_id: target_lvl,
            entity_id,
            field_name,
            array_index,
        } if *target_lvl == level_id => {
            if let Some(hit_id) =
                pick_entity_at_with_defs(cx.project(), level, local, cv.viewport.zoom)
            {
                if let Some(ent) = level.entity(*entity_id) {
                    let allowed = cx
                        .project()
                        .defs
                        .entity(ent.def_id)
                        .and_then(|d| d.field_def(field_name))
                        .is_none_or(|f| {
                            filter_entity_choices(cx.project(), ent.def_id, f)
                                .iter()
                                .any(|c| c.id == hit_id)
                        });
                    if !allowed {
                        cx.status("Entity is not allowed by AllowedRefs filter");
                        return true;
                    }
                    let next_val = match array_index {
                        None => FieldValue::EntityRef(Some(hit_id)),
                        Some(idx) => {
                            let mut items = match ent.fields.get(field_name) {
                                Some(FieldValue::Array(v)) => v.clone(),
                                _ => Vec::new(),
                            };
                            if *idx < items.len() {
                                items[*idx] = FieldValue::EntityRef(Some(hit_id));
                            } else {
                                items.push(FieldValue::EntityRef(Some(hit_id)));
                            }
                            FieldValue::Array(items)
                        }
                    };
                    cx.exec(SetEntityFieldValuesCommand::new(
                        level_id,
                        vec![EntityFieldUpdate {
                            entity_id: *entity_id,
                            field_name: field_name.clone(),
                            value: next_val,
                        }],
                    ));
                    cx.emit(EditorEvent::SetCanvasPickMode(None));
                    cx.status(format!("Linked Entity #{}", hit_id.0));
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_resized_rect(
    start_pos: IVec2,
    start_size: UVec2,
    handle: ResizeHandle,
    delta: IVec2,
    grid: IVec2,
    keep_aspect: bool,
    allow_oob: bool,
    level_size: UVec2,
) -> (IVec2, UVec2) {
    let min_x = grid.x.max(1);
    let min_y = grid.y.max(1);
    let mut left = start_pos.x;
    let mut top = start_pos.y;
    let mut right = start_pos.x + start_size.x as i32;
    let mut bottom = start_pos.y + start_size.y as i32;

    match handle {
        ResizeHandle::East | ResizeHandle::NorthEast | ResizeHandle::SouthEast => {
            right = start_pos.x + snap_size(IVec2::new(start_size.x as i32 + delta.x, 1), grid).x;
        }
        ResizeHandle::West | ResizeHandle::NorthWest | ResizeHandle::SouthWest => {
            let max_left = right - min_x;
            left = snap_to_grid(IVec2::new(start_pos.x + delta.x, 0), grid)
                .x
                .min(max_left);
        }
        _ => {}
    }
    match handle {
        ResizeHandle::South | ResizeHandle::SouthEast | ResizeHandle::SouthWest => {
            bottom = start_pos.y + snap_size(IVec2::new(1, start_size.y as i32 + delta.y), grid).y;
        }
        ResizeHandle::North | ResizeHandle::NorthEast | ResizeHandle::NorthWest => {
            let max_top = bottom - min_y;
            top = snap_to_grid(IVec2::new(0, start_pos.y + delta.y), grid)
                .y
                .min(max_top);
        }
        _ => {}
    }

    let w = (right - left).max(min_x) as u32;
    let mut h = (bottom - top).max(min_y) as u32;
    if keep_aspect && start_size.y > 0 {
        let ratio = start_size.x as f32 / start_size.y as f32;
        h = ((w as f32 / ratio).round() as u32).max(min_y as u32);
    }
    let pos = clamp_position_for_def(
        IVec2::new(left, top),
        UVec2::new(w, h),
        level_size,
        allow_oob,
    );
    (pos, UVec2::new(w, h))
}

/// Returns the active Entities layer, falling back to the first Entities layer
/// in the level so tools also work from headless tests and direct calls.
pub fn active_entities_layer(cx: &EditorCtx, cv: &CanvasCtx, level: &Level) -> Option<LayerDefId> {
    if let Some(id) = cv.active_layer_id {
        if cx
            .project()
            .defs
            .layer(id)
            .is_some_and(|def| def.kind == LayerKind::Entities)
        {
            return Some(id);
        }
    }
    level
        .layers
        .iter()
        .find(|layer| matches!(layer.data, LayerData::Entities(_)))
        .map(|layer| layer.def_id)
}

/// Entity definition used for placement.
///
/// `Place #<uid>` (emitted by the Entities panel) wins, then the definition of
/// the selected instance, then the project's first entity definition.
pub fn placement_def_id(cx: &EditorCtx, tool_name: &str) -> Option<EntityDefId> {
    if let Some(raw) = tool_name.strip_prefix("Place #") {
        if let Ok(uid) = raw.trim().parse::<u64>() {
            let id = EntityDefId::new(uid);
            if cx.project().defs.entity(id).is_some() {
                return Some(id);
            }
        }
    }
    if let Some(level) = cx.active_level().and_then(|id| cx.project().level(id)) {
        for entity_id in cx.selection().entities().unwrap_or_default() {
            if let Some(entity) = level.entity(*entity_id) {
                return Some(entity.def_id);
            }
        }
    }
    cx.project()
        .defs
        .entities
        .first()
        .map(|def: &EntityDef| def.id)
}
