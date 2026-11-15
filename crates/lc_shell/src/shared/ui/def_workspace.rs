//! Definition workspace chrome kit shared by enum, tileset, entity, and layer workspaces.
//!
//! Provides the standard layout pieces:
//! - [`def_left_panel`]: definitions list with search, "+ New X", drag reordering, and context menus
//! - [`def_right_header`]: top sub-tab strip, title heading "<name> (Kind)", and description
//! - [`def_section`]: accent section label with standard card frame
//! - [`def_field_row`]: bold field label on left, input control on right
//! - [`def_footer`]: docked footer with "ID: ... • Used by ..." status and action buttons

pub mod left_panel;

pub use left_panel::{def_left_panel, DefItem, DefLeftPanelAction, DefLeftPanelConfig};

use egui::{vec2, Align, CornerRadius, Frame, Layout, Margin, Pos2, Rect, RichText, Ui};
use lc_canvas::theme::Tokens;

use super::sub_tabs::sub_tabs;

// =============================================================================
// Constants: strokes, rounding, spacing, widths in ONE place
// =============================================================================

/// Fixed width of the definitions left panel.
pub const DEF_PANEL_WIDTH: f32 = 260.0;
/// Inner padding for the left definitions panel.
pub const DEF_PANEL_PADDING: f32 = 12.0;
/// Fixed height of a definition card row in the left panel.
pub const DEF_CARD_HEIGHT: f32 = 36.0;
/// Vertical gap between definition card rows.
pub const DEF_CARD_GAP: f32 = 5.0;
/// Horizontal padding inside a definition card.
pub const DEF_CARD_PADDING_X: f32 = 10.0;
/// Vertical padding inside a definition card.
pub const DEF_CARD_PADDING_Y: f32 = 6.0;
/// Corner radius of a definition card.
pub const DEF_CARD_CORNER_RADIUS: u8 = 4;
/// Height of standard buttons and single-line controls.
pub const DEF_CONTROL_HEIGHT: f32 = 32.0;
/// Height of the search input field.
pub const DEF_SEARCH_HEIGHT: f32 = 28.0;
/// Width allocated for the drag handle.
pub const DEF_HANDLE_WIDTH: f32 = 16.0;
/// Gap between drag handle and card text.
pub const DEF_HANDLE_GAP: f32 = 12.0;
/// Width of the "⋮" kebab menu button.
pub const DEF_KEBAB_WIDTH: f32 = 20.0;
/// Gap between card text and kebab menu button.
pub const DEF_KEBAB_GAP: f32 = 8.0;
/// Height of the docked detail footer.
pub const DEF_FOOTER_HEIGHT: f32 = 48.0;
/// Normal unselected border stroke width.
pub const DEF_CARD_STROKE_NORMAL: f32 = 1.0;
/// Selected border stroke width.
pub const DEF_CARD_STROKE_SELECTED: f32 = 1.5;

// =============================================================================
// Pure layout helpers
// =============================================================================

/// Sub-rects computed for a definition card row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefRowLayout {
    /// Bounding rect for the drag handle on the left.
    pub handle: Rect,
    /// Bounding rect for the left-aligned item name.
    pub name: Rect,
    /// Bounding rect for the "⋮" kebab menu on the right.
    pub kebab: Rect,
}

/// Pure helper returning the fixed width of the definition left panel.
pub const fn def_panel_width() -> f32 {
    DEF_PANEL_WIDTH
}

/// Pure helper computing sub-component rects for a definition card row.
pub fn def_row_layout(row_rect: Rect) -> DefRowLayout {
    let content_h = (row_rect.height() - 2.0 * DEF_CARD_PADDING_Y - 2.0).max(1.0);
    let cy = row_rect.center().y;

    let handle = Rect::from_min_size(
        Pos2::new(
            row_rect.min.x + DEF_CARD_PADDING_X + 1.0,
            cy - content_h / 2.0,
        ),
        vec2(DEF_HANDLE_WIDTH, content_h),
    );

    let kebab = Rect::from_min_size(
        Pos2::new(
            row_rect.max.x - DEF_CARD_PADDING_X - 1.0 - DEF_KEBAB_WIDTH,
            cy - content_h / 2.0,
        ),
        vec2(DEF_KEBAB_WIDTH, content_h),
    );

    let text_start_x = handle.max.x + DEF_HANDLE_GAP;
    let text_end_x = kebab.min.x - DEF_KEBAB_GAP;
    let text_w = (text_end_x - text_start_x).max(10.0);

    let name = Rect::from_min_size(
        Pos2::new(text_start_x, cy - content_h / 2.0),
        vec2(text_w, content_h),
    );

    DefRowLayout {
        handle,
        name,
        kebab,
    }
}

