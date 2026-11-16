//! Left panel component for definition workspaces.

use egui::{
    vec2, Align, CornerRadius, Frame, Layout, Margin, Pos2, RichText, ScrollArea, Stroke, TextEdit,
    Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

use super::{DEF_PANEL_PADDING, DEF_PANEL_WIDTH, DEF_SEARCH_HEIGHT};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::reorder_list::{reorder_list, ListItem};
use crate::shared::ui::reorder_row::{
    GhostPreview, MoveDirection, SelectDirection, Trailing, ROW_LAYOUT_CARD,
};

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
            ui.set_max_width(panel_w);

            let inner_w = DEF_PANEL_WIDTH - 2.0 * DEF_PANEL_PADDING;
            Frame::new()
                .fill(t.panel)
                .inner_margin(Margin::same(DEF_PANEL_PADDING as i8))
                .show(ui, |ui| {
                    ui.set_min_width(inner_w);
                    ui.set_max_width(inner_w);

                    // Top row
                    ui.horizontal(|ui| {
                        let gap = 6.0;
                        ui.spacing_mut().item_spacing.x = gap;

                        let menu_space = if config.has_menu_btn { 28.0 + gap } else { 0.0 };
                        let new_w = (inner_w - menu_space).max(40.0);
                        let new_btn = ui.add_sized(
                            [new_w, 28.0],
                            egui::Button::new(
                                RichText::new(config.new_btn_label)
                                    .size(12.0)
                                    .strong()
                                    .color(t.invert_fg),
                            )
                            .truncate()
                            .fill(t.accent)
                            .stroke(t.hairline())
                            .corner_radius(CornerRadius::same(3)),
                        );
                        if new_btn.clicked() {
                            actions.push(DefLeftPanelAction::NewClicked);
                        }

                        if config.has_menu_btn {
                            let mb = icon_button(
                                ui,
                                Icon::DotsThreeVertical,
                                28.0,
                                IconButtonStyle::bordered(),
                            );
                            if mb.clicked() {
                                actions.push(DefLeftPanelAction::MenuClicked);
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Search box
                    ui.add_sized(
                        [inner_w, DEF_SEARCH_HEIGHT],
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

                    let list_items: Vec<ListItem> = filtered
                        .iter()
                        .map(|&idx| {
                            let it = &items[idx];
                            let is_selected = selected_id == Some(it.id);
                            ListItem::new(("def_item", idx), GhostPreview::new(&it.name))
                                .selected(is_selected)
                                .clickable(true)
                                .trailing(Trailing::Kebab)
                        })
                        .collect();

                    ScrollArea::vertical()
                        .id_salt("def_left_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(inner_w);
                            ui.set_max_width(inner_w);
                            let list_events = reorder_list(
                                ui,
                                drag_state,
                                &list_items,
                                ROW_LAYOUT_CARD,
                                |ui, slot_idx, body| {
                                    let real_idx = filtered[slot_idx];
                                    let item = &items[real_idx];
                                    let is_selected = selected_id == Some(item.id);
                                    body.slot(ui, 0, |ui| {
                                        let text_color =
                                            if is_selected { t.ink_strong } else { t.ink };
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(&item.name)
                                                    .strong()
                                                    .color(text_color)
                                                    .size(12.0),
                                            )
                                            .truncate()
                                            .selectable(false),
                                        );
                                    });
                                },
                            );

                            if let Some(slot_idx) = list_events.selected {
                                if let Some(&real_idx) = filtered.get(slot_idx) {
                                    actions.push(DefLeftPanelAction::Selected(items[real_idx].id));
                                }
                            }

                            if let Some((slot_idx, pos)) = list_events.context_menu_requested {
                                if let Some(&real_idx) = filtered.get(slot_idx) {
                                    actions.push(DefLeftPanelAction::ContextMenu {
                                        id: items[real_idx].id,
                                        pos,
                                    });
                                }
                            }

                            if let Some((from_slot, to_slot)) = list_events.moved {
                                if let (Some(&rf), Some(&rt)) =
                                    (filtered.get(from_slot), filtered.get(to_slot))
                                {
                                    actions.push(DefLeftPanelAction::Reorder { from: rf, to: rt });
                                }
                            }

                            if let Some((slot_idx, dir)) = list_events.move_requested {
                                match dir {
                                    MoveDirection::Up if slot_idx > 0 => {
                                        actions.push(DefLeftPanelAction::Reorder {
                                            from: filtered[slot_idx],
                                            to: filtered[slot_idx - 1],
                                        });
                                    }
                                    MoveDirection::Down if slot_idx + 1 < filtered.len() => {
                                        actions.push(DefLeftPanelAction::Reorder {
                                            from: filtered[slot_idx],
                                            to: filtered[slot_idx + 1],
                                        });
                                    }
                                    _ => {}
                                }
                            }

                            if let Some((slot_idx, dir)) = list_events.select_requested {
                                match dir {
                                    SelectDirection::Previous if slot_idx > 0 => {
                                        actions.push(DefLeftPanelAction::Selected(
                                            items[filtered[slot_idx - 1]].id,
                                        ));
                                    }
                                    SelectDirection::Next if slot_idx + 1 < filtered.len() => {
                                        actions.push(DefLeftPanelAction::Selected(
                                            items[filtered[slot_idx + 1]].id,
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                        });
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
