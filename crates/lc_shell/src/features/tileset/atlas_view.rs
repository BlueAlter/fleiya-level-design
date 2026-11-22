//! Interactive atlas view widget for the Tilesets workspace.

use egui::{vec2, PointerButton, RichText, Sense, TextureHandle, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_feature_tileset::compute_tileset_grid;

use crate::features::tileset::state::TilesetViewState;
use crate::shared::ui::atlas_canvas::{render_atlas_canvas, AtlasEvent};
use crate::shared::ui::tile_bounds::{TileGrid, TileRect};

const PAGE_BG_BYTES: &[u8] =
    include_bytes!("../../../../../assets/Bg/tileset_picker_bg_netral.png");

/// Renders the atlas view with pan, zoom, hover tooltip, and tile selection via `render_atlas_canvas`.
pub fn render_atlas_view(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    texture: Option<&TextureHandle>,
    state: &mut TilesetViewState,
) {
    let Some(tex) = texture else {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, t.bg);
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                if tileset.path.is_empty() {
                    ui.label(
                        RichText::new("No image loaded")
                            .font(egui::FontId::proportional(13.0))
                            .color(t.muted),
                    );
                } else {
                    ui.label(
                        RichText::new(format!(
                            "Warning: Source image file could not be loaded: {}",
                            tileset.path
                        ))
                        .font(egui::FontId::proportional(13.0))
                        .color(t.muted),
                    );
                }
            });
        });
        return;
    };

    let sz = tex.size();
    let (img_w, img_h) = (sz[0] as u32, sz[1] as u32);
    if img_w == 0 || img_h == 0 {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, t.bg);
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Warning: Image has zero dimensions")
                        .font(egui::FontId::proportional(13.0))
                        .color(t.muted),
                );
            });
        });
        return;
    }

    let metrics = compute_tileset_grid(
        [img_w, img_h],
        tileset.tile_size,
        tileset.spacing,
        tileset.padding,
    );
    let grid = TileGrid::new(
        tileset.tile_size.x,
        tileset.tile_size.y,
        tileset.spacing,
        tileset.padding,
        metrics.cols,
        metrics.rows,
    );

    let page_bg_id = egui::Id::new("tileset_atlas_page_bg_texture");
    let page_bg = ui.ctx().data_mut(|d| {
        d.get_temp::<TextureHandle>(page_bg_id).or_else(|| {
            image::load_from_memory(PAGE_BG_BYTES).ok().map(|img| {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    &rgba.into_raw(),
                );
                let handle = ui.ctx().load_texture(
                    "tileset_atlas_page_bg",
                    color_image,
                    egui::TextureOptions::NEAREST_REPEAT,
                );
                d.insert_temp(page_bg_id, handle.clone());
                handle
            })
        })
    });

    let (response, events) = {
        let canvas_state = state.canvas_state_mut(tileset.id);
        render_atlas_canvas(
            ui,
            t,
            canvas_state,
            tex,
            page_bg.as_ref(),
            &grid,
            vec2(img_w as f32, img_h as f32),
            None,
            None,
        )
    };

    // Overlay "Fit" button in top-right of canvas viewport
    let fit_btn_w = 40.0;
    let fit_btn_h = 24.0;
    let fit_rect = egui::Rect::from_min_size(
        egui::pos2(
            response.rect.max.x - fit_btn_w - 8.0,
            response.rect.min.y + 8.0,
        ),
        vec2(fit_btn_w, fit_btn_h),
    );
    let fit_btn = egui::Button::new(RichText::new("Fit").size(11.0).color(t.ink_strong))
        .fill(t.panel)
        .stroke(t.hairline())
        .corner_radius(egui::CornerRadius::same(3));

    if ui
        .put(fit_rect, fit_btn)
        .on_hover_text("Fit atlas to view (F / Home)")
        .clicked()
    {
        state.canvas_state_mut(tileset.id).fit_to_viewport(
            response.rect,
            img_w as f32,
            img_h as f32,
        );
    }

    // Hover tooltip: "Tile N (col, row)"
    if let Some(hovered_id) = state.canvas_state_mut(tileset.id).hovered_tile {
        if grid.cols > 0 {
            let col = hovered_id % grid.cols;
            let row = hovered_id / grid.cols;
            response.on_hover_text(format!("Tile {hovered_id} ({col}, {row})"));
        }
    }

    // Process events (clicks never mutate project data)
    for event in events {
        match event {
            AtlasEvent::Clicked {
                cell,
                tile_id,
                button,
                ..
            } => {
                if button == PointerButton::Primary {
                    state.selected_tile = Some(tile_id);
                    state.canvas_state_mut(tileset.id).selected_tile_rect =
                        Some(TileRect::single(cell.0, cell.1));
                }
            }
            AtlasEvent::DragRectEnd(rect) => {
                state.selected_tile = Some(rect.top_left_id(&grid));
            }
            _ => {}
        }
    }

    // Synchronize pan and zoom settings in view state
    let (zoom, offset) = {
        let cs = state.canvas_state_mut(tileset.id);
        (cs.view.zoom, cs.view.offset)
    };
    let pz = state.pan_zoom_mut(tileset.id);
    pz.zoom = zoom;
    pz.pan = offset;
}
