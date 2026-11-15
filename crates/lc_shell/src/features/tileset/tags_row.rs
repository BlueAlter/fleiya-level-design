//! Tags row and Enum for tile marking row for the Tileset attributes card.

use std::collections::BTreeSet;

use egui::{vec2, Align2, CornerRadius, Id, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::id::EnumDefId;
use lc_feature_tileset::{SetMarkingSourceCommand, UpdateTilesetDefCommand};

use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::search_picker::{
    search_picker, PickerChoice, PickerItem, SearchPickerConfig,
};
use crate::shared::ui::tag_input::{tag_text_input, TagInputEvent};

pub const CONTROL_HEIGHT: f32 = 28.0;

/// Renders the Tags row: chips with 'x', inline '+' input, and recall dropdown via search_picker.
pub fn render_tags_row(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;
        ui.label(
            RichText::new("Tags:")
                .font(egui::FontId::proportional(11.5))
                .strong()
                .color(t.ink_strong),
        );

        let mut remove_tag: Option<String> = None;
        let mut rename_tag_action: Option<(String, String)> = None;
        let mut add_tag_action: Option<String> = None;
        let mut recall_btn_resp: Option<egui::Response> = None;

        // Gather all unused project tags for recall dropdown
        let mut all_tags: BTreeSet<String> = BTreeSet::new();
        for ts in &app.project.defs.tilesets {
            for tag in &ts.tags {
                all_tags.insert(tag.clone());
            }
        }
        for en in &app.project.defs.enums {
            for tag in &en.tags {
                all_tags.insert(tag.clone());
            }
        }
        for tag_def in &app.tag_registry {
            all_tags.insert(tag_def.name.clone());
        }
        let unused_tags: Vec<String> = all_tags
            .into_iter()
            .filter(|tag| !tileset.tags.contains(tag))
            .collect();
        let has_unused = !unused_tags.is_empty();

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            ui.spacing_mut().interact_size.y = CONTROL_HEIGHT;

            // Empty state: "(No tag)"
            if tileset.tags.is_empty() && !app.tileset_view_state.tag_inline_new_active {
                let (no_tag_rect, no_tag_resp) =
                    ui.allocate_exact_size(vec2(72.0, CONTROL_HEIGHT), Sense::click());
                let border_stroke = if no_tag_resp.hovered() {
                    Stroke::new(1.0, t.muted)
                } else {
                    Stroke::new(1.0, t.muted.linear_multiply(0.35))
                };
                ui.painter().rect_stroke(
                    no_tag_rect,
                    CornerRadius::same(3),
                    border_stroke,
                    StrokeKind::Inside,
                );
                ui.painter().text(
                    no_tag_rect.center(),
                    Align2::CENTER_CENTER,
                    "(No tag)",
                    egui::FontId::proportional(11.5),
                    t.muted,
                );
                if no_tag_resp.clicked() {
                    app.tileset_view_state.tag_inline_new_active = true;
                    app.tileset_view_state.tag_inline_new_text.clear();
                    app.tileset_view_state.tag_focus_pending = true;
                }
                no_tag_resp.on_hover_text("Click to add tag");
            }

            // Chips
            for tag_name in &tileset.tags {
                if app.tileset_view_state.tag_editing_name.as_deref() == Some(tag_name) {
                    let (slot, _) =
                        ui.allocate_exact_size(vec2(84.0, CONTROL_HEIGHT), Sense::hover());
                    let focus_now = std::mem::take(&mut app.tileset_view_state.tag_focus_pending);
                    match tag_text_input(
                        ui.ctx(),
                        Id::new(("ts_tag_edit_input", tag_name)),
                        slot,
                        &mut app.tileset_view_state.tag_editing_text,
                        "",
                        focus_now,
                    ) {
                        TagInputEvent::Commit => {
                            let new_text =
                                app.tileset_view_state.tag_editing_text.trim().to_string();
                            if !new_text.is_empty() {
                                rename_tag_action = Some((tag_name.clone(), new_text));
                            }
                            app.tileset_view_state.tag_editing_name = None;
                        }
                        TagInputEvent::Cancel => {
                            app.tileset_view_state.tag_editing_name = None;
                        }
                        TagInputEvent::Editing => {}
                    }
                } else {
                    let name_galley = ui.painter().layout_no_wrap(
                        tag_name.clone(),
                        egui::FontId::proportional(12.0),
                        t.ink_strong,
                    );
                    let close_btn_w = 16.0;
                    let chip_w = (name_galley.size().x + close_btn_w + 22.0).max(44.0);
                    let (chip_rect, chip_resp) =
                        ui.allocate_exact_size(vec2(chip_w, CONTROL_HEIGHT), Sense::click());

                    let chip_fill = if chip_resp.hovered() {
                        t.canvas
                    } else {
                        t.sunken
                    };
                    ui.painter()
                        .rect_filled(chip_rect, CornerRadius::same(3), chip_fill);
                    ui.painter().rect_stroke(
                        chip_rect,
                        CornerRadius::same(3),
                        t.hairline(),
                        StrokeKind::Inside,
                    );

                    let text_pos = Pos2::new(chip_rect.min.x + 8.0, chip_rect.center().y);
                    ui.painter().galley(
                        Pos2::new(text_pos.x, text_pos.y - name_galley.size().y / 2.0),
                        name_galley,
                        t.ink_strong,
                    );

                    let close_rect = Rect::from_min_size(
                        Pos2::new(
                            chip_rect.max.x - 20.0,
                            chip_rect.min.y + (CONTROL_HEIGHT - 16.0) / 2.0,
                        ),
                        vec2(16.0, 16.0),
                    );
                    let close_resp = ui
                        .interact(
                            close_rect,
                            Id::new(("ts_tag_chip_close", tag_name)),
                            Sense::click(),
                        )
                        .on_hover_text("Remove tag");
                    let close_color = if close_resp.hovered() {
                        t.ink_strong
                    } else {
                        t.muted
                    };
                    paint_icon(
                        ui.painter(),
                        close_rect.center(),
                        Icon::Close,
                        11.0,
                        close_color,
                    );

                    if close_resp.clicked() {
                        remove_tag = Some(tag_name.clone());
                    } else if chip_resp.double_clicked() {
                        app.tileset_view_state.tag_editing_name = Some(tag_name.clone());
                        app.tileset_view_state.tag_editing_text = tag_name.clone();
                        app.tileset_view_state.tag_focus_pending = true;
                    } else {
                        chip_resp.on_hover_text("Double-click to rename tag");
                    }
                }
            }

            // Inline new tag input
            if app.tileset_view_state.tag_inline_new_active {
                let (slot, _) = ui.allocate_exact_size(vec2(90.0, CONTROL_HEIGHT), Sense::hover());
                let focus_now = std::mem::take(&mut app.tileset_view_state.tag_focus_pending);
                match tag_text_input(
                    ui.ctx(),
                    Id::new("ts_tag_inline_new_input"),
                    slot,
                    &mut app.tileset_view_state.tag_inline_new_text,
                    "Tag name...",
                    focus_now,
                ) {
                    TagInputEvent::Commit => {
                        let trimmed = app.tileset_view_state.tag_inline_new_text.trim();
                        if !trimmed.is_empty() {
                            add_tag_action = Some(trimmed.to_string());
                        }
                        app.tileset_view_state.tag_inline_new_active = false;
                    }
                    TagInputEvent::Cancel => {
                        app.tileset_view_state.tag_inline_new_active = false;
                    }
                    TagInputEvent::Editing => {}
                }
            }

            // Square "+" button
            let plus_btn = ui
                .add_sized(
                    [CONTROL_HEIGHT, CONTROL_HEIGHT],
                    egui::Button::new(Icon::Add.rich(ui.ctx(), 13.0, t.ink_strong))
                        .fill(t.sunken)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::same(3)),
                )
                .on_hover_text("Add tag");
            if plus_btn.clicked() {
                app.tileset_view_state.tag_inline_new_active = true;
                app.tileset_view_state.tag_inline_new_text.clear();
                app.tileset_view_state.tag_focus_pending = true;
            }

            // Square "⌄" recall button
            let recall_icon_color = if has_unused {
                t.ink_strong
            } else {
                t.muted.linear_multiply(0.4)
            };
            let recall_btn = ui
                .add_enabled(
                    has_unused,
                    egui::Button::new(Icon::ChevronDown.rich(ui.ctx(), 13.0, recall_icon_color))
                        .fill(t.sunken)
                        .stroke(t.hairline())
                        .corner_radius(CornerRadius::same(3)),
                )
                .on_hover_text(if has_unused {
                    "Assign existing tag"
                } else {
                    "No other tags available"
                });
            recall_btn_resp = Some(recall_btn);
        });

        // Handle tag actions with undoable UpdateTilesetDefCommand
        if let Some(tag) = remove_tag {
            let mut updated = tileset.clone();
            updated.tags.retain(|t| t != &tag);
            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
        }
        if let Some((old_name, new_name)) = rename_tag_action {
            let mut updated = tileset.clone();
            for t in &mut updated.tags {
                if *t == old_name {
                    *t = new_name.clone();
                }
            }
            app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
        }
        if let Some(new_tag) = add_tag_action {
            if !tileset.tags.contains(&new_tag) {
                let mut updated = tileset.clone();
                updated.tags.push(new_tag);
                app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
            }
        }

        // Tag recall dropdown popup via shared search_picker
        if let Some(ref recall_btn) = recall_btn_resp {
            if has_unused {
                let picker_items: Vec<PickerItem<String>> = unused_tags
                    .iter()
                    .map(|tag| PickerItem::new(tag.clone(), tag.clone()))
                    .collect();
                let mut cfg = SearchPickerConfig::new("Search tags...");
                cfg.max_rows = 5;
                if let Some(PickerChoice::Item(chosen_tag)) = search_picker(
                    recall_btn,
                    Id::new(("ts_tag_recall_popup", tileset.id)),
                    &picker_items,
                    &mut app.tileset_view_state.tag_recall_picker,
                    &cfg,
                ) {
                    if !tileset.tags.contains(&chosen_tag) {
                        let mut updated = tileset.clone();
                        updated.tags.push(chosen_tag);
                        app.execute_command(Box::new(UpdateTilesetDefCommand::new(updated)));
                    }
                }
            }
        }
    });
}

