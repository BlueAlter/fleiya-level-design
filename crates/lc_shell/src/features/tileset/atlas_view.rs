//! Interactive atlas view widget for the Tilesets workspace.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, PointerButton, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::compute_tileset_grid;

use crate::features::tileset::state::TilesetViewState;
use crate::shared::ui::tile_bounds::TileGrid;

/// Renders the atlas view with pan, zoom, hover tooltip, and tile selection.
pub fn render_atlas_view(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    texture: Option<&egui::TextureHandle>,
    state: &mut TilesetViewState,
) {
    let (viewport_rect, response) =
        ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let painter = ui.painter().with_clip_rect(viewport_rect);

    // Dark backdrop for viewport
    painter.rect_filled(viewport_rect, CornerRadius::ZERO, t.bg);

    let (img_w, img_h) = if let Some(tex) = texture {
        let sz = tex.size();
        (sz[0] as u32, sz[1] as u32)
    } else {
        (0, 0)
    };

    if img_w == 0 || img_h == 0 {
        // Missing or zero-size image: show centered message
        ui.scope_builder(egui::UiBuilder::new().max_rect(viewport_rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("No image loaded")
                        .font(egui::FontId::proportional(13.0))
                        .color(t.muted),
                );
            });
        });
        return;
    }

    let pan_zoom = state.pan_zoom_mut(tileset.id);

    // Initial center if offset is zero
    let init_id = Id::new(("tileset_atlas_init_done", tileset.id));
    let initialized = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(init_id))
        .unwrap_or(false);
    if !initialized {
        let avail_w = (viewport_rect.width() - 48.0).max(10.0);
        let avail_h = (viewport_rect.height() - 48.0).max(10.0);
        let fit_zoom = (avail_w / img_w as f32)
            .min(avail_h / img_h as f32)
            .clamp(0.25, 4.0);
        pan_zoom.zoom = fit_zoom;
        let centered_x =
            viewport_rect.min.x + (viewport_rect.width() - img_w as f32 * fit_zoom) * 0.5;
        let centered_y =
            viewport_rect.min.y + (viewport_rect.height() - img_h as f32 * fit_zoom) * 0.5;
        pan_zoom.pan = vec2(centered_x, centered_y);
        ui.ctx().data_mut(|d| d.insert_temp(init_id, true));
    }

    // Input: Pan (Middle drag or Space + Left drag)
    let is_space = ui.input(|i| i.key_down(egui::Key::Space));
    let is_panning = response.dragged_by(PointerButton::Middle)
        || (is_space && response.dragged_by(PointerButton::Primary));

    if is_panning {
        pan_zoom.pan += response.drag_delta();
        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    } else if is_space && response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Grab);
    }

    // Input: Zoom (Ctrl + scroll towards pointer)
    let ctrl_down = ui.input(|i| i.modifiers.ctrl || i.modifiers.command);
    if ctrl_down && response.hovered() {
        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_y.abs() > 0.0 {
            if let Some(cursor) = response.hover_pos() {
                let zoom_factor = (scroll_y * 0.005).exp();
                let old_zoom = pan_zoom.zoom;
                let new_zoom = (old_zoom * zoom_factor).clamp(0.25, 32.0);
                if (new_zoom - old_zoom).abs() > 0.0001 {
                    // Zoom centered around cursor
                    let rel_x = cursor.x - pan_zoom.pan.x;
                    let rel_y = cursor.y - pan_zoom.pan.y;
                    pan_zoom.pan.x = (cursor.x - rel_x * (new_zoom / old_zoom)).round();
                    pan_zoom.pan.y = (cursor.y - rel_y * (new_zoom / old_zoom)).round();
                    pan_zoom.zoom = new_zoom;
                }
            }
        }
    }

    let zoom = pan_zoom.zoom;
    let offset = pan_zoom.pan;

    // Helper closures for coordinate mapping
    let page_to_screen = |p: Pos2| -> Pos2 {
        pos2(
            (p.x * zoom + offset.x).round(),
            (p.y * zoom + offset.y).round(),
        )
    };
    let page_to_screen_rect =
        |r: Rect| -> Rect { Rect::from_min_max(page_to_screen(r.min), page_to_screen(r.max)) };
    let screen_to_page =
        |s: Pos2| -> Pos2 { pos2((s.x - offset.x) / zoom, (s.y - offset.y) / zoom) };

    let page_rect = Rect::from_min_size(Pos2::ZERO, vec2(img_w as f32, img_h as f32));
    let screen_page_rect = page_to_screen_rect(page_rect);

    // Draw background & checkerboard
    let page_clip = screen_page_rect.intersect(viewport_rect);
    if page_clip.is_positive() {
        let page_painter = painter.with_clip_rect(page_clip);
        draw_checkerboard(
            &page_painter,
            screen_page_rect,
            12.0 * zoom.clamp(0.5, 2.0),
            t,
        );

        // Draw atlas texture (un-distorted aspect ratio)
        if let Some(tex) = texture {
            page_painter.image(
                tex.id(),
                screen_page_rect,
                Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }

    // Page border
    painter.rect_stroke(
        screen_page_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, t.line_strong),
        StrokeKind::Outside,
    );

    // Compute grid metrics
    let metrics = compute_tileset_grid(
        [img_w, img_h],
        tileset.tile_size,
        tileset.spacing,
        tileset.padding,
    );
    let tile_grid = TileGrid::new(
        tileset.tile_size.x,
        tileset.tile_size.y,
        tileset.spacing,
        tileset.padding,
        metrics.cols,
        metrics.rows,
    );

    // Hover & Click detection
    let mut hovered_tile: Option<(u32, u32, u32)> = None; // (col, row, id)
    if response.hovered() && !is_panning && !is_space {
        if let Some(pointer_pos) = response.hover_pos() {
            if screen_page_rect.contains(pointer_pos) {
                let page_pos = screen_to_page(pointer_pos);
                if let Some((col, row)) = tile_grid.cell_at(page_pos) {
                    let tile_id = tile_grid.tile_id(col, row);
                    hovered_tile = Some((col, row, tile_id));
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);

                    if response.clicked() {
                        state.selected_tile = Some(tile_id);
                    }
                }
            }
        }
    }

    // Draw grid lines
    if metrics.cols > 0 && metrics.rows > 0 {
        let grid_clip = screen_page_rect.intersect(viewport_rect);
        if grid_clip.is_positive() {
            let grid_painter = painter.with_clip_rect(grid_clip);
            let grid_stroke = Stroke::new(1.0, Color32::from_white_alpha(35));

            for r in 0..metrics.rows {
                for c in 0..metrics.cols {
                    let cell_rect = tile_grid.cell_rect(c, r);
                    let screen_cell = page_to_screen_rect(cell_rect);
                    grid_painter.rect_stroke(
                        screen_cell,
                        CornerRadius::ZERO,
                        grid_stroke,
                        StrokeKind::Inside,
                    );
                }
            }

            // Draw hover highlight
            if let Some((col, row, _)) = hovered_tile {
                let cell_rect = tile_grid.cell_rect(col, row);
                let screen_cell = page_to_screen_rect(cell_rect);
                grid_painter.rect_filled(
                    screen_cell,
                    CornerRadius::ZERO,
                    Color32::from_rgba_unmultiplied(61, 214, 184, 40),
                );
            }

            // Draw selection highlight (2 px zoom-invariant outline)
            if let Some(selected_id) = state.selected_tile {
                let sel_row = selected_id / metrics.cols.max(1);
                let sel_col = selected_id % metrics.cols.max(1);
                if sel_row < metrics.rows && sel_col < metrics.cols {
                    let cell_rect = tile_grid.cell_rect(sel_col, sel_row);
                    let screen_cell = page_to_screen_rect(cell_rect);
                    grid_painter.rect_stroke(
                        screen_cell,
                        CornerRadius::ZERO,
                        Stroke::new(2.0, t.accent),
                        StrokeKind::Inside,
                    );
                }
            }
        }
    }

    // Hover tooltip: "Tile N (col, row)"
    if let Some((col, row, id)) = hovered_tile {
        response.on_hover_text(format!("Tile {id} ({col}, {row})"));
    }
}
