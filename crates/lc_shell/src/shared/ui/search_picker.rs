//! Universal, searchable picker popup with tag-based grouping.
//!
//! The picker is a popup anchored to a toggle button: a search field on top, an
//! optional `-- None --` row, flat rows grouped by tags (LDtk style), keyboard
//! navigation and a row cap with scrolling. It knows nothing about tags, enums,
//! tilesets or the shell app; callers pass [`PickerItem`]s and own the
//! [`SearchPickerState`] between frames.
//!
//! Tags only *categorize* here: an item may carry several tag names and appears
//! under each of those groups. They never carry data.
//!
//! # Frame flow
//! 1. Draw the toggle button (`egui::Button`, menu button, ...).
//! 2. Call [`search_picker`] with that button's [`egui::Response`] every frame.
//! 3. Act on the returned [`PickerChoice`] after drawing finishes.

use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Layout, Margin, Popup, Pos2, Rect,
    Response, RichText, ScrollArea, Sense, TextEdit, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::context_menu::ContextMenuTheme;
use crate::shared::i18n::{common_keys, tr};

/// Height of one selectable row (matches the enum window control height).
pub const PICKER_ROW_HEIGHT: f32 = 32.0;

/// Height of one group header row.
pub const PICKER_HEADER_HEIGHT: f32 = 22.0;

/// Default number of visible item rows before the list starts scrolling.
pub const PICKER_MAX_ROWS: usize = 5;

/// Default popup width.
pub const PICKER_WIDTH: f32 = 200.0;

/// One selectable entry of the picker.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerItem<K> {
    /// Value returned when the entry is chosen.
    pub key: K,
    /// Text shown in the row and matched by the search field.
    pub label: String,
    /// Tag names used for grouping. Empty means "untagged".
    pub groups: Vec<String>,
    /// Greyed out and not selectable, but still visible.
    pub disabled: bool,
}

impl<K> PickerItem<K> {
    /// Creates an untagged, enabled item.
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            groups: Vec::new(),
            disabled: false,
        }
    }

    /// Adds tag names to the item (builder style).
    pub fn with_groups<I, S>(mut self, groups: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.groups = groups.into_iter().map(Into::into).collect();
        self
    }

    /// Marks the item disabled (builder style).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// View state of one picker, owned by the caller and kept between frames.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchPickerState {
    search: String,
    selected_idx: usize,
    was_open: bool,
}

impl SearchPickerState {
    /// Current search text.
    pub fn search(&self) -> &str {
        &self.search
    }

    /// Index of the highlighted row, as returned by [`picker_rows`].
    pub fn selected_idx(&self) -> usize {
        self.selected_idx
    }

    /// Whether the popup was open on the previous frame.
    pub fn was_open(&self) -> bool {
        self.was_open
    }
}

/// Layout and labels of one picker.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchPickerConfig<'a, K> {
    /// Placeholder of the search field, e.g. `"Search tags..."`.
    pub hint: &'a str,
    /// Visible item rows before the list scrolls. Defaults to [`PICKER_MAX_ROWS`].
    pub max_rows: usize,
    /// Popup width. Defaults to [`PICKER_WIDTH`].
    pub width: f32,
    /// Adds a first `-- None --` row with this label.
    pub none_label: Option<&'a str>,
    /// Value highlighted as the current one (marked with a check icon).
    pub current: Option<K>,
    /// Header of the group holding untagged items.
    pub group_untagged_label: &'a str,
    /// Header of a list that only has a single group. Empty hides the header so
    /// plain lists render exactly like an ungrouped list.
    pub ungrouped_label: &'a str,
}

impl<'a, K: Clone + PartialEq> SearchPickerConfig<'a, K> {
    /// Creates a config with the default metrics and no `-- None --` row.
    pub fn new(hint: &'a str) -> Self {
        Self {
            hint,
            max_rows: PICKER_MAX_ROWS,
            width: PICKER_WIDTH,
            none_label: None,
            current: None,
            group_untagged_label: "Untagged",
            ungrouped_label: "",
        }
    }
}

/// What the user chose this frame.
#[derive(Clone, Debug, PartialEq)]
pub enum PickerChoice<K> {
    /// The `-- None --` row was activated.
    None,
    /// A regular item was activated.
    Item(K),
}

/// One rendered row of the flattened list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickerRow {
    /// The optional `-- None --` row.
    NoneRow,
    /// A group header; never selectable.
    Header(String),
    /// An item, addressed by its index in the `items` slice.
    Item {
        /// Index into the `items` slice handed to [`picker_rows`].
        idx: usize,
        /// Whether the item is disabled (not selectable).
        disabled: bool,
    },
}