/// Formats the left status string for the definition footer.
pub fn format_footer_left(id_text: &str, used_by_text: &str, unsaved: bool) -> String {
    let mut s = format!("ID: {id_text} • Used by {used_by_text}");
    if unsaved {
        s.push_str(" • Unsaved changes");
    }
    s
}

// =============================================================================
// Workspace Chrome Components
// =============================================================================

/// Renders the definition workspace detail header.
///
/// Order:
/// 1. Sub-tabs row at the very top
/// 2. "<name> (<kind>)" heading
/// 3. One-line description
pub fn def_right_header(
    ui: &mut Ui,
    t: &Tokens,
    active_tab: &mut usize,
    tab_labels: &[&str],
    name: &str,
    kind: &str,
    description: &str,
) -> bool {
    let tab_changed = sub_tabs(ui, t, active_tab, tab_labels);
    ui.add_space(8.0);
    ui.add(
        egui::Label::new(
            RichText::new(format!("{name} ({kind})"))
                .size(20.0)
                .strong()
                .color(t.ink_strong),
        )
        .truncate(),
    );
    ui.add_space(4.0);
    ui.label(RichText::new(description).size(11.5).color(t.muted));
    ui.add_space(10.0);
    tab_changed
}

/// Renders an accent-labeled section card frame.
///
/// Places an accent label OUTSIDE the card, then the bordered card.
pub fn def_section<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.label(RichText::new(label).size(11.0).strong().color(t.accent_ink));
    ui.add_space(4.0);
    Frame::new()
        .fill(t.panel)
        .stroke(t.hairline())
        .inner_margin(Margin::same(12))
        .corner_radius(CornerRadius::same(DEF_CARD_CORNER_RADIUS))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contents(ui)
        })
        .inner
}

/// Renders a definition property row with a bold label on the left and a control on the right.
pub fn def_field_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong().color(t.ink_strong));
        control(ui)
    })
    .inner
}

/// Renders a docked definition footer with a top divider line, status info, and right buttons.
pub fn def_footer(ui: &mut Ui, t: &Tokens, left_text: &str, right_buttons: impl FnOnce(&mut Ui)) {
    let cursor = ui.cursor().min;
    let avail_w = ui.available_width();
    ui.painter().line_segment(
        [cursor, Pos2::new(cursor.x + avail_w, cursor.y)],
        t.hairline(),
    );

    Frame::new()
        .fill(t.panel)
        .inner_margin(Margin::symmetric(20, 10))
        .show(ui, |ui| {
            ui.set_min_height(DEF_FOOTER_HEIGHT - 20.0);
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new(left_text).size(11.5).color(t.muted));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    right_buttons(ui);
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    #[test]
    fn test_def_panel_width_constant() {
        assert_eq!(def_panel_width(), 260.0);
    }

    #[test]
    fn test_def_row_layout_geometry() {
        let r = Rect::from_min_size(pos2(10.0, 50.0), vec2(240.0, DEF_CARD_HEIGHT));
        let layout = def_row_layout(r);

        assert!(layout.handle.min.x >= r.min.x);
        assert!(layout.handle.max.x < layout.name.min.x);
        assert!(layout.name.max.x < layout.kebab.min.x);
        assert!(layout.kebab.max.x <= r.max.x);
        assert_eq!(layout.handle.width(), DEF_HANDLE_WIDTH);
        assert_eq!(layout.kebab.width(), DEF_KEBAB_WIDTH);
    }

    #[test]
    fn test_format_footer_left() {
        assert_eq!(
            format_footer_left("EnumDefId(1)", "2 Entity Def(s)", false),
            "ID: EnumDefId(1) • Used by 2 Entity Def(s)"
        );
        assert_eq!(
            format_footer_left("TilesetDefId(2)", "4 Level(s)", true),
            "ID: TilesetDefId(2) • Used by 4 Level(s) • Unsaved changes"
        );
    }
}
