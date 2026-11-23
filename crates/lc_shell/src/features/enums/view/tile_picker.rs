//! Tile picker modal integration for enum values.

use egui::Context;
use lc_core::layer::TileRef;

use super::types::EnumViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::tile_bounds::TileGrid;
use crate::shared::ui::tile_picker::{TilePickerEvent, TilesetData};

/// Renders the tile picker modal popup and updates tile bindings if a tile is chosen.
pub fn render_tile_picker_modal(app: &mut ShellApp, ctx: &Context, state: &mut EnumViewState) {
    let target_enum_id = state
        .tile_picker
        .target
        .as_ref()
        .map(|t| t.enum_id)
        .or(app.selected_enum_def);

    let active_ts_id = target_enum_id.and_then(|eid| {
        app.enum_draft
            .iter()
            .find(|e| e.id == eid)
            .and_then(|e| e.tileset_id)
    });

    let active_tileset_data = active_ts_id
        .and_then(|id| app.project.defs.tileset(id))
        .map(TilesetData::from)
        .unwrap_or_else(TilesetData::empty);

    let current_bound_tile_rect = if let Some(target) = state.tile_picker.target.as_ref() {
        app.enum_draft
            .iter()
            .find(|e| e.id == target.enum_id)
            .and_then(|ed| ed.values.iter().find(|v| v.id == target.value_id))
            .and_then(|v| {
                let img_w = app
                    .resolve_tileset_image_size(active_tileset_data.id)
                    .map(|(w, _)| w)
                    .unwrap_or(256);
                let tile_w = active_tileset_data.tile_size.x.max(1);
                let padding = active_tileset_data.padding;
                let spacing = active_tileset_data.spacing;
                let eff_w = img_w.saturating_sub(padding * 2);
                let stride_x = tile_w + spacing;
                let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                app.get_enum_value_tile_rect(target.enum_id, &v.id, v.tile.as_ref(), cols)
            })
    } else {
        None
    };

    match state
        .tile_picker
        .show(ctx, &active_tileset_data, current_bound_tile_rect)
    {
        TilePickerEvent::Selected(picked_rect) => {
            if let Some(target) = state.tile_picker.target.take() {
                let target_enum_id = target.enum_id;
                let target_val_id = target.value_id;

                let target_ts_id = app
                    .enum_draft
                    .iter()
                    .find(|e| e.id == target_enum_id)
                    .and_then(|e| e.tileset_id)
                    .or(active_ts_id);

                if let Some(ts_id) = target_ts_id {
                    let img_w = app
                        .resolve_tileset_image_size(ts_id)
                        .map(|(w, _)| w)
                        .unwrap_or(256);
                    let tile_w = active_tileset_data.tile_size.x.max(1);
                    let tile_h = active_tileset_data.tile_size.y.max(1);
                    let padding = active_tileset_data.padding;
                    let spacing = active_tileset_data.spacing;
                    let eff_w = img_w.saturating_sub(padding * 2);
                    let stride_x = tile_w + spacing;
                    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);
                    let grid = TileGrid::new(tile_w, tile_h, spacing, padding, cols, 1000);
                    let top_left_id = picked_rect.top_left_id(&grid);

                    if let Some(ed) = app.enum_draft.iter_mut().find(|e| e.id == target_enum_id) {
                        if let Some(v) = ed.values.iter_mut().find(|v| v.id == target_val_id) {
                            v.tile = Some(TileRef::new(ts_id, top_left_id));
                        }
                    }
                    if let Some(draft) = app.enum_drafts.get_mut(&target_enum_id) {
                        if let Some(v) = draft.def.values.iter_mut().find(|v| v.id == target_val_id)
                        {
                            v.tile = Some(TileRef::new(ts_id, top_left_id));
                        }
                        draft.tile_rects.insert(target_val_id.clone(), picked_rect);
                        draft.revision += 1;
                    }
                    app.enum_value_tile_rects
                        .insert((target_enum_id, target_val_id), picked_rect);
                    app.enum_draft_revision += 1;
                }
                ctx.request_repaint();
            }
        }
        TilePickerEvent::Closed => {
            state.tile_picker.target = None;
            state.tile_picker.target_val_id = None;
        }
        TilePickerEvent::None => {}
    }
}
