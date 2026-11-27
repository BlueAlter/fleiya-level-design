//! Left panel component for definition workspaces.

use egui::{
    vec2, Align, Color32, CornerRadius, Frame, Layout, Margin, Pos2, RichText, ScrollArea, Stroke,
    StrokeKind, TextEdit, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

use super::tag_groups::{group_items_by_tag, tag_group_container};
use super::{DEF_PANEL_PADDING, DEF_PANEL_WIDTH, DEF_SEARCH_HEIGHT};
use crate::shared::ui::drag_list::DragListState;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::reorder_list::{reorder_list, ListItem};
use crate::shared::ui::reorder_row::{
    GhostPreview, MoveDirection, SelectDirection, Slot as BodySlot, Trailing, ROW_LAYOUT_CARD,
};
use crate::shared::ui::swatch::TileIconData;

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

/// Optional 1:1 visual icon rendered on the left of a definition card.
#[derive(Debug, Clone)]
pub struct DefItemIcon {
    /// Base color of the item (used for fallback badge or border accent).
    pub color: Color32,
    /// Optional bound tile texture & normalized UV rectangle (rendered 1:1).
    pub tile: Option<TileIconData>,
    /// Whether the fallback color badge renders hollow (`true`) or filled (`false`).
    pub hollow: bool,
}

impl DefItemIcon {
    /// Creates a fallback color badge icon.
    pub fn color(color: Color32, hollow: bool) -> Self {
        Self {
            color,
            tile: None,
            hollow,
        }
    }

    /// Creates an icon with an optional bound tile texture (rendered 1:1).
    pub fn with_tile(color: Color32, hollow: bool, tile: Option<TileIconData>) -> Self {
        Self {
            color,
            tile,
            hollow,
        }
    }
}

/// A definition item to be displayed in the left panel.
#[derive(Debug, Clone)]
pub struct DefItem<ID> {
    /// Stable identifier.
    pub id: ID,
    /// Display name.
    pub name: String,
    /// Optional tags for collapsible tag-group rendering.
    pub tags: Vec<String>,
    /// Optional 1x1 bound tile / color icon shown on the left of the card.
    pub icon: Option<DefItemIcon>,
}

impl<ID> DefItem<ID> {
    /// Creates a new definition item.
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            tags: Vec::new(),
            icon: None,
        }
    }

    /// Attaches category tags for optional collapsible tag grouping.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Attaches a 1:1 visual icon descriptor shown inside the left panel card.
    pub fn with_icon(mut self, icon: DefItemIcon) -> Self {
        self.icon = Some(icon);
        self
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

                    let groups = group_items_by_tag(items, &filtered);
                    ScrollArea::vertical()
                        .id_salt("def_left_panel_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(inner_w);
                            ui.set_max_width(inner_w);

                            if groups.len() == 1 && groups[0].label.is_empty() {
                                render_def_group_list(
                                    ui,
                                    t,
                                    items,
                                    &groups[0].item_indices,
                                    selected_id,
                                    drag_state,
                                    &mut actions,
                                );
                            } else {
                                for group in &groups {
                                    let grp_drag_id =
                                        egui::Id::new(("def_tag_group_drag", &group.label));
                                    let mut grp_drag = ui.ctx().data_mut(|d| {
                                        d.get_temp::<DragListState>(grp_drag_id).unwrap_or_default()
                                    });
                                    tag_group_container(
                                        ui,
                                        t,
                                        ("def_tag_group_open", &group.label),
                                        &group.label,
                                        group.item_indices.len(),
                                        group.key.is_some(),
                                        |ui| {
                                            render_def_group_list(
                                                ui,
                                                t,
                                                items,
                                                &group.item_indices,
                                                selected_id,
                                                &mut grp_drag,
                                                &mut actions,
                                            );
                                        },
                                    );
                                    ui.ctx().data_mut(|d| d.insert_temp(grp_drag_id, grp_drag));
                                    ui.add_space(6.0);
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

fn render_def_group_list<ID: Copy + PartialEq + std::fmt::Debug>(
    ui: &mut Ui,
    t: &Tokens,
    items: &[DefItem<ID>],
    indices: &[usize],
    selected_id: Option<ID>,
    drag_state: &mut DragListState,
    actions: &mut Vec<DefLeftPanelAction<ID>>,
) {
    let has_icons = indices.iter().any(|&idx| items[idx].icon.is_some());
    let list_items: Vec<ListItem> = indices
        .iter()
        .map(|&idx| {
            let it = &items[idx];
            let is_selected = selected_id == Some(it.id);
            let mut li = ListItem::new(("def_item", idx), GhostPreview::new(&it.name))
                .selected(is_selected)
                .clickable(true)
                .trailing(Trailing::Kebab);
            if has_icons {
                li = li.with_slots([BodySlot::Fixed(22.0), BodySlot::Flex]);
            }
            li
        })
        .collect();

    let list_events = reorder_list(
        ui,
        drag_state,
        &list_items,
        ROW_LAYOUT_CARD,
        |ui, slot_idx, body| {
            let real_idx = indices[slot_idx];
            let item = &items[real_idx];
            let is_selected = selected_id == Some(item.id);
            let text_slot = if has_icons {
                body.slot(ui, 0, |ui| {
                    let (icon_rect, _) =
                        ui.allocate_exact_size(vec2(22.0, 22.0), egui::Sense::hover());
                    if ui.is_rect_visible(icon_rect) {
                        let p = ui.painter();
                        p.rect_filled(icon_rect, CornerRadius::ZERO, t.sunken);
                        if let Some(ref ic) = item.icon {
                            if let Some(tile) = ic.tile {
                                p.image(
                                    tile.texture_id,
                                    icon_rect.shrink(1.0),
                                    tile.uv,
                                    Color32::WHITE,
                                );
                            } else if ic.hollow {
                                p.rect_stroke(
                                    icon_rect.shrink(3.0),
                                    CornerRadius::ZERO,
                                    Stroke::new(2.0, ic.color),
                                    StrokeKind::Inside,
                                );
                            } else {
                                p.rect_filled(icon_rect.shrink(3.0), CornerRadius::ZERO, ic.color);
                            }
                        }
                        p.rect_stroke(
                            icon_rect,
                            CornerRadius::ZERO,
                            t.hairline(),
                            StrokeKind::Inside,
                        );
                    }
                });
                1
            } else {
                0
            };

            body.slot(ui, text_slot, |ui| {
                let text_color = if is_selected { t.ink_strong } else { t.ink };
                ui.add(
                    egui::Label::new(RichText::new(&item.name).strong().color(text_color))
                        .truncate()
                        .selectable(false),
                );
            });
        },
    );

    if let Some(slot_idx) = list_events.selected {
        if let Some(&real_idx) = indices.get(slot_idx) {
            actions.push(DefLeftPanelAction::Selected(items[real_idx].id));
        }
    }

    if let Some((slot_idx, pos)) = list_events.context_menu_requested {
        if let Some(&real_idx) = indices.get(slot_idx) {
            actions.push(DefLeftPanelAction::ContextMenu {
                id: items[real_idx].id,
                pos,
            });
        }
    }

    if let Some((from_slot, to_slot)) = list_events.moved {
        if let (Some(&rf), Some(&rt)) = (indices.get(from_slot), indices.get(to_slot)) {
            actions.push(DefLeftPanelAction::Reorder { from: rf, to: rt });
        }
    }

    if let Some((slot_idx, dir)) = list_events.move_requested {
        match dir {
            MoveDirection::Up if slot_idx > 0 => {
                actions.push(DefLeftPanelAction::Reorder {
                    from: indices[slot_idx],
                    to: indices[slot_idx - 1],
                });
            }
            MoveDirection::Down if slot_idx + 1 < indices.len() => {
                actions.push(DefLeftPanelAction::Reorder {
                    from: indices[slot_idx],
                    to: indices[slot_idx + 1],
                });
            }
            _ => {}
        }
    }

    if let Some((slot_idx, dir)) = list_events.select_requested {
        match dir {
            SelectDirection::Previous if slot_idx > 0 => {
                actions.push(DefLeftPanelAction::Selected(
                    items[indices[slot_idx - 1]].id,
                ));
            }
            SelectDirection::Next if slot_idx + 1 < indices.len() => {
                actions.push(DefLeftPanelAction::Selected(
                    items[indices[slot_idx + 1]].id,
                ));
            }
            _ => {}
        }
    }
}
