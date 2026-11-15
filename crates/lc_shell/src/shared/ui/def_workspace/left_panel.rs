//! Left panel component for definition workspaces.

use egui::{
    vec2, Align, CornerRadius, FontId, Frame, Id, Layout, Margin, Pos2, Rect, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, TextEdit, Ui,
};
use lc_canvas::theme::Tokens;

use super::{
    def_row_layout, DEF_CARD_GAP, DEF_CARD_HEIGHT, DEF_CARD_STROKE_NORMAL,
    DEF_CARD_STROKE_SELECTED, DEF_CONTROL_HEIGHT, DEF_PANEL_PADDING, DEF_PANEL_WIDTH,
    DEF_SEARCH_HEIGHT,
};
use crate::shared::ui::drag_list::{
    draw_drag_handle, paint_placeholder, DragListState, DragOutcome, DragTheme,
};
use crate::shared::ui::reorder::Slot;

/// Configuration for the definition workspace left panel header and buttons.
#[derive(Debug, Clone, Copy)]
pub struct DefLeftPanelConfig<'a> {
    /// Label for the "+ New X" button.
    pub new_btn_label: &'a str,
    /// Whether to show the top "⋮" menu button next to "+ New X".
    pub has_menu_btn: bool,
    /// Prefix title for the items count (e.g. "ENUM DEFINITIONS").
    pub header_title: &'a str,
}

impl<'a> DefLeftPanelConfig<'a> {
    /// Creates a configuration with the given new button label and header title.
    pub fn new(new_btn_label: &'a str, header_title: &'a str) -> Self {
        Self {
            new_btn_label,
            has_menu_btn: false,
            header_title,
        }
    }

    /// Sets whether the top "⋮" menu button is displayed.
    pub fn with_menu(mut self, has_menu_btn: bool) -> Self {
        self.has_menu_btn = has_menu_btn;
        self
    }
}

/// Action emitted by the definition left panel during interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefLeftPanelAction<ID> {
    /// The "+ New X" button was clicked.
    NewClicked,
    /// The top "⋮" menu button was clicked.
    MenuClicked,
    /// An item was selected by clicking its row.
    Selected(ID),
    /// A context menu was requested via right-click or kebab button.
    ContextMenu {
        /// Item ID.
        id: ID,
        /// Screen position for the context popup.
        pos: Pos2,
    },
    /// Items were reordered via drag-and-drop.
    Reorder {
        /// Original index before move.
        from: usize,
        /// Target insertion index.
        to: usize,
    },
}

/// A definition item to be displayed in the left panel.
#[derive(Debug, Clone)]
pub struct DefItem<ID> {
    /// Stable identifier.
    pub id: ID,
    /// Display name.
    pub name: String,
}

impl<ID> DefItem<ID> {
    /// Creates a new definition item.
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}

/// Renders a single definition row card.
fn render_card_row<ID: Copy>(
    ui: &mut Ui,
    t: &Tokens,
    item: &DefItem<ID>,
    is_selected: bool,
    indices: (usize, usize),
    drag_state: &mut DragListState,
    actions: &mut Vec<DefLeftPanelAction<ID>>,
) -> Rect {
    let (order_idx, real_idx) = indices;
    let card_w = ui.available_width();
    let (row_rect, row_resp) =
        ui.allocate_exact_size(vec2(card_w, DEF_CARD_HEIGHT), Sense::click_and_drag());
    let layout = def_row_layout(row_rect);

    let handle_resp = ui.interact(layout.handle, Id::new(("def_h", real_idx)), Sense::drag());
    if handle_resp.drag_started() {
        drag_state.begin(ui.ctx(), order_idx, row_rect);
    }

    let kebab_resp = ui.interact(layout.kebab, Id::new(("def_k", real_idx)), Sense::click());
    if kebab_resp.clicked() {
        actions.push(DefLeftPanelAction::ContextMenu {
            id: item.id,
            pos: layout.kebab.left_bottom(),
        });
    }

    if row_resp.secondary_clicked() {
        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
            actions.push(DefLeftPanelAction::ContextMenu { id: item.id, pos });
        }
    }

    if row_resp.clicked() && !kebab_resp.clicked() && !handle_resp.dragged() {
        actions.push(DefLeftPanelAction::Selected(item.id));
    }

    let bg_color = if is_selected || row_resp.hovered() {
        t.sunken
    } else {
        t.panel
    };
    ui.painter().rect_filled(
        row_rect,
        CornerRadius::same(super::DEF_CARD_CORNER_RADIUS),
        bg_color,
    );

    let stroke = if is_selected {
        Stroke::new(DEF_CARD_STROKE_SELECTED, t.accent)
    } else {
        Stroke::new(DEF_CARD_STROKE_NORMAL, t.line_strong)
    };
    ui.painter().rect_stroke(
        row_rect,
        CornerRadius::same(super::DEF_CARD_CORNER_RADIUS),
        stroke,
        StrokeKind::Inside,
    );

    let handle_col = if handle_resp.hovered() || handle_resp.dragged() {
        t.accent
    } else {
        t.muted
    };
    draw_drag_handle(ui.painter(), layout.handle, handle_col);

    let text_col = if is_selected {
        t.accent_ink
    } else {
        t.ink_strong
    };
    let galley =
        ui.painter()
            .layout_no_wrap(item.name.clone(), FontId::proportional(12.0), text_col);
    let text_pos = Pos2::new(
        layout.name.min.x,
        layout.name.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, galley, text_col);

    let kebab_col = if kebab_resp.hovered() {
        t.ink_strong
    } else {
        t.muted
    };
    let kg = ui
        .painter()
        .layout_no_wrap("⋮".to_owned(), FontId::proportional(13.0), kebab_col);
    let kp = Pos2::new(
        layout.kebab.center().x - kg.size().x / 2.0,
        layout.kebab.center().y - kg.size().y / 2.0,
    );
    ui.painter().galley(kp, kg, kebab_col);

    row_rect
}

