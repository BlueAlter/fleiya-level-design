//! Standardized tag management row with chips, inline creation, renaming, and recall picker.
//!
//! Renders tags as chips with remove ("x") buttons, inline editing on double click,
//! inline creation via "+" or "(No tag)", and a recall dropdown for existing tags.
//! Purely functional: does not mutate data, emitting [`TagEvent`] instead.
//!
//! # Usage Example
//! ```ignore
//! use lc_shell::shared::ui::tag_row::{tag_row, TagEvent};
//!
//! let tags = vec!["Gameplay".to_string(), "NPC".to_string()];
//! let all_tags = vec!["Gameplay".to_string(), "NPC".to_string(), "Audio".to_string()];
//! let events = tag_row(ui, &tags, &all_tags);
//! for event in events {
//!     // handle TagEvent::Add, TagEvent::Remove, TagEvent::Rename
//! }
//! ```

use egui::{vec2, CornerRadius, CursorIcon, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;

use super::icon_button::{icon_button, IconButtonStyle};
use super::search_picker::{
    search_picker, PickerChoice, PickerItem, SearchPickerConfig, SearchPickerState,
};
use super::tag_input::{input_blocker, tag_text_input, TagInputEvent};
use super::workspace_metrics::{CONTROL_HEIGHT_FORM, ICON_BUTTON_SIZE_FORM};

/// Event emitted by user actions on the tag row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagEvent {
    /// Request to add an existing or newly created tag.
    Add(String),
    /// Request to remove a tag from the current entity.
    Remove(String),
    /// Request to rename an existing tag across the project.
    Rename { from: String, to: String },
}

/// Normalizes a single tag: trims whitespace, rejects empty, and returns sanitized text.
pub fn normalize_tag(tag: &str) -> Option<String> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Normalizes a list of tags: trims, rejects empty, and deduplicates case-insensitively
/// while preserving the casing of the first occurrence.
pub fn normalize_tags(tags: &[impl AsRef<str>]) -> Vec<String> {
    let mut result = Vec::new();
    for tag in tags {
        if let Some(norm) = normalize_tag(tag.as_ref()) {
            if !result
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(&norm))
            {
                result.push(norm);
            }
        }
    }
    result
}

/// Transient state for the tag row stored in egui temp data.
#[derive(Clone, Debug, Default)]
struct TagRowState {
    inline_new_active: bool,
    inline_new_text: String,
    editing_tag: Option<String>,
    editing_text: String,
    focus_pending: bool,
    recall_picker: SearchPickerState,
}

