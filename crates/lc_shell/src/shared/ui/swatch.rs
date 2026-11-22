//! Standardized color swatch and tile preview icon button widgets.
//!
//! Provides [`color_swatch`] with an integrated color picker popup,
//! and [`tile_icon_button`] with a checkerboard empty state.
//!
//! # Usage Example
//! ```ignore
//! use egui::Color32;
//! use lc_shell::shared::ui::swatch::{color_swatch, tile_icon_button};
//!
//! let mut color = Color32::RED;
//! color_swatch(ui, "my_color", &mut color);
//! let tile_resp = tile_icon_button(ui, None);
//! ```

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, Rect, Response, Sense, Stroke, StrokeKind, TextureId,
    Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;

use super::workspace_metrics::ICON_BUTTON_SIZE_FORM;

/// Visual representation of a bound tile for [`tile_icon_button`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileIconData {
    /// Texture handle allocated in egui.
    pub texture_id: TextureId,
    /// UV coordinates within the texture normalized in 0.0..=1.0.
    pub uv: Rect,
}

impl TileIconData {
    /// Constructs a new [`TileIconData`].
    pub fn new(texture_id: TextureId, uv: Rect) -> Self {
        Self { texture_id, uv }
    }
}

/// Renders a square 1:1 color swatch button that toggles an anchored color picker popup.
///
/// Bypasses `button_padding` using `allocate_exact_size` matching [`ICON_BUTTON_SIZE_FORM`].
pub fn color_swatch(ui: &mut Ui, id: impl egui::AsId, color: &mut Color32) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(
        vec2(ICON_BUTTON_SIZE_FORM, ICON_BUTTON_SIZE_FORM),
        Sense::click(),
    );
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, CornerRadius::same(3), *color);

        let border_stroke = if resp.hovered() {
            Stroke::new(1.5, t.ink_strong)
        } else {
            Stroke::new(1.0, Color32::from_black_alpha(80))
        };
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            border_stroke,
            StrokeKind::Inside,
        );
    }

    // Anchored color picker popup
    egui::Popup::from_toggle_button_response(&resp)
        .id(egui::Id::new(id).with("color_picker_popup"))
        .show(|ui| {
            ui.spacing_mut().slider_width = 180.0;
            let mut c = *color;
            if egui::color_picker::color_picker_color32(
                ui,
                &mut c,
                egui::color_picker::Alpha::Opaque,
            ) {
                *color = c;
            }
        });

    resp
}

/// Renders a square 1:1 tile preview button (checkered pattern when empty).
///
/// Takes an optional [`TileIconData`] containing texture ID and UV coordinates.
pub fn tile_icon_button(ui: &mut Ui, tile: Option<TileIconData>) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(
        vec2(ICON_BUTTON_SIZE_FORM, ICON_BUTTON_SIZE_FORM),
        Sense::click(),
    );
    let resp = resp.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(rect);
        painter.rect_filled(rect, CornerRadius::same(3), t.sunken);

        let draw_rect = rect.shrink(1.0);
        draw_checkerboard(&painter, draw_rect, 4.0, &t);

        if let Some(tile_data) = tile {
            painter.image(
                tile_data.texture_id,
                draw_rect,
                tile_data.uv,
                Color32::WHITE,
            );
        }

        let border_color = if resp.hovered() { t.ink_strong } else { t.line };
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            Stroke::new(1.0, border_color),
            StrokeKind::Inside,
        );
    }

    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_tile_icon_data_constructor() {
        let data = TileIconData::new(
            TextureId::default(),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        );
        assert_eq!(data.texture_id, TextureId::default());
        assert_eq!(data.uv.min, Pos2::ZERO);
    }

    #[test]
    fn test_color_swatch_and_tile_icon_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let mut color = Color32::from_rgb(100, 150, 200);

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let swatch_resp = color_swatch(ui, "test_swatch", &mut color);
            assert_eq!(swatch_resp.rect.width(), ICON_BUTTON_SIZE_FORM);
            assert_eq!(swatch_resp.rect.height(), ICON_BUTTON_SIZE_FORM);

            let tile_resp = tile_icon_button(ui, None);
            assert_eq!(tile_resp.rect.width(), ICON_BUTTON_SIZE_FORM);
            assert_eq!(tile_resp.rect.height(), ICON_BUTTON_SIZE_FORM);
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "swatch and tile_icon_button should produce render shapes"
        );
    }
}
