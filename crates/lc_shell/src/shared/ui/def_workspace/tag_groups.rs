//! Shared collapsible tag-group container helpers matching LDtk `groupUsingTags`.

use std::collections::BTreeMap;

use egui::{
    vec2, Align, Color32, CornerRadius, CursorIcon, Frame, Id, Layout, Margin, RichText, Sense,
    Stroke, Ui,
};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::{self, Tokens};

use super::DefItem;

/// A logical group of left-panel items grouped by tag (`None` = untagged).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagGroup {
    /// Tag name (`None` for untagged items).
    pub key: Option<String>,
    /// Display title (e.g. `"UNTAGGED"`, `"ACTOR"`). Empty when flat list.
    pub label: String,
    /// Indices into the caller's original `items` slice.
    pub item_indices: Vec<usize>,
}

/// Groups filtered `items` indices by their tags using LDtk's `groupUsingTags` rule:
/// - If `<= 1` distinct group exists (or no item has tags), returns a single flat group (`label = ""`).
/// - Otherwise returns `"UNTAGGED"` first (if any untagged item exists) followed by tags sorted A-Z.
///   Items with multiple tags appear under each of their tags.
pub fn group_items_by_tag<ID>(items: &[DefItem<ID>], filtered: &[usize]) -> Vec<TagGroup> {
    if filtered.is_empty() {
        return Vec::new();
    }

    let mut untagged = Vec::new();
    let mut by_tag: BTreeMap<String, (String, Vec<usize>)> = BTreeMap::new();

    for &idx in filtered {
        let item = &items[idx];
        if item.tags.is_empty() {
            untagged.push(idx);
        } else {
            for tag in &item.tags {
                let norm = tag.trim();
                if norm.is_empty() {
                    continue;
                }
                let sort_key = norm.to_lowercase();
                let entry = by_tag
                    .entry(sort_key)
                    .or_insert_with(|| (norm.to_string(), Vec::new()));
                if !entry.1.contains(&idx) {
                    entry.1.push(idx);
                }
            }
        }
    }

    let group_count = usize::from(!untagged.is_empty()) + by_tag.len();
    if by_tag.is_empty() || group_count <= 1 {
        return vec![TagGroup {
            key: None,
            label: String::new(),
            item_indices: filtered.to_vec(),
        }];
    }

    let mut groups = Vec::with_capacity(group_count);
    if !untagged.is_empty() {
        groups.push(TagGroup {
            key: None,
            label: "UNTAGGED".to_string(),
            item_indices: untagged,
        });
    }
    for (_k, (display, indices)) in by_tag {
        groups.push(TagGroup {
            key: Some(display.clone()),
            label: display.to_uppercase(),
            item_indices: indices,
        });
    }
    groups
}

/// Renders a shared collapsible tag-group container around child items (`CornerRadius::ZERO`).
///
/// Clicking the header toggles visibility of the group's child items cleanly via egui temp memory.
pub fn tag_group_container(
    ui: &mut Ui,
    t: &Tokens,
    group_id: impl egui::AsId,
    title: &str,
    count: usize,
    is_tagged: bool,
    body: impl FnOnce(&mut Ui),
) {
    let id = Id::new(group_id);
    let mut collapsed = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false));

    Frame::new()
        .fill(Color32::TRANSPARENT)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::same(0))
        .show(ui, |ui| {
            let header_w = ui.available_width().max(40.0);
            let (header_rect, mut header_resp) =
                ui.allocate_exact_size(vec2(header_w, 24.0), Sense::click());
            header_resp = header_resp.on_hover_cursor(CursorIcon::PointingHand);
            if header_resp.clicked() {
                collapsed = !collapsed;
                ui.ctx().data_mut(|d| d.insert_temp(id, collapsed));
            }

            if ui.is_rect_visible(header_rect) {
                let p = ui.painter();
                let (base_alpha, hover_alpha) = if is_tagged {
                    (
                        if t.dark { 0.18 } else { 0.15 },
                        if t.dark { 0.28 } else { 0.24 },
                    )
                } else {
                    (
                        if t.dark { 0.09 } else { 0.07 },
                        if t.dark { 0.16 } else { 0.13 },
                    )
                };
                let bg = theme::tint(
                    t.accent,
                    if header_resp.hovered() {
                        hover_alpha
                    } else {
                        base_alpha
                    },
                );
                let line_col = if is_tagged {
                    theme::tint(t.accent, 0.45)
                } else {
                    t.line
                };
                p.rect_filled(header_rect, CornerRadius::ZERO, bg);
                p.line_segment(
                    [header_rect.left_bottom(), header_rect.right_bottom()],
                    Stroke::new(1.0, line_col),
                );
                paint_icon(
                    p,
                    egui::pos2(header_rect.min.x + 12.0, header_rect.center().y),
                    if collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    },
                    10.0,
                    t.accent_ink,
                );
            }

            let text_rect = egui::Rect::from_min_max(
                egui::pos2(header_rect.min.x + 24.0, header_rect.min.y),
                header_rect.max,
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(text_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            child.add(
                egui::Label::new(
                    RichText::new(format!("{title} ({count})"))
                        .size(10.5)
                        .strong()
                        .color(t.accent_ink),
                )
                .truncate()
                .selectable(false),
            );

            if !collapsed {
                Frame::new()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin::symmetric(4, 5))
                    .show(ui, body);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_items_flat_when_no_tags_or_single_group() {
        let items = vec![DefItem::new(1, "A"), DefItem::new(2, "B")];
        let groups = group_items_by_tag(&items, &[0, 1]);
        assert_eq!(groups.len(), 1);
        assert!(groups[0].label.is_empty());

        let single_tag = vec![
            DefItem::new(1, "A").with_tags(vec!["Actor".into()]),
            DefItem::new(2, "B").with_tags(vec!["Actor".into()]),
        ];
        let g2 = group_items_by_tag(&single_tag, &[0, 1]);
        assert_eq!(g2.len(), 1);
        assert!(g2[0].label.is_empty());
    }

    #[test]
    fn group_items_separates_untagged_and_sorted_tags_with_multi_tag_duplication() {
        let items = vec![
            DefItem::new(1, "Player").with_tags(vec!["Region".into(), "Actor".into()]),
            DefItem::new(2, "Chest").with_tags(vec!["Loot".into()]),
            DefItem::new(3, "Wall"),
        ];
        let groups = group_items_by_tag(&items, &[0, 1, 2]);
        let labels: Vec<&str> = groups.iter().map(|g| g.label.as_str()).collect();
        assert_eq!(labels, vec!["UNTAGGED", "ACTOR", "LOOT", "REGION"]);
        assert_eq!(groups[0].item_indices, vec![2]);
        assert_eq!(groups[1].item_indices, vec![0]);
        assert_eq!(groups[2].item_indices, vec![1]);
        assert_eq!(groups[3].item_indices, vec![0]);
    }
}