/// Returns `true` when the row can hold the keyboard selection.
fn is_selectable(row: &PickerRow) -> bool {
    match row {
        PickerRow::NoneRow => true,
        PickerRow::Header(_) => false,
        PickerRow::Item { disabled, .. } => !disabled,
    }
}

/// `true` when the query is empty or occurs in the label or in one of the tags.
fn item_matches<K>(item: &PickerItem<K>, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    if item.label.to_lowercase().contains(query_lower) {
        return true;
    }
    item.groups
        .iter()
        .any(|g| g.to_lowercase().contains(query_lower))
}

/// Groups the items that match `query`, without any egui types.
///
/// Groups are the union of every item's tags, sorted case-insensitively and
/// deduplicated case-insensitively. Untagged items form the first group
/// (`None`); an item with several tags appears under each of its groups. Groups
/// left empty by the search filter are dropped, headers included.
///
/// Returns `(group tag or None for untagged, indices into `items`)`.
pub fn group_items<K>(items: &[PickerItem<K>], query: &str) -> Vec<(Option<String>, Vec<usize>)> {
    let query_lower = query.trim().to_lowercase();

    let mut tags: Vec<String> = Vec::new();
    for item in items {
        for tag in &item.groups {
            if !tags.iter().any(|known| known.eq_ignore_ascii_case(tag)) {
                tags.push(tag.clone());
            }
        }
    }
    tags.sort_by(|a, b| {
        a.to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b))
    });

    let mut groups: Vec<(Option<String>, Vec<usize>)> = Vec::new();

    let untagged: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.groups.is_empty() && item_matches(item, &query_lower))
        .map(|(idx, _)| idx)
        .collect();
    if !untagged.is_empty() {
        groups.push((None, untagged));
    }

    for tag in tags {
        let members: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.groups.iter().any(|g| g.eq_ignore_ascii_case(&tag))
                    && item_matches(item, &query_lower)
            })
            .map(|(idx, _)| idx)
            .collect();
        if !members.is_empty() {
            groups.push((Some(tag), members));
        }
    }

    groups
}

/// Flattens the grouped, filtered items into the rows drawn by the popup.
///
/// `none_label` prepends the optional `-- None --` row. Headers are omitted when
/// there is only one group and `ungrouped_label` is empty, so plain lists look
/// unchanged; `ungrouped_label` is used as that single header otherwise.
pub fn picker_rows<K>(
    items: &[PickerItem<K>],
    query: &str,
    none_label: Option<&str>,
    group_untagged_label: &str,
    ungrouped_label: &str,
) -> Vec<PickerRow> {
    let groups = group_items(items, query);
    let group_count = groups.len();

    let mut rows: Vec<PickerRow> = Vec::new();
    if none_label.is_some() {
        rows.push(PickerRow::NoneRow);
    }

    let show_headers = group_count > 1 || !ungrouped_label.is_empty();
    for (tag, members) in groups {
        if show_headers {
            let label = match &tag {
                Some(name) => name.clone(),
                None if group_count == 1 => ungrouped_label.to_owned(),
                None => group_untagged_label.to_owned(),
            };
            rows.push(PickerRow::Header(label));
        }
        for idx in members {
            let disabled = items.get(idx).is_some_and(|item| item.disabled);
            rows.push(PickerRow::Item { idx, disabled });
        }
    }

    rows
}

/// Index of the first selectable row, if any.
pub fn first_selectable(rows: &[PickerRow]) -> Option<usize> {
    rows.iter().position(is_selectable)
}

/// Index of the first selectable item row (skips the `-- None --` row).
pub fn first_item_row(rows: &[PickerRow]) -> Option<usize> {
    rows.iter().position(|row| match row {
        PickerRow::Item { disabled, .. } => !disabled,
        _ => false,
    })
}

/// Moves the selection one selectable row forward or backward.
///
/// Headers and disabled rows are skipped. At either end the selection stays on
/// the current row. An invalid or non-selectable `current` falls back to the
/// first selectable row.
pub fn step_selection(rows: &[PickerRow], current: Option<usize>, forward: bool) -> Option<usize> {
    let Some(cur) = current.filter(|idx| rows.get(*idx).is_some_and(is_selectable)) else {
        return first_selectable(rows);
    };

    let next = if forward {
        ((cur + 1)..rows.len()).find(|idx| rows.get(*idx).is_some_and(is_selectable))
    } else {
        (0..cur)
            .rev()
            .find(|idx| rows.get(*idx).is_some_and(is_selectable))
    };

    Some(next.unwrap_or(cur))
}

