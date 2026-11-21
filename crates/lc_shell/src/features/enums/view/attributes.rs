//! Enum identifier, attributes card, tileset selection, and tag management.

use egui::{CornerRadius, Frame, Margin, RichText, Ui};

use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;

use super::types::{card_label, enum_sub_tabs, EnumViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::features::enums::file::EnumFileDialogManager;
use crate::features::enums::i18n::keys;
use crate::features::enums::metrics::*;
use crate::shared::i18n::tr;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::tag_row::{tag_row, TagEvent};
use crate::shared::ui::text_field::{text_field, FieldStyle};
use crate::shared::ui::tileset_select::{tileset_select, TilesetSelectItem};

pub fn render_attributes(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut EnumViewState,
    dialog_manager: &std::sync::Arc<EnumFileDialogManager>,
    enum_idx: usize,
) {
    let t = Tokens::get(ui.ctx());
    let current_enum_id = app.enum_draft[enum_idx].id;
    let _is_val_dragging = state.value_drag.is_active();

    // 1. Sub-tabs: ENUM DEF / DOCUMENT
    enum_sub_tabs(
        ui,
        &t,
        &mut state.active_sub_tab,
        &[tr(keys::ENUMS_TAB_DEFINITION), tr(keys::ENUMS_TAB_DOCUMENT)],
    );

    ui.add_space(8.0);

    // 2. Title & Description (truncated with ellipsis, max-width: 100%)
    ui.add(
        egui::Label::new(
            RichText::new(format!("{} (Enum)", app.enum_draft[enum_idx].name))
                .size(20.0)
                .strong()
                .color(t.ink_strong),
        )
        .truncate(),
    );
    ui.label(
        RichText::new(tr(keys::ENUMS_DESCRIPTION))
            .size(11.5)
            .color(t.muted),
    );

    ui.add_space(10.0);

    // 3. Section: ENUM IDENTIFIER & ATTRIBUTES (Frozen top card)
    ui.label(
        RichText::new("ENUM IDENTIFIER & ATTRIBUTES")
            .size(11.0)
            .strong()
            .color(t.accent_ink),
    );
    ui.add_space(4.0);

    Frame::new()
        .fill(t.panel)
        .stroke(t.hairline())
        .inner_margin(Margin::same(12))
        .corner_radius(CornerRadius::same(4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Row 1: Enum Identifier + Export this enum icon
            ui.horizontal(|ui| {
                card_label(ui, "Enum Identifier", &t);
                let export_btn_w = ICON_BUTTON_SIZE_FORM;
                let gap = ui.spacing().item_spacing.x;
                let text_w = (ui.available_width() - export_btn_w - gap).max(60.0);
                let mut name_text = app.enum_draft[enum_idx].name.clone();

                let style = FieldStyle::default()
                    .with_hint("Identifier")
                    .with_outline(true)
                    .with_width(text_w);

                let resp = text_field(ui, &mut name_text, style);

                if resp.changed {
                    if name_text.chars().count() > 48 {
                        name_text = name_text.chars().take(48).collect();
                    }
                    app.enum_draft[enum_idx].name = name_text;
                    app.enum_draft_revision += 1;
                }

                // Top-right "Export this enum" button (1:1 square: 32x32)
                let export_btn = icon_button(
                    ui,
                    Icon::Export,
                    ICON_BUTTON_SIZE_FORM,
                    IconButtonStyle::bordered(),
                )
                .on_hover_text("Export this enum (.fleiyaenum)");

                if export_btn.clicked() {
                    let name_snapshot = app.enum_draft[enum_idx].name.clone();
                    dialog_manager.request_export_one(&name_snapshot, current_enum_id);
                }
            });

            ui.add_space(8.0);

            // Row 2: Tileset dropdown
            ui.horizontal(|ui| {
                ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                card_label(ui, "Tileset:", &t);

                let tileset_items: Vec<TilesetSelectItem<lc_core::id::TilesetDefId>> = app
                    .project
                    .defs
                    .tilesets
                    .iter()
                    .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
                    .collect();

                let prev_ts = app.enum_draft[enum_idx].tileset_id;
                let mut chosen_ts = prev_ts;

                if tileset_select(ui, &tileset_items, &mut chosen_ts, true) && chosen_ts != prev_ts
                {
                    let has_bound_tiles = app.enum_draft[enum_idx]
                        .values
                        .iter()
                        .any(|v| v.tile.is_some());
                    if has_bound_tiles {
                        state.pending_tileset_change = Some(chosen_ts);
                    } else {
                        app.enum_draft[enum_idx].tileset_id = chosen_ts;
                        if let Some(ed) = app.enum_drafts.get_mut(&current_enum_id) {
                            ed.def.tileset_id = chosen_ts;
                            ed.revision += 1;
                        }
                        if let Some(tid) = chosen_ts {
                            app.enum_tileset_map.insert(current_enum_id, tid);
                        } else {
                            app.enum_tileset_map.remove(&current_enum_id);
                        }
                        app.enum_draft_revision += 1;
                    }
                }
            });

            ui.add_space(8.0);

            // Row 3: Tags row
            ui.horizontal(|ui| {
                ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                card_label(ui, "Tags:", &t);

                let assigned_tag_ids: Vec<crate::features::editor::shell_app::TagId> = app
                    .enum_tags_map
                    .get(&current_enum_id)
                    .cloned()
                    .unwrap_or_default();

                let assigned_tags: Vec<String> = assigned_tag_ids
                    .iter()
                    .filter_map(|id| {
                        app.tag_registry
                            .iter()
                            .find(|t| t.id == *id)
                            .map(|t| t.name.clone())
                    })
                    .collect();

                let all_tags: Vec<String> =
                    app.tag_registry.iter().map(|t| t.name.clone()).collect();

                let events = tag_row(ui, &assigned_tags, &all_tags);
                for ev in events {
                    match ev {
                        TagEvent::Add(tag_name) => {
                            let tag_id = app.get_or_create_tag(&tag_name);
                            app.assign_tag_to_enum(current_enum_id, tag_id);
                        }
                        TagEvent::Remove(tag_name) => {
                            if let Some(tag) = app.tag_registry.iter().find(|t| t.name == tag_name)
                            {
                                app.remove_tag_from_enum(current_enum_id, tag.id);
                            }
                        }
                        TagEvent::Rename { from, to } => {
                            if let Some(tag) = app.tag_registry.iter().find(|t| t.name == from) {
                                if let Err(e) = app.rename_tag(tag.id, &to) {
                                    state.toast = Some((e, std::time::Instant::now()));
                                }
                            }
                        }
                    }
                }
            });
        });
}
