//! Tooltip drag ghost preview for enum definition cards.

use egui::{
    vec2, Align, Area, CornerRadius, CursorIcon, Id, Layout, Order, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui,
};

use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::EnumDef;

use super::types::{
    draw_six_dot_handle, DRAG_HANDLE_GAP, DRAG_HANDLE_WIDTH, ENUM_CARD_HEIGHT, ENUM_CARD_PADDING_X,
    ENUM_CARD_PADDING_Y, KEBAB_ICON_GAP, KEBAB_ICON_WIDTH,
};

pub fn render_card_drag_preview(
    ui: &mut Ui,
    t: &Tokens,
    dragged_enum: &EnumDef,
    ghost_pos: Pos2,
    card_w: f32,
) {
    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    Area::new(Id::new("enum_card_drag_preview"))
        .order(Order::Tooltip)
        .fixed_pos(ghost_pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            ui.set_min_size(vec2(card_w, ENUM_CARD_HEIGHT));
            ui.set_max_size(vec2(card_w, ENUM_CARD_HEIGHT));
            let preview_rect = Rect::from_min_size(ui.cursor().min, vec2(card_w, ENUM_CARD_HEIGHT));
            ui.allocate_rect(preview_rect, Sense::hover());
            ui.painter()
                .rect_filled(preview_rect, CornerRadius::same(4), t.sunken);
            ui.painter().rect_stroke(
                preview_rect,
                CornerRadius::same(4),
                Stroke::new(1.5, t.accent),
                StrokeKind::Inside,
            );

            let content_h = ENUM_CARD_HEIGHT - 2.0 * ENUM_CARD_PADDING_Y - 2.0;
            let cy = preview_rect.center().y;

            let hr = Rect::from_min_size(
                Pos2::new(
                    preview_rect.min.x + ENUM_CARD_PADDING_X + 1.0,
                    cy - content_h / 2.0,
                ),
                vec2(DRAG_HANDLE_WIDTH, content_h),
            );
            draw_six_dot_handle(ui.painter(), hr, true, t);

            let text_start_x = hr.max.x + DRAG_HANDLE_GAP;
            let text_end_x =
                preview_rect.max.x - ENUM_CARD_PADDING_X - 1.0 - KEBAB_ICON_WIDTH - KEBAB_ICON_GAP;
            let text_w = (text_end_x - text_start_x).max(10.0);
            let name_rect = Rect::from_min_size(
                Pos2::new(text_start_x, cy - content_h / 2.0),
                vec2(text_w, content_h),
            );
            ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(name_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            )
            .add(
                egui::Label::new(
                    RichText::new(&dragged_enum.name)
                        .strong()
                        .color(t.ink_strong),
                )
                .truncate(),
            );

            let kebab_center = Pos2::new(
                preview_rect.max.x - ENUM_CARD_PADDING_X - 1.0 - KEBAB_ICON_WIDTH / 2.0,
                cy,
            );
            lc_canvas::icons::paint_icon(
                ui.painter(),
                kebab_center,
                Icon::DotsThreeVertical,
                13.0,
                t.muted,
            );
        });
}
