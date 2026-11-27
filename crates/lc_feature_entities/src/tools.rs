//! Interactive tools for Entities layers: place, select, move, resize, delete
//! and duplicate.
//!
//! A single composite tool dispatches on `cx.active_tool_name()`:
//! `Paint`/`Place #<id>` places, `Erase` deletes under the pointer, and any
//! other name (including `Select`) selects and edits. Delete and Ctrl+D work
//! on the current selection while the canvas is hovered.

use egui::{Key, PointerButton};
use glam::{IVec2, UVec2};
use lc_core::id::{EntityId, LayerDefId, LevelId};
use lc_core::layer::EntityInstance;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::selection::Selection;

use crate::field_defs::default_field_values;
use crate::geometry::{
    clamp_position_for_def, entity_layer_of, hit_resize_handle, pick_entity_at_with_defs,
    ResizeHandle,
};
use crate::helpers::next_entity_id;
use crate::instance_commands::{
    AddEntityInstancesCommand, RemoveEntityInstancesCommand, SetEntityPositionsCommand,
    SetEntitySizesCommand,
};
use crate::limits::{can_place_with_limits, PlaceEntityWithLimitsCommand};
use crate::stroke::next_stroke_id;
use crate::tool_helpers::{compute_resized_rect, handle_pick_click};

pub use crate::geometry::{
    clamp_position, grid_size, local_pos, pick_entity_at, selected_in_level, snap_to_grid,
};
pub use crate::tool_helpers::{active_entities_layer, placement_def_id};

#[derive(Debug)]
enum DragKind {
    Move {
        starts: Vec<(EntityId, IVec2)>,
        grid: IVec2,
    },
    Resize {
        id: EntityId,
        handle: ResizeHandle,
        start_pos: IVec2,
        start_size: UVec2,
        keep_aspect: bool,
        allow_out_of_bounds: bool,
        grid: IVec2,
    },
}

#[derive(Debug)]
struct DragState {
    kind: DragKind,
    origin: IVec2,
    stroke_id: u64,
}

/// Composite canvas tool for entity layers.
#[derive(Debug, Default)]
pub struct EntityCanvasTool {
    drag: Option<DragState>,
}