/// Renders the "Enum for tile marking" row: button opening shared search_picker with grouped enums.
pub fn render_marking_enum_row(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Enum for tile marking:")
                .font(egui::FontId::proportional(11.5))
                .strong()
                .color(t.ink_strong),
        );

        let current_enum_id = tileset.marking.source_enum;
        let btn_text = match current_enum_id {
            None => "-- None --".to_string(),
            Some(id) => match app.project.defs.enums.iter().find(|e| e.id == id) {
                Some(e) => e.name.clone(),
                None => "(missing enum)".to_string(),
            },
        };

        let btn = ui.add(
            egui::Button::new(
                RichText::new(btn_text)
                    .font(egui::FontId::proportional(12.0))
                    .color(t.ink_strong),
            )
            .fill(t.sunken)
            .stroke(t.hairline())
            .corner_radius(CornerRadius::same(3)),
        );

        let picker_items: Vec<PickerItem<EnumDefId>> = app
            .project
            .defs
            .enums
            .iter()
            .map(|e| {
                let mut item = PickerItem::new(e.id, e.name.clone());
                if !e.tags.is_empty() {
                    item = item.with_groups(e.tags.clone());
                }
                item
            })
            .collect();

        let mut cfg = SearchPickerConfig::new("Search enums...");
        cfg.none_label = Some("-- None --");
        cfg.current = current_enum_id;
        cfg.group_untagged_label = "Untagged";
        cfg.max_rows = 5;

        if let Some(choice) = search_picker(
            &btn,
            Id::new(("ts_marking_enum_picker", tileset.id)),
            &picker_items,
            &mut app.tileset_view_state.marking_picker,
            &cfg,
        ) {
            match choice {
                PickerChoice::None => {
                    app.execute_command(Box::new(SetMarkingSourceCommand::new(tileset.id, None)));
                }
                PickerChoice::Item(enum_id) => {
                    app.execute_command(Box::new(SetMarkingSourceCommand::new(
                        tileset.id,
                        Some(enum_id),
                    )));
                }
            }
        }
    });
}
