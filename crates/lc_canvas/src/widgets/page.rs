//! Page-level document and dialog layout primitives.
//!
//! These helpers keep document tabs visually consistent without imposing any
//! feature-specific behaviour. A page is a full-bleed surface with the editor's
//! standard padding and a vertical scroll area; headers, metadata rows, empty
//! states and the read-only canvas preview are composed from the same tokens as
//! the rest of the shell.

use egui::{
    Align, CornerRadius, FontFamily, FontId, Frame, Layout, Margin, RichText, Sense, StrokeKind,
    Ui, Vec2,
};
use lc_ui_api::viewport::Viewport;

use super::primitives::{hline, hud, hud_elided, tag};
use crate::grid::{draw_checkerboard, draw_grid};
use crate::theme::{self, Tokens};

/// Fills the current tab with a padded, scrollable page surface.
pub fn page(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    Frame::new()
        .fill(t.bg)
        .inner_margin(Margin::same(theme::PAGE_PAD))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    add_contents(ui);
                });
        });
}

/// A document page header: kind tag, display title and one-line description.
pub fn page_header(ui: &mut Ui, kind: &str, title: &str, description: &str) {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        tag(ui, kind, theme::tint(t.accent, 0.14), t.accent_ink);
        ui.add_space(8.0);
        hud(ui, "DOCUMENT", t.faint);
    });
    ui.add_space(10.0);
    ui.label(
        RichText::new(title)
            .font(FontId::new(26.0, theme::display_for(ui.ctx())))
            .color(t.ink_strong),
    );
    ui.add_space(4.0);
    ui.label(
        RichText::new(description)
            .font(FontId::new(13.0, FontFamily::Proportional))
            .color(t.muted),
    );
    ui.add_space(12.0);
    hline(ui);
}

/// A bordered empty-state block used while a feature editor is not wired yet.
pub fn empty_state(ui: &mut Ui, eyebrow: &str, title: &str, body: &str, hint: &str) {
    let t = Tokens::get(ui.ctx());
    Frame::new()
        .fill(t.canvas)
        .stroke(t.hairline())
        .inner_margin(Margin::same(24))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            hud(ui, eyebrow, t.accent_ink);
            ui.add_space(8.0);
            ui.label(
                RichText::new(title)
                    .font(FontId::new(18.0, theme::semibold_for(ui.ctx())))
                    .color(t.ink_strong),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(body)
                    .font(FontId::new(13.0, FontFamily::Proportional))
                    .color(t.muted),
            );
            ui.add_space(12.0);
            hud(ui, hint, t.faint);
        });
}

/// A HUD-labelled metadata row with a trailing hairline.
pub fn meta_row(ui: &mut Ui, label: &str, value: &str) {
    let t = Tokens::get(ui.ctx());
    // Keep values clear of the scroll area's overlay scrollbar.
    const VALUE_GUTTER: f32 = 16.0;
    ui.horizontal(|ui| {
        hud(ui, label, t.faint);
        let value_width = (ui.available_width() - VALUE_GUTTER).max(0.0);
        ui.allocate_ui_with_layout(
            Vec2::new(value_width, theme::HUD_SIZE),
            Layout::right_to_left(Align::Center),
            |ui| {
                hud_elided(ui, value, t.muted, value_width);
            },
        );
    });
    ui.add_space(4.0);
    hline(ui);
}

/// A read-only canvas preview with checkerboard and world-aligned grid.
///
/// This is deliberately not a canvas host: it has no input handling, selection
/// or tool state. Feature crates replace it with their own canvas when they land.
pub fn canvas_preview(ui: &mut Ui, height: f32, grid_size: i32) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height.max(64.0)),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.canvas);
    let mut viewport = Viewport::new(rect);
    viewport.set_zoom(1.0);
    draw_checkerboard(ui.painter(), rect, 16.0, &t);
    draw_grid(ui.painter(), rect, &viewport, grid_size, &t);
    ui.painter()
        .rect_stroke(rect, CornerRadius::ZERO, t.hairline(), StrokeKind::Inside);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_primitives_render_without_panicking() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            page(ui, |ui| {
                page_header(
                    ui,
                    "LEVEL",
                    "Level_0",
                    "Level canvas and tools. The canvas host arrives in T-0101.",
                );
                ui.add_space(16.0);
                meta_row(ui, "LEVEL ID", "#1");
                meta_row(ui, "SIZE", "1920 x 1080 px");
                ui.add_space(16.0);
                hud(ui, "CANVAS PREVIEW", Tokens::DARK.faint);
                ui.add_space(6.0);
                canvas_preview(ui, 180.0, 16);
                ui.add_space(12.0);
                empty_state(
                    ui,
                    "CANVAS HOST",
                    "Level canvas is not wired yet",
                    "Pan, zoom, layers and tools arrive with T-0101.",
                    "READ-ONLY PREVIEW",
                );
            });
        });
        output.textures_delta.clear();
    }
}