/// Draws the picker popup anchored to `anchor` and reports the user's choice.
///
/// The popup keeps its own open/closed state (egui popup memory) and resets the
/// search text and selection the first frame it opens. `state` must be the same
/// instance every frame for the same `id`.
pub fn search_picker<K: Clone + PartialEq>(
    anchor: &Response,
    id: Id,
    items: &[PickerItem<K>],
    state: &mut SearchPickerState,
    cfg: &SearchPickerConfig<'_, K>,
) -> Option<PickerChoice<K>> {
    let ctx = anchor.ctx.clone();
    let is_open_now = Popup::is_id_open(&ctx, id);
    let just_opened = is_open_now && !state.was_open;
    state.was_open = is_open_now;
    if just_opened {
        state.search.clear();
        state.selected_idx = 0;
    }

    let t = Tokens::get(&ctx);
    let mut choice: Option<PickerChoice<K>> = None;

    Popup::from_toggle_button_response(anchor)
        .id(id)
        .frame(ContextMenuTheme::default().frame(&t))
        .width(cfg.width)
        .show(|ui| {
            ui.set_width(cfg.width);
            ui.spacing_mut().interact_size.y = PICKER_ROW_HEIGHT;

            // Search field (outside the scroll area so it never scrolls away).
            ui.horizontal(|ui| {
                let icon_rect = Rect::from_min_size(ui.cursor().min, vec2(16.0, PICKER_ROW_HEIGHT));
                paint_icon(
                    ui.painter(),
                    icon_rect.center(),
                    Icon::Search,
                    12.0,
                    t.muted,
                );
                ui.add_space(20.0);
                let search_w = ui.available_width().max(60.0);
                let search_resp = ui.add_sized(
                    [search_w, PICKER_ROW_HEIGHT],
                    TextEdit::singleline(&mut state.search)
                        .id(id.with("search_input"))
                        .hint_text(cfg.hint)
                        .vertical_align(Align::Center)
                        .margin(Margin::symmetric(6, 0)),
                );
                if just_opened {
                    search_resp.request_focus();
                }
            });

            ui.add_space(4.0);

            let rows = picker_rows(
                items,
                &state.search,
                cfg.none_label,
                cfg.group_untagged_label,
                cfg.ungrouped_label,
            );

            if just_opened {
                state.selected_idx = first_item_row(&rows)
                    .or_else(|| first_selectable(&rows))
                    .unwrap_or(0);
            } else if !rows.get(state.selected_idx).is_some_and(is_selectable) {
                state.selected_idx = first_selectable(&rows).unwrap_or(0);
            }

            if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                if let Some(next) = step_selection(&rows, Some(state.selected_idx), true) {
                    state.selected_idx = next;
                }
            }
            if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                if let Some(prev) = step_selection(&rows, Some(state.selected_idx), false) {
                    state.selected_idx = prev;
                }
            }
            if ui.input(|i| i.key_pressed(Key::Enter)) {
                match rows.get(state.selected_idx) {
                    Some(PickerRow::NoneRow) => {
                        choice = Some(PickerChoice::None);
                        ui.close();
                    }
                    Some(PickerRow::Item {
                        idx,
                        disabled: false,
                    }) => {
                        if let Some(item) = items.get(*idx) {
                            choice = Some(PickerChoice::Item(item.key.clone()));
                            ui.close();
                        }
                    }
                    _ => {}
                }
            }
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                ui.close();
            }

            let list_height = cfg.max_rows.max(1) as f32 * PICKER_ROW_HEIGHT;
            ScrollArea::vertical()
                .id_salt(id.with("scroll"))
                .max_height(list_height)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if rows.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(tr(common_keys::SEARCH_NO_RESULTS))
                                    .size(12.0)
                                    .color(t.muted),
                            );
                            ui.add_space(8.0);
                        });
                        return;
                    }

                    for (row_idx, row) in rows.iter().enumerate() {
                        match row {
                            PickerRow::Header(label) => {
                                let (rect, _) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), PICKER_HEADER_HEIGHT),
                                    Sense::hover(),
                                );
                                ui.painter().text(
                                    Pos2::new(rect.min.x + 8.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    label,
                                    FontId::proportional(11.0),
                                    t.muted,
                                );
                            }
                            PickerRow::NoneRow => {
                                let label = cfg.none_label.unwrap_or("-- None --");
                                let is_current = cfg.current.is_none();
                                let clicked = draw_row(
                                    ui,
                                    &t,
                                    row_idx,
                                    label,
                                    false,
                                    is_current,
                                    &mut state.selected_idx,
                                );
                                if clicked {
                                    choice = Some(PickerChoice::None);
                                    ui.close();
                                }
                            }
                            PickerRow::Item { idx, disabled } => {
                                let Some(item) = items.get(*idx) else {
                                    continue;
                                };
                                let is_current =
                                    cfg.current.as_ref().is_some_and(|c| c == &item.key);
                                let clicked = draw_row(
                                    ui,
                                    &t,
                                    row_idx,
                                    &item.label,
                                    *disabled,
                                    is_current,
                                    &mut state.selected_idx,
                                );
                                if clicked {
                                    choice = Some(PickerChoice::Item(item.key.clone()));
                                    ui.close();
                                }
                            }
                        }
                    }
                });
        });

    choice
}