/// Renders a tag row containing chips with remove buttons, inline editing, and recall picker.
///
/// Binds no project or command types. Returns a list of [`TagEvent`] describing changes.
pub fn tag_row(ui: &mut Ui, tags: &[String], recall: &[String]) -> Vec<TagEvent> {
    let mut events = Vec::new();
    let t = Tokens::get(ui.ctx());
    let state_id = ui.id().with("tag_row_state");
    let mut state: TagRowState = ui
        .ctx()
        .data_mut(|d| d.get_temp(state_id).unwrap_or_default());

    let mut remove_tag = None;
    let mut create_tag = None;
    let mut rename_tag = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        // 1. Empty state: "(No tag)" chip
        if tags.is_empty() && !state.inline_new_active {
            let (no_tag_rect, no_tag_resp) =
                ui.allocate_exact_size(vec2(76.0, CONTROL_HEIGHT_FORM), Sense::click());
            let painter = ui.painter();
            let border_stroke = if no_tag_resp.hovered() {
                Stroke::new(1.0, t.muted)
            } else {
                Stroke::new(1.0, t.muted.linear_multiply(0.35))
            };
            painter.rect_stroke(
                no_tag_rect,
                CornerRadius::same(3),
                border_stroke,
                StrokeKind::Inside,
            );
            painter.text(
                no_tag_rect.center(),
                egui::Align2::CENTER_CENTER,
                "(No tag)",
                egui::FontId::proportional(12.0),
                t.muted,
            );
            if no_tag_resp.clicked() {
                state.inline_new_active = true;
                state.inline_new_text.clear();
                state.focus_pending = true;
            }
        }

        // 2. Existing tag chips
        for tag_name in tags {
            if state.editing_tag.as_deref() == Some(tag_name.as_str()) {
                // Inline editing mode for this tag
                let focus_now = state.focus_pending;
                state.focus_pending = false;

                let (edit_slot, _) =
                    ui.allocate_exact_size(vec2(100.0, CONTROL_HEIGHT_FORM), Sense::hover());
                let input_res = tag_text_input(
                    ui.ctx(),
                    ui.id().with(("tag_rename_input", tag_name)),
                    edit_slot,
                    &mut state.editing_text,
                    tag_name,
                    focus_now,
                );

                match input_res {
                    TagInputEvent::Commit => {
                        if let Some(new_val) = normalize_tag(&state.editing_text) {
                            if new_val != *tag_name {
                                rename_tag = Some((tag_name.clone(), new_val));
                            }
                        }
                        state.editing_tag = None;
                    }
                    TagInputEvent::Cancel => {
                        state.editing_tag = None;
                    }
                    TagInputEvent::Editing => {}
                }
            } else {
                // Static chip display
                let text_layout = ui.painter().layout_no_wrap(
                    tag_name.clone(),
                    egui::FontId::proportional(12.0),
                    t.ink_strong,
                );
                let close_btn_w = 16.0;
                let chip_w = (text_layout.size().x + close_btn_w + 16.0).max(48.0);
                let (chip_rect, chip_resp) =
                    ui.allocate_exact_size(vec2(chip_w, CONTROL_HEIGHT_FORM), Sense::click());

                let painter = ui.painter();
                let chip_bg = if chip_resp.hovered() {
                    t.sunken
                } else {
                    t.panel
                };
                painter.rect_filled(chip_rect, CornerRadius::same(3), chip_bg);
                painter.rect_stroke(
                    chip_rect,
                    CornerRadius::same(3),
                    t.hairline(),
                    StrokeKind::Inside,
                );

                let label_pos = Pos2::new(chip_rect.min.x + 8.0, chip_rect.center().y);
                painter.galley(
                    Pos2::new(label_pos.x, label_pos.y - text_layout.size().y / 2.0),
                    text_layout,
                    t.ink_strong,
                );

                // 'x' close button
                let close_rect = Rect::from_center_size(
                    Pos2::new(chip_rect.max.x - 10.0, chip_rect.center().y),
                    vec2(close_btn_w, close_btn_w),
                );
                let close_resp = ui
                    .interact(
                        close_rect,
                        ui.id().with(("chip_close", tag_name)),
                        Sense::click(),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);

                let close_color = if close_resp.hovered() {
                    t.ink_strong
                } else {
                    t.muted
                };
                paint_icon(painter, close_rect.center(), Icon::Close, 11.0, close_color);

                if close_resp.clicked() {
                    remove_tag = Some(tag_name.clone());
                } else if chip_resp.double_clicked() {
                    state.editing_tag = Some(tag_name.clone());
                    state.editing_text = tag_name.clone();
                    state.focus_pending = true;
                } else {
                    chip_resp.on_hover_text("Double-click to rename tag");
                }
            }
        }

        // 3. Inline new tag input
        if state.inline_new_active {
            let (slot, _) = ui.allocate_exact_size(vec2(90.0, CONTROL_HEIGHT_FORM), Sense::hover());
            let focus_now = state.focus_pending;
            state.focus_pending = false;

            match tag_text_input(
                ui.ctx(),
                ui.id().with("tag_inline_new_input"),
                slot,
                &mut state.inline_new_text,
                "Tag name...",
                focus_now,
            ) {
                TagInputEvent::Commit => {
                    if let Some(norm) = normalize_tag(&state.inline_new_text) {
                        create_tag = Some(norm);
                    }
                    state.inline_new_active = false;
                }
                TagInputEvent::Cancel => {
                    state.inline_new_active = false;
                }
                TagInputEvent::Editing => {}
            }
        }

        // 4. Square "+" button
        let plus_btn = icon_button(
            ui,
            Icon::Add,
            ICON_BUTTON_SIZE_FORM,
            IconButtonStyle::bordered(),
        )
        .on_hover_text("Add tag");

        if plus_btn.clicked() {
            state.inline_new_active = true;
            state.inline_new_text.clear();
            state.focus_pending = true;
        }

        // 5. Square "⌄" button for recall dropdown
        let unused_recall: Vec<String> = recall
            .iter()
            .filter(|r| !tags.iter().any(|t| t.eq_ignore_ascii_case(r)))
            .cloned()
            .collect();
        let has_unused = !unused_recall.is_empty();

        let recall_btn = icon_button(
            ui,
            Icon::ChevronDown,
            ICON_BUTTON_SIZE_FORM,
            if has_unused {
                IconButtonStyle::bordered()
            } else {
                IconButtonStyle::ghost()
            },
        )
        .on_hover_text("Choose from existing tags");

        // 6. Search picker dropdown (5 rows then scroll)
        if has_unused {
            let picker_items: Vec<PickerItem<String>> = unused_recall
                .iter()
                .map(|t| PickerItem::new(t.clone(), t.clone()))
                .collect();

            let mut cfg = SearchPickerConfig::new("Search tag...");
            cfg.max_rows = 5;

            if let Some(PickerChoice::Item(chosen)) = search_picker(
                &recall_btn,
                ui.id().with("tag_recall_popup"),
                &picker_items,
                &mut state.recall_picker,
                &cfg,
            ) {
                create_tag = Some(chosen);
            }
        }
    });

    // While inline editing or new tag input is active, lock input under blocker
    if state.inline_new_active || state.editing_tag.is_some() {
        input_blocker(ui.ctx());
    }

    if let Some(tag) = remove_tag {
        events.push(TagEvent::Remove(tag));
    }
    if let Some(tag) = create_tag {
        events.push(TagEvent::Add(tag));
    }
    if let Some((from, to)) = rename_tag {
        events.push(TagEvent::Rename { from, to });
    }

    ui.ctx().data_mut(|d| d.insert_temp(state_id, state));
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Context, Pos2, RawInput, Rect};
    use lc_canvas::theme::{self, ThemeChoice};

    #[test]
    fn test_normalize_tag() {
        assert_eq!(normalize_tag(""), None);
        assert_eq!(normalize_tag("   "), None);
        assert_eq!(normalize_tag("  Player  "), Some("Player".to_string()));
    }

    #[test]
    fn test_normalize_tags_dedup_and_casing() {
        let input = ["  Enemy ", "enemy", "PLAYER", "player", " ", "Boss"];
        let normalized = normalize_tags(&input);
        assert_eq!(
            normalized,
            vec![
                "Enemy".to_string(),
                "PLAYER".to_string(),
                "Boss".to_string()
            ]
        );
    }

    #[test]
    fn test_tag_row_headless_render() {
        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        let tags = vec!["Items".to_string(), "Weapons".to_string()];
        let recall = vec![
            "Items".to_string(),
            "Weapons".to_string(),
            "Armor".to_string(),
        ];

        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1024.0, 768.0))),
            ..Default::default()
        };

        let mut output = ctx.run_ui(raw_input, |ui| {
            let events = tag_row(ui, &tags, &recall);
            assert!(events.is_empty());
        });

        output.textures_delta.clear();
        assert!(
            !output.shapes.is_empty(),
            "tag_row should produce render shapes for chips and buttons"
        );
    }
}
