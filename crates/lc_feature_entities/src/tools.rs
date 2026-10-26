//! Interactive tools for Entities layers: place, select, move, resize, delete
//! and duplicate.
//!
//! A single composite tool dispatches on `cx.active_tool_name()`:
//! `Paint`/`Place #<id>` places, `Erase` deletes under the pointer, and any
//! other name (including `Select`) selects and edits. Delete and Ctrl+D work
//! on the current selection while the canvas is hovered.

use egui::{Key, PointerButton};
use glam::{IVec2, UVec2};
use lc_core::defs::EntityDef;
use lc_core::id::{EntityDefId, EntityId, LayerDefId, LevelId};
use lc_core::layer::{EntityInstance, LayerData, LayerKind};
use lc_core::level::Level;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::selection::Selection;

use crate::field_defs::default_field_values;
use crate::geometry::{entity_layer_of, snap_size};
use crate::helpers::next_entity_id;
use crate::instance_commands::{
    AddEntityInstancesCommand, RemoveEntityInstancesCommand, SetEntityPositionsCommand,
    SetEntitySizesCommand,
};
use crate::stroke::next_stroke_id;

pub use crate::geometry::{
    clamp_position, grid_size, local_pos, pick_entity_at, selected_in_level, snap_to_grid,
};

#[derive(Debug)]
enum DragKind {
    Move {
        starts: Vec<(EntityId, IVec2)>,
        grid: IVec2,
    },
    Resize {
        id: EntityId,
        start_size: UVec2,
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
        let position = clamp_position(snap_to_grid(local, grid), def.size_px, level.size_px);
        let entity = EntityInstance {
            id: next_entity_id(cx.project()),
            def_id,
            pos_px: position,
            size_px: def.size_px,
            fields: default_field_values(&def, &cx.project().defs.enums),
        };
        let new_id = entity.id;
        let def_name = def.name.clone();
        cx.exec(AddEntityInstancesCommand::new(
            level_id,
            layer_def_id,
            vec![entity],
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
        let hit = pick_entity_at(level, local);

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
        let Some(id) = pick_entity_at(level, local) else {
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
        let Some(hit) = pick_entity_at(&level, local) else {
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

        // Resize when the pointer grabs the bottom-right corner of a single
        // selected entity.
        if selected.len() == 1 {
            if let Some(entity) = level.entity(selected[0]) {
                let corner = entity.pos_px + entity.size_px.as_ivec2();
                let threshold = (8.0 / cv.viewport.zoom).max(2.0) as i32;
                if (local - corner).abs().max_element() <= threshold {
                    self.drag = Some(DragState {
                        kind: DragKind::Resize {
                            id: entity.id,
                            start_size: entity.size_px,
                            grid,
                        },
                        origin: local,
                        stroke_id: next_stroke_id(),
                    });
                    return;
                }
            }
        }

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
                        let size = level
                            .entity(*id)
                            .map(|e| e.size_px)
                            .unwrap_or_else(|| UVec2::splat(1));
                        (
                            *id,
                            clamp_position(
                                snap_to_grid(*start + delta, *grid),
                                size,
                                level.size_px,
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
                start_size,
                grid,
            } => {
                let min = UVec2::new(grid.x.max(1) as u32, grid.y.max(1) as u32);
                let target = start_size.as_ivec2() + delta;
                let snapped = snap_size(target, *grid);
                let width = (snapped.x.max(1) as u32).max(min.x.min(level.size_px.x.max(1)));
                let height = (snapped.y.max(1) as u32).max(min.y.min(level.size_px.y.max(1)));
                cx.exec(SetEntitySizesCommand::with_stroke(
                    level_id,
                    vec![(*id, UVec2::new(width, height))],
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