/// Draws one flat row and returns `true` when it was clicked.
fn draw_row(
    ui: &mut Ui,
    t: &Tokens,
    row_idx: usize,
    label: &str,
    disabled: bool,
    is_current: bool,
    selected_idx: &mut usize,
) -> bool {
    let sense = if disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), PICKER_ROW_HEIGHT), sense);

    let hovered = resp.hovered() && !disabled;
    if hovered {
        *selected_idx = row_idx;
    }

    let fill = if hovered || *selected_idx == row_idx {
        t.sunken
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);

    let mut text_left = rect.min.x + 8.0;
    if is_current {
        paint_icon(
            ui.painter(),
            Pos2::new(text_left + 5.0, rect.center().y),
            Icon::Check,
            11.0,
            t.accent,
        );
        text_left += 18.0;
    }

    let color = if disabled {
        t.muted.linear_multiply(0.5)
    } else {
        t.ink_strong
    };
    let text_rect = Rect::from_min_max(
        Pos2::new(text_left, rect.min.y),
        Pos2::new(rect.max.x - 8.0, rect.max.y),
    );
    ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::left_to_right(Align::Center)),
    )
    .add(
        egui::Label::new(RichText::new(label).size(12.0).color(color))
            .truncate()
            .selectable(false),
    );

    let resp = resp.on_hover_text(label);
    !disabled && resp.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_keyed(key: &'static str, label: &str, groups: &[&str]) -> PickerItem<&'static str> {
        PickerItem::new(key, label).with_groups(groups.iter().copied())
    }

    fn labels(rows: &[PickerRow]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                PickerRow::NoneRow => "<none>".to_owned(),
                PickerRow::Header(label) => format!("#{label}"),
                PickerRow::Item { idx, disabled } => {
                    format!("{idx}{}", if *disabled { "!" } else { "" })
                }
            })
            .collect()
    }

    #[test]
    fn multi_tag_items_appear_under_each_group() {
        let items = vec![
            item_keyed("a", "Alpha", &["Terrain", "Props"]),
            item_keyed("b", "Beta", &["Terrain"]),
            item_keyed("c", "Gamma", &[]),
        ];
        let groups = group_items(&items, "");

        // Untagged first, then alphabetical tags.
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].0, None);
        assert_eq!(groups[0].1, vec![2]);
        assert_eq!(groups[1].0.as_deref(), Some("Props"));
        assert_eq!(groups[1].1, vec![0]);
        assert_eq!(groups[2].0.as_deref(), Some("Terrain"));
        assert_eq!(groups[2].1, vec![0, 1]);
    }

    #[test]
    fn untagged_group_leads_and_is_absent_when_empty() {
        let items = vec![
            item_keyed("a", "Alpha", &["Zeta"]),
            item_keyed("b", "Beta", &["alpha"]),
        ];
        let groups = group_items(&items, "");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0.as_deref(), Some("alpha"));
        assert_eq!(groups[1].0.as_deref(), Some("Zeta"));

        let tagged_only = vec![item_keyed("a", "Alpha", &["Zeta"])];
        assert!(group_items(&tagged_only, "")
            .iter()
            .all(|(tag, _)| tag.is_some()));
    }

    #[test]
    fn tag_groups_sort_case_insensitively() {
        let items = vec![
            item_keyed("a", "A", &["beta"]),
            item_keyed("b", "B", &["Alpha"]),
            item_keyed("c", "C", &["gamma"]),
            item_keyed("d", "D", &["Beta"]),
        ];
        let groups = group_items(&items, "");
        let names: Vec<&str> = groups
            .iter()
            .filter_map(|(tag, _)| tag.as_deref())
            .collect();
        assert_eq!(names, vec!["Alpha", "beta", "gamma"]);

        // Case-insensitive dedupe keeps the first spelling and one group.
        let beta = groups
            .iter()
            .find(|(tag, _)| tag.as_deref() == Some("beta"))
            .expect("beta group");
        assert_eq!(beta.1, vec![0, 3]);
    }

    #[test]
    fn search_hides_empty_groups_and_matches_tags() {
        let items = vec![
            item_keyed("a", "Alpha", &["Terrain"]),
            item_keyed("b", "Beta", &["Props"]),
            item_keyed("c", "Gamma", &[]),
        ];

        // Label match only keeps the group that still has members.
        let rows = picker_rows(&items, "gam", None, "Untagged", "All items");
        assert_eq!(labels(&rows), vec!["#All items", "2"]);

        // A tag match keeps every item of that tag, and hides the other groups.
        let rows = picker_rows(&items, "prop", None, "Untagged", "All items");
        assert_eq!(labels(&rows), vec!["#Props", "1"]);

        // Nothing matches: no rows at all (headers included).
        let rows = picker_rows(&items, "zzz", None, "Untagged", "All items");
        assert!(rows.is_empty());
    }

    #[test]
    fn single_group_collapses_the_header() {
        let flat = vec![item_keyed("a", "Alpha", &[]), item_keyed("b", "Beta", &[])];
        // Empty ungrouped label: no header, plain list.
        let rows = picker_rows(&flat, "", None, "Untagged", "");
        assert_eq!(labels(&rows), vec!["0", "1"]);

        // A label opts into a single header.
        let rows = picker_rows(&flat, "", None, "Untagged", "All items");
        assert_eq!(labels(&rows), vec!["#All items", "0", "1"]);

        // Multi-group lists always show headers, untagged one first.
        let mixed = vec![
            item_keyed("a", "Alpha", &[]),
            item_keyed("b", "Beta", &["Props"]),
        ];
        let rows = picker_rows(&mixed, "", None, "Untagged", "");
        assert_eq!(labels(&rows), vec!["#Untagged", "0", "#Props", "1"]);
    }

    #[test]
    fn none_row_is_first_and_selectable() {
        let items = vec![item_keyed("a", "Alpha", &[])];
        let rows = picker_rows(&items, "", Some("-- None --"), "Untagged", "");
        assert_eq!(labels(&rows), vec!["<none>", "0"]);
        assert_eq!(first_selectable(&rows), Some(0));
        assert_eq!(first_item_row(&rows), Some(1));
    }

    #[test]
    fn selection_skips_headers_and_disabled_rows() {
        let items = vec![
            item_keyed("a", "Alpha", &["Props"]).disabled(true),
            item_keyed("b", "Beta", &["Props"]),
            item_keyed("c", "Gamma", &["Terrain"]),
        ];
        let rows = picker_rows(&items, "", None, "Untagged", "");
        // #Props, 0(disabled), 1, #Terrain, 2
        assert_eq!(labels(&rows), vec!["#Props", "0!", "1", "#Terrain", "2"]);

        let mut sel = Some(2);
        sel = step_selection(&rows, sel, true);
        assert_eq!(sel, Some(4), "arrow down jumps over the header");
        sel = step_selection(&rows, sel, false);
        assert_eq!(sel, Some(2), "arrow up jumps back over the header");
    }

    #[test]
    fn selection_clamps_at_both_ends() {
        let items = vec![item_keyed("a", "Alpha", &[]), item_keyed("b", "Beta", &[])];
        let rows = picker_rows(&items, "", None, "Untagged", "");

        assert_eq!(step_selection(&rows, Some(0), false), Some(0));
        assert_eq!(step_selection(&rows, Some(1), true), Some(1));
        // Invalid or disabled start falls back to the first selectable row.
        assert_eq!(step_selection(&rows, Some(99), true), Some(0));
        assert_eq!(step_selection(&rows, None, true), Some(0));
    }

    #[test]
    fn empty_list_has_no_selection() {
        let items: Vec<PickerItem<&'static str>> = Vec::new();
        let rows = picker_rows(&items, "", None, "Untagged", "");
        assert!(rows.is_empty());
        assert_eq!(first_selectable(&rows), None);
        assert_eq!(first_item_row(&rows), None);
        assert_eq!(step_selection(&rows, Some(0), true), None);
    }
}