impl EntityCanvasTool {
    /// Creates a new entity tool.
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_shortcuts(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        // Never steal Delete/Backspace/Ctrl+D while a widget owns the keyboard.
        // Canvas input only requires the pointer to hover, so an inspector text
        // field can still be focused while these keys reach the canvas.
        if cv.response.ctx.egui_wants_keyboard_input() {
            return;
        }
        let (delete, duplicate) = cv.response.ctx.input(|i| {
            (
                i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace),
                i.modifiers.command && i.key_pressed(Key::D),
            )
        });
        if delete {
            self.delete_selected(cx, cv.active_level_id);
        }
        if duplicate {
            self.duplicate_selected(cv, cx);
        }
    }

    fn delete_selected(&mut self, cx: &mut EditorCtx, level_id: LevelId) {
        let Some(level) = cx.project().level(level_id) else {
            return;
        };
        let ids = selected_in_level(cx.selection(), level);
        if ids.is_empty() {
            return;
        }
        cx.exec(RemoveEntityInstancesCommand::new(level_id, ids));
        cx.emit(EditorEvent::SelectionChanged(Selection::None));
        cx.status("Deleted selected entities");
    }

    fn duplicate_selected(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id).cloned() else {
            return;
        };
        let source_ids = selected_in_level(cx.selection(), &level);
        if source_ids.is_empty() {
            return;
        }
        let mut next_id = next_entity_id(cx.project()).0;
        let mut copies_by_layer: Vec<(LayerDefId, Vec<EntityInstance>)> = Vec::new();
        let mut new_ids = Vec::new();

        for source_id in source_ids {
            let Some(layer_def_id) = entity_layer_of(&level, source_id) else {
                continue;
            };
            let Some(source) = level.entity(source_id) else {
                continue;
            };
            let grid = grid_size(cx.project(), &level, layer_def_id);
            let offset = IVec2::new(grid.x.max(1), grid.y.max(1));
            let mut copy = source.clone();
            copy.id = EntityId::new(next_id);
            next_id = next_id.saturating_add(1);
            copy.pos_px = clamp_position(source.pos_px + offset, copy.size_px, level.size_px);
            new_ids.push(copy.id);
            match copies_by_layer
                .iter_mut()
                .find(|(existing, _)| *existing == layer_def_id)
            {
                Some((_, copies)) => copies.push(copy),
                None => copies_by_layer.push((layer_def_id, vec![copy])),
            }
        }

        if new_ids.is_empty() {
            return;
        }
        // Copies stay on their source layer instead of being regrouped onto
        // whichever Entities layer happens to be active.
        for (layer_def_id, copies) in copies_by_layer {
            cx.exec(AddEntityInstancesCommand::new(
                level_id,
                layer_def_id,
                copies,
            ));
        }
        cx.emit(EditorEvent::SelectionChanged(Selection::Entities(
            new_ids.clone(),
        )));
        cx.status(format!("Duplicated {} entit(ies)", new_ids.len()));
    }

    fn place_at(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id) else {
            return;
        };
        let Some(layer_def_id) = active_entities_layer(cx, cv, level) else {
            cx.status("Select an Entities layer to place entities");
            return;
        };
        let Some(local) = local_pos(cv, level) else {
            return;
        };
        let Some(def_id) = placement_def_id(cx, cx.active_tool_name()) else {
            cx.status("No entity definition available to place");
            return;
        };
        let Some(def) = cx.project().defs.entity(def_id).cloned() else {
            return;
        };

        let grid = grid_size(cx.project(), level, layer_def_id);
        let world_id = cx.project().level_and_world(level_id).map(|(w, _)| w.id);
        if let Some(wid) = world_id {
            if !can_place_with_limits(cx.project(), def_id, wid, level_id, layer_def_id) {
                cx.status(format!(
                    "Cannot place '{}': max count ({}) reached",
                    def.name, def.max_count
                ));
                return;
            }
        }
        let position = clamp_position_for_def(
            snap_to_grid(local, grid),
            def.size_px,
            level.size_px,
            def.allow_out_of_bounds,
        );
        let entity = EntityInstance {
            id: next_entity_id(cx.project()),
            def_id,
            pos_px: position,
            size_px: def.size_px,
            fields: default_field_values(&def, &cx.project().defs.enums),
        };
        let new_id = entity.id;
        let def_name = def.name.clone();
        cx.exec(PlaceEntityWithLimitsCommand::new(
            level_id,
            layer_def_id,
            entity,
        ));
        cx.emit(EditorEvent::SelectionChanged(Selection::Entities(vec![
            new_id,
        ])));
        cx.status(format!("Placed entity '{def_name}'"));
    }

    fn select_at(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx, additive: bool) {
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id) else {
            return;
        };
        let Some(local) = local_pos(cv, level) else {
            return;
        };
        let hit = pick_entity_at_with_defs(cx.project(), level, local, cv.viewport.zoom);

        let mut ids: Vec<EntityId> = if additive {
            // Never seed a cross-level selection: shift-click only extends
            // entities that belong to the level being edited.
            selected_in_level(cx.selection(), level)
        } else {
            Vec::new()
        };

        match hit {
            Some(id) => {
                if additive {
                    if let Some(pos) = ids.iter().position(|existing| *existing == id) {
                        ids.remove(pos);
                    } else {
                        ids.push(id);
                    }
                } else {
                    ids = vec![id];
                }
            }
            None if !additive => ids.clear(),
            None => {}
        }

        let selection = if ids.is_empty() {
            Selection::None
        } else {
            Selection::Entities(ids)
        };
        if &selection != cx.selection() {
            cx.emit(EditorEvent::SelectionChanged(selection));
        }
    }

    fn erase_at(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id) else {
            return;
        };
        let Some(local) = local_pos(cv, level) else {
            return;
        };
        let Some(id) = pick_entity_at_with_defs(cx.project(), level, local, cv.viewport.zoom)
        else {
            return;
        };
        cx.exec(RemoveEntityInstancesCommand::new(level_id, vec![id]));
        cx.emit(EditorEvent::SelectionChanged(Selection::None));
        cx.status("Deleted entity");
    }

    fn begin_drag(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id).cloned() else {
            return;
        };
        let Some(local) = local_pos(cv, &level) else {
            return;
        };

        // Check resize handles on currently selected single entity first
        let cur_selected = selected_in_level(cx.selection(), &level);
        if cur_selected.len() == 1 {
            if let Some(entity) = level.entity(cur_selected[0]) {
                if let Some(def) = cx.project().defs.entity(entity.def_id) {
                    if let Some(handle) = hit_resize_handle(
                        entity.pos_px,
                        entity.size_px,
                        local,
                        cv.viewport.zoom,
                        def.resizable_x,
                        def.resizable_y,
                    ) {
                        let grid = entity_layer_of(&level, entity.id)
                            .map(|id| grid_size(cx.project(), &level, id))
                            .unwrap_or_else(|| IVec2::splat(16));
                        self.drag = Some(DragState {
                            kind: DragKind::Resize {
                                id: entity.id,
                                handle,
                                start_pos: entity.pos_px,
                                start_size: entity.size_px,
                                keep_aspect: def.keep_aspect_ratio,
                                allow_out_of_bounds: def.allow_out_of_bounds,
                                grid,
                            },
                            origin: local,
                            stroke_id: next_stroke_id(),
                        });
                        return;
                    }
                }
            }
        }

        let Some(hit) = pick_entity_at_with_defs(cx.project(), &level, local, cv.viewport.zoom)
        else {
            return;
        };

        let current: Vec<EntityId> = cx
            .selection()
            .entities()
            .map(<[EntityId]>::to_vec)
            .unwrap_or_default();
        let was_selected = current.contains(&hit);
        let selected: Vec<EntityId> = if was_selected {
            current
                .into_iter()
                .filter(|id| level.entity(*id).is_some())
                .collect()
        } else {
            vec![hit]
        };
        if !was_selected {
            cx.emit(EditorEvent::SelectionChanged(Selection::Entities(vec![
                hit,
            ])));
        }

        let layer_def_id = entity_layer_of(&level, hit);
        let grid = layer_def_id
            .map(|id| grid_size(cx.project(), &level, id))
            .unwrap_or_else(|| IVec2::splat(16));

        let starts: Vec<(EntityId, IVec2)> = selected
            .into_iter()
            .filter_map(|id| level.entity(id).map(|e| (id, e.pos_px)))
            .collect();
        if starts.is_empty() {
            return;
        }
        self.drag = Some(DragState {
            kind: DragKind::Move { starts, grid },
            origin: local,
            stroke_id: next_stroke_id(),
        });
    }

    fn update_drag(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        let Some(drag) = &self.drag else {
            return;
        };
        let level_id = cv.active_level_id;
        let Some(level) = cx.project().level(level_id) else {
            return;
        };
        let Some(local) = local_pos(cv, level) else {
            return;
        };
        let delta = local - drag.origin;

        match &drag.kind {
            DragKind::Move { starts, grid } => {
                let positions: Vec<(EntityId, IVec2)> = starts
                    .iter()
                    .map(|(id, start)| {
                        let (size, allow_oob) = level
                            .entity(*id)
                            .map(|e| {
                                let oob = cx
                                    .project()
                                    .defs
                                    .entity(e.def_id)
                                    .is_some_and(|d| d.allow_out_of_bounds);
                                (e.size_px, oob)
                            })
                            .unwrap_or((UVec2::splat(1), false));
                        (
                            *id,
                            clamp_position_for_def(
                                snap_to_grid(*start + delta, *grid),
                                size,
                                level.size_px,
                                allow_oob,
                            ),
                        )
                    })
                    .collect();
                cx.exec(SetEntityPositionsCommand::with_stroke(
                    level_id,
                    positions,
                    drag.stroke_id,
                ));
            }
            DragKind::Resize {
                id,
                handle,
                start_pos,
                start_size,
                keep_aspect,
                allow_out_of_bounds,
                grid,
            } => {
                let (new_pos, new_size) = compute_resized_rect(
                    *start_pos,
                    *start_size,
                    *handle,
                    delta,
                    *grid,
                    *keep_aspect && !cv.modifiers.shift,
                    *allow_out_of_bounds,
                    level.size_px,
                );
                if new_pos != *start_pos {
                    cx.exec(SetEntityPositionsCommand::with_stroke(
                        level_id,
                        vec![(*id, new_pos)],
                        drag.stroke_id,
                    ));
                }
                cx.exec(SetEntitySizesCommand::with_stroke(
                    level_id,
                    vec![(*id, new_size)],
                    drag.stroke_id,
                ));
            }
        }
    }
}