/// Renders the definition workspace left panel.
pub fn def_left_panel<ID: Copy + PartialEq + std::fmt::Debug>(
    ui: &mut Ui,
    t: &Tokens,
    config: &DefLeftPanelConfig<'_>,
    search_query: &mut String,
    items: &[DefItem<ID>],
    selected_id: Option<ID>,
    drag_state: &mut DragListState,
) -> Vec<DefLeftPanelAction<ID>> {
    let mut actions = Vec::new();
    let panel_w = DEF_PANEL_WIDTH;

    ui.allocate_ui_with_layout(
        vec2(panel_w, ui.available_height()),
        Layout::top_down(Align::Min),
        |ui| {
            ui.set_width(panel_w);

            Frame::new()
                .fill(t.panel)
                .inner_margin(Margin::same(DEF_PANEL_PADDING as i8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Top row
                    ui.horizontal(|ui| {
                        let menu_space = if config.has_menu_btn {
                            DEF_CONTROL_HEIGHT + 6.0
                        } else {
                            0.0
                        };
                        let new_w = (ui.available_width() - menu_space).max(40.0);
                        let new_btn = ui.add_sized(
                            [new_w, DEF_CONTROL_HEIGHT],
                            egui::Button::new(
                                RichText::new(config.new_btn_label)
                                    .size(12.0)
                                    .strong()
                                    .color(t.ink_strong),
                            )
                            .fill(t.sunken)
                            .stroke(t.hairline())
                            .corner_radius(CornerRadius::same(3)),
                        );
                        if new_btn.clicked() {
                            actions.push(DefLeftPanelAction::NewClicked);
                        }

                        if config.has_menu_btn {
                            let mb = ui.add_sized(
                                [DEF_CONTROL_HEIGHT, DEF_CONTROL_HEIGHT],
                                egui::Button::new(
                                    RichText::new("⋮").size(14.0).strong().color(t.muted),
                                )
                                .fill(t.sunken)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::same(3)),
                            );
                            if mb.clicked() {
                                actions.push(DefLeftPanelAction::MenuClicked);
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Search box
                    ui.add_sized(
                        [ui.available_width(), DEF_SEARCH_HEIGHT],
                        TextEdit::singleline(search_query)
                            .hint_text("Search definitions...")
                            .vertical_align(Align::Center)
                            .margin(Margin::symmetric(8, 0)),
                    );

                    ui.add_space(12.0);

                    // Header: "X DEFINITIONS (N)"
                    ui.label(
                        RichText::new(format!("{} ({})", config.header_title, items.len()))
                            .size(11.0)
                            .strong()
                            .color(t.accent_ink),
                    );

                    ui.add_space(10.0);

                    // Filtered list
                    let query = search_query.trim().to_lowercase();
                    let filtered: Vec<usize> = items
                        .iter()
                        .enumerate()
                        .filter(|(_, it)| {
                            query.is_empty() || it.name.to_lowercase().contains(&query)
                        })
                        .map(|(i, _)| i)
                        .collect();

                    let drag_theme = DragTheme::from_accent(t.accent);
                    let mut card_rects = Vec::with_capacity(filtered.len());

                    ScrollArea::vertical()
                        .id_salt("def_left_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for slot in drag_state.layout(filtered.len()) {
                                let order_idx = match slot {
                                    Slot::Item(i) => i,
                                    Slot::Placeholder => {
                                        let ph_h =
                                            drag_state.row_size().map_or(DEF_CARD_HEIGHT, |s| s.y);
                                        let (ph_rect, _) = ui.allocate_exact_size(
                                            vec2(ui.available_width(), ph_h),
                                            Sense::hover(),
                                        );
                                        paint_placeholder(ui.painter(), ph_rect, &drag_theme);
                                        card_rects.push(ph_rect);
                                        ui.add_space(DEF_CARD_GAP);
                                        continue;
                                    }
                                };

                                let Some(&real_idx) = filtered.get(order_idx) else {
                                    continue;
                                };
                                let Some(item) = items.get(real_idx) else {
                                    continue;
                                };

                                let is_selected = selected_id == Some(item.id);
                                let rect = render_card_row(
                                    ui,
                                    t,
                                    item,
                                    is_selected,
                                    (order_idx, real_idx),
                                    drag_state,
                                    &mut actions,
                                );
                                card_rects.push(rect);
                                ui.add_space(DEF_CARD_GAP);
                            }

                            drag_state.update(ui, &card_rects, &drag_theme);
                        });

                    if let DragOutcome::Dropped { from, to } = drag_state.finish(ui) {
                        if let (Some(&rf), Some(&rt)) = (filtered.get(from), filtered.get(to)) {
                            actions.push(DefLeftPanelAction::Reorder { from: rf, to: rt });
                        }
                    }
                });

            // Vertical divider line
            let panel_rect = ui.min_rect();
            ui.painter().line_segment(
                [panel_rect.right_top(), panel_rect.right_bottom()],
                Stroke::new(1.0, t.line_strong),
            );
        },
    );

    actions
}
