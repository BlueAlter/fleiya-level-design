//! Shared sub-tab strip used by definition windows (e.g. `ENUM DEF` / `DOCUMENT`).
//!
//! Every tab is laid out with identical padding, stroke width and font, so the
//! strip never shifts or resizes when the active tab changes. The inactive tab
//! keeps a transparent stroke of the same width instead of `Stroke::NONE`,
//! because egui adds the stroke width to a frame's outer size.
//!
//! The strip owns no state: callers pass the active index and read it back after
//! the call.

use egui::{
    vec2, Color32, CornerRadius, CursorIcon, FontId, Id, Response, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::theme::Tokens;

/// Font size of a tab label.
const TAB_FONT_SIZE: f32 = 11.0;

/// Horizontal padding inside a tab, left and right.
const TAB_PAD_X: f32 = 10.0;

/// Vertical padding inside a tab, top and bottom.
const TAB_PAD_Y: f32 = 4.0;

/// Width of the tab border, drawn inside the tab rect so it never changes size.
const TAB_STROKE: f32 = 1.0;

/// Corner radius of a tab.
const TAB_RADIUS: u8 = 2;

/// Returns the stable id of tab `index` in the strip drawn by `ui`.
///
/// Exposed so tests can measure individual tab rects across frames.
pub fn sub_tab_id(ui: &Ui, index: usize) -> Id {
    ui.id().with(("sub_tab", index))
}

/// Draws a strip of sub-tabs and returns `true` when the user picked another tab.
///
/// All tabs always use the same padding, stroke width and font; only the colors
/// depend on the active index, so switching tabs cannot move or resize anything.
pub fn sub_tabs(ui: &mut Ui, t: &Tokens, active: &mut usize, labels: &[&str]) -> bool {
    let base_id = ui.id();
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (index, label) in labels.iter().enumerate() {
            let is_active = *active == index;
            let color = if is_active { t.accent_ink } else { t.muted };
            let galley = ui.painter().layout_no_wrap(
                (*label).to_owned(),
                FontId::proportional(TAB_FONT_SIZE),
                color,
            );
            let size = vec2(
                galley.size().x + TAB_PAD_X * 2.0 + TAB_STROKE,
                galley.size().y + TAB_PAD_Y * 2.0 + TAB_STROKE,
            );

            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            let response: Response = ui
                .interact(rect, base_id.with(("sub_tab", index)), Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand);

            let fill = if is_active {
                t.sunken
            } else {
                Color32::TRANSPARENT
            };
            let stroke = Stroke::new(
                TAB_STROKE,
                if is_active {
                    t.accent
                } else {
                    Color32::TRANSPARENT
                },
            );
            ui.painter().rect(
                rect,
                CornerRadius::same(TAB_RADIUS),
                fill,
                stroke,
                StrokeKind::Inside,
            );
            ui.painter()
                .galley(rect.center() - galley.size() * 0.5, galley, color);

            if response.clicked() && !is_active {
                *active = index;
                changed = true;
            }
        }
    });
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_ids_are_stable_and_distinct() {
        let ctx = egui::Context::default();
        let mut ids = Vec::new();
        let _ = ctx.run_ui(Default::default(), |ui| {
            ids.push(sub_tab_id(ui, 0));
            ids.push(sub_tab_id(ui, 1));
        });
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }
}