impl CanvasTool for EntityCanvasTool {
    fn name(&self) -> &str {
        "Entities"
    }

    fn on_input(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) {
        self.handle_shortcuts(cv, cx);

        if let Some(pick) = cx.pick_mode().cloned() {
            if cv.response.ctx.input(|i| i.key_pressed(Key::Escape)) {
                cx.emit(EditorEvent::SetCanvasPickMode(None));
                cx.status("Cancelled pick mode");
                return;
            }
            if cv.response.clicked_by(PointerButton::Primary) && handle_pick_click(cv, cx, &pick) {
                return;
            }
        }

        let tool_name = cx.active_tool_name().to_string();
        let place_mode = tool_name == "Paint" || tool_name.starts_with("Place");
        let erase_mode = tool_name == "Erase";

        if cv.response.drag_started_by(PointerButton::Primary) && !place_mode && !erase_mode {
            self.begin_drag(cv, cx);
        }

        if cv.response.dragged_by(PointerButton::Primary) && self.drag.is_some() {
            self.update_drag(cv, cx);
        }

        if cv.response.drag_stopped_by(PointerButton::Primary) {
            self.drag = None;
        }

        if cv.response.clicked_by(PointerButton::Primary) {
            if place_mode {
                self.place_at(cv, cx);
            } else if erase_mode {
                self.erase_at(cv, cx);
            } else {
                let additive = cv.modifiers.shift;
                self.select_at(cv, cx, additive);
            }
        }
    }
}
