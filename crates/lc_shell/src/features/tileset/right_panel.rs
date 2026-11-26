//! Right panel shell for the Tileset workspace.

use egui::{
    vec2, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Pos2, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, ButtonKind};
use lc_core::defs::TilesetDef;
use lc_core::id::EnumDefId;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_feature_tileset::{compute_tileset_grid, SetMarkingSourceCommand, SetTileCustomDataCommand};
use lc_ui_api::AssetService;

use super::i18n::keys;
use crate::features::tileset::state::{TilesetViewState, CUSTOM_DATA_VALUE_ID};
use crate::shared::i18n::tr;
use crate::shared::ui::tileset_select::{
    is_tileset_select_open, tileset_select_full_width, TilesetSelectItem,
};
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

use lc_core::Command;

/// Paints a tile graphic thumbnail inside the given rect.
pub fn paint_tile_thumbnail(
    painter: &egui::Painter,
    ctx: &egui::Context,
    icon_rect: Rect,
    project: &Project,
    assets: &dyn AssetService,
    tile_ref: &TileRef,
) -> bool {
    if let Some(target_tileset) = project.defs.tileset(tile_ref.tileset_id) {
        if let Ok(texture) = assets.load_texture(ctx, &target_tileset.path) {
            let img_size = texture.size_vec2();
            let tile_w = target_tileset.tile_size.x.max(1);
            let tile_h = target_tileset.tile_size.y.max(1);
            let padding = target_tileset.padding;
            let spacing = target_tileset.spacing;

            let img_w = img_size.x as u32;
            let img_h = img_size.y as u32;
            let eff_w = img_w.saturating_sub(padding * 2);
            let stride_x = tile_w + spacing;
            let stride_y = tile_h + spacing;
            let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(1).max(1);

            let col = tile_ref.tile_id % cols;
            let row = tile_ref.tile_id / cols;

            let px = padding + col * stride_x;
            let py = padding + row * stride_y;

            let mut u0 = px as f32 / img_w as f32;
            let mut v0 = py as f32 / img_h as f32;
            let mut u1 = (px + tile_w) as f32 / img_w as f32;
            let mut v1 = (py + tile_h) as f32 / img_h as f32;

            if tile_ref.flip_x {
                std::mem::swap(&mut u0, &mut u1);
            }
            if tile_ref.flip_y {
                std::mem::swap(&mut v0, &mut v1);
            }

            painter.image(
                texture.id(),
                icon_rect.shrink(1.0),
                Rect::from_min_max(Pos2::new(u0, v0), Pos2::new(u1, v1)),
                Color32::WHITE,
            );
            return true;
        }
    }
    false
}

/// Paints the rectangular "Custom data" header row with #tile_id badge.
fn paint_custom_data_header_row(
    ui: &mut Ui,
    t: &Tokens,
    badge_text: &str,
    is_selected: bool,
) -> egui::Response {
    let custom_data_h = 30.0;
    let (cd_rect, cd_resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), custom_data_h), Sense::click());

    let cd_fill = if is_selected {
        theme::tint(t.accent, 0.18)
    } else if cd_resp.hovered() {
        t.sunken
    } else {
        t.bg
    };

    ui.painter()
        .rect_filled(cd_rect, CornerRadius::ZERO, cd_fill);

    let stroke = if is_selected {
        Stroke::new(1.5, t.accent)
    } else {
        t.hairline()
    };
    ui.painter()
        .rect_stroke(cd_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

    let cd_text_pos = Pos2::new(cd_rect.min.x + 8.0, cd_rect.center().y);
    ui.painter().text(
        cd_text_pos,
        egui::Align2::LEFT_CENTER,
        "Custom data",
        FontId::new(12.0, FontFamily::Proportional),
        if is_selected { t.ink_strong } else { t.ink },
    );

    let badge_w = 32.0;
    let badge_h = 16.0;
    let badge_rect = Rect::from_min_size(
        Pos2::new(
            cd_rect.max.x - badge_w - 4.0,
            cd_rect.center().y - badge_h / 2.0,
        ),
        vec2(badge_w, badge_h),
    );
    ui.painter()
        .rect_filled(badge_rect, CornerRadius::ZERO, t.sunken);
    ui.painter().rect_stroke(
        badge_rect,
        CornerRadius::ZERO,
        t.outline(),
        StrokeKind::Inside,
    );
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        FontId::new(10.0, theme::mono_for(ui.ctx())),
        t.ink_strong,
    );

    cd_resp
}

/// Renders the right panel shell (tool content area).
pub fn render_right_panel(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add_space(8.0);

        // Content for selected mode
        match state.active_mode.as_str() {
            "None" => {
                render_tileset_def_inspector(ui, t, tileset, project, assets, state, commands);
            }
            "Marking" => {
                // Source enum selector moved here from Attributes card (full width).
                Frame::new()
                    .inner_margin(Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new(tr(keys::FIELD_MARKING_ENUM))
                                .font(FontId::new(10.5, theme::mono_bold_for(ui.ctx())))
                                .color(t.muted),
                        );
                        ui.add_space(4.0);
                        ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;

                        let enum_items: Vec<TilesetSelectItem<EnumDefId>> = project
                            .defs
                            .enums
                            .iter()
                            .map(|e| TilesetSelectItem::new(e.id, e.name.clone()))
                            .collect();

                        let prev_enum = tileset.marking.source_enum;
                        let mut chosen_enum = prev_enum;

                        if tileset_select_full_width(ui, &enum_items, &mut chosen_enum, true)
                            && chosen_enum != prev_enum
                        {
                            commands.push(Box::new(SetMarkingSourceCommand::new(
                                tileset.id,
                                chosen_enum,
                            )));
                        }
                        state.is_enum_dropdown_open = is_tileset_select_open(ui);
                    });
                ui.add_space(6.0);

                if let Some(source_id) = tileset.marking.source_enum {
                    if let Some(enum_def) = project.defs.enum_def(source_id) {
                        // Ensure an active value is selected if possible
                        if state.selected_marking_value.is_none() {
                            state.selected_marking_value =
                                enum_def.values.first().map(|v| v.id.clone());
                        } else if let Some(ref sel) = state.selected_marking_value {
                            if sel != CUSTOM_DATA_VALUE_ID && enum_def.value(sel).is_none() {
                                state.selected_marking_value =
                                    enum_def.values.first().map(|v| v.id.clone());
                            }
                        }

                        // Determine active tile index for #badge
                        let active_tile_id = state
                            .canvas_states
                            .get(&tileset.id)
                            .and_then(|cs| cs.hovered_tile)
                            .or(state.selected_tile);
                        let badge_text = active_tile_id
                            .map(|id| format!("#{id}"))
                            .unwrap_or_else(|| "#--".to_string());

                        // Top item: "Custom data" with tile ID badge
                        let is_cd_selected =
                            state.selected_marking_value.as_deref() == Some(CUSTOM_DATA_VALUE_ID);
                        let cd_resp =
                            paint_custom_data_header_row(ui, t, &badge_text, is_cd_selected);

                        if cd_resp.clicked() {
                            state.selected_marking_value = Some(CUSTOM_DATA_VALUE_ID.to_string());
                        }

                        ui.add_space(2.0);

                        // Enum value items list
                        ScrollArea::vertical()
                            .id_salt("tileset_marking_values_scroll")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for val in &enum_def.values {
                                    let is_selected =
                                        state.selected_marking_value.as_deref() == Some(&val.id);

                                    let row_height = 28.0;
                                    let (row_rect, row_resp) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), row_height),
                                        Sense::click(),
                                    );

                                    if row_resp.clicked() {
                                        state.selected_marking_value = Some(val.id.clone());
                                    }

                                    let base_r = val.color[0];
                                    let base_g = val.color[1];
                                    let base_b = val.color[2];

                                    // Solid background fill matching enum color (slightly deepened)
                                    let fill_color = Color32::from_rgb(
                                        (base_r as f32 * 0.8) as u8,
                                        (base_g as f32 * 0.8) as u8,
                                        (base_b as f32 * 0.8) as u8,
                                    );
                                    let accent_color = Color32::from_rgb(base_r, base_g, base_b);

                                    ui.painter().rect_filled(
                                        row_rect,
                                        CornerRadius::ZERO,
                                        fill_color,
                                    );

                                    // Left vertical accent stripe (4px wide)
                                    let stripe_rect = Rect::from_min_size(
                                        row_rect.min,
                                        vec2(4.0, row_rect.height()),
                                    );
                                    ui.painter().rect_filled(
                                        stripe_rect,
                                        CornerRadius::ZERO,
                                        accent_color,
                                    );

                                    // Selection outline
                                    if is_selected {
                                        ui.painter().rect_stroke(
                                            row_rect,
                                            CornerRadius::ZERO,
                                            Stroke::new(2.0, t.ink_strong),
                                            StrokeKind::Inside,
                                        );
                                    } else if row_resp.hovered() {
                                        ui.painter().rect_stroke(
                                            row_rect,
                                            CornerRadius::ZERO,
                                            t.outline(),
                                            StrokeKind::Inside,
                                        );
                                    }

                                    // Square icon container
                                    let icon_box_size = 22.0;
                                    let icon_box_rect = Rect::from_min_size(
                                        Pos2::new(
                                            row_rect.min.x + 8.0,
                                            row_rect.center().y - icon_box_size / 2.0,
                                        ),
                                        vec2(icon_box_size, icon_box_size),
                                    );
                                    ui.painter().rect_filled(
                                        icon_box_rect,
                                        CornerRadius::ZERO,
                                        t.sunken,
                                    );
                                    ui.painter().rect_stroke(
                                        icon_box_rect,
                                        CornerRadius::ZERO,
                                        t.hairline(),
                                        StrokeKind::Inside,
                                    );

                                    // Icon: bound tile if present, color chip fallback if not
                                    let mut drew_tile = false;
                                    if let Some(ref tile_ref) = val.tile {
                                        drew_tile = paint_tile_thumbnail(
                                            ui.painter(),
                                            ui.ctx(),
                                            icon_box_rect,
                                            project,
                                            assets,
                                            tile_ref,
                                        );
                                    }
                                    if !drew_tile {
                                        let dot_size = 10.0;
                                        let dot_rect = Rect::from_min_size(
                                            Pos2::new(
                                                icon_box_rect.center().x - dot_size / 2.0,
                                                icon_box_rect.center().y - dot_size / 2.0,
                                            ),
                                            vec2(dot_size, dot_size),
                                        );
                                        ui.painter().rect_filled(
                                            dot_rect,
                                            CornerRadius::ZERO,
                                            accent_color,
                                        );
                                    }

                                    // Label text contrasted against fill_color luminance
                                    let luma = 0.299 * f32::from(fill_color.r())
                                        + 0.587 * f32::from(fill_color.g())
                                        + 0.114 * f32::from(fill_color.b());
                                    let label_color = if luma > 160.0 {
                                        Color32::BLACK
                                    } else {
                                        Color32::WHITE
                                    };
                                    let text_pos =
                                        Pos2::new(icon_box_rect.max.x + 8.0, row_rect.center().y);
                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::LEFT_CENTER,
                                        &val.name,
                                        FontId::new(12.0, FontFamily::Proportional),
                                        label_color,
                                    );

                                    ui.add_space(2.0);
                                }
                            });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new("(Missing source enum)")
                                    .font(FontId::new(12.0, FontFamily::Proportional))
                                    .color(t.muted),
                            );
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Select a source enum above to mark tiles")
                                .font(FontId::new(11.5, FontFamily::Proportional))
                                .color(t.muted),
                        );
                    });
                }
            }
            "Collision" => {
                super::collision_panel::render_collision_panel(
                    ui, t, tileset, project, assets, state, commands,
                );
            }
            "Animation" => {
                super::animation_panel::render_animation_panel(
                    ui, t, tileset, project, assets, state, commands,
                );
            }
            _ => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new("Coming in a later update")
                            .font(FontId::new(12.0, FontFamily::Proportional))
                            .color(t.muted),
                    );
                });
            }
        }
    });
}

fn render_tileset_def_inspector(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
) {
    let active_tile_id = state
        .canvas_states
        .get(&tileset.id)
        .and_then(|cs| cs.hovered_tile)
        .or(state.selected_tile);
    let badge_text = active_tile_id
        .map(|id| format!("#{id}"))
        .unwrap_or_else(|| "#--".to_string());

    let cd_resp = paint_custom_data_header_row(ui, t, &badge_text, false);
    if let Some(tid) = active_tile_id {
        if cd_resp.clicked() {
            let existing = tileset.custom_data.get(&tid).map(|s| s.as_str());
            state.custom_data_dialog.open(tileset.id, tid, existing);
        }
    }

    let grid_cols = assets
        .load_texture(ui.ctx(), &tileset.path)
        .ok()
        .map(|tex| {
            let sz = tex.size();
            compute_tileset_grid(
                [sz[0] as u32, sz[1] as u32],
                tileset.tile_size,
                tileset.spacing,
                tileset.padding,
            )
        });

    ScrollArea::vertical()
        .id_salt("tileset_def_inspector_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 8.0;

                    if let Some(tid) = active_tile_id {
                        render_active_tile_card(
                            ui,
                            t,
                            tileset,
                            project,
                            assets,
                            tid,
                            grid_cols.as_ref().map(|m| m.cols),
                        );
                        render_custom_data_section(ui, t, tileset, state, commands, tid);
                        render_tile_metadata_summary(ui, t, tileset, project, tid);
                    } else {
                        render_tileset_summary_card(
                            ui,
                            t,
                            tileset,
                            grid_cols.as_ref().map(|m| m.total_tiles),
                        );
                    }
                });
        });
}

fn render_active_tile_card(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    assets: &dyn AssetService,
    tile_id: u32,
    cols: Option<u32>,
) {
    Frame::new()
        .fill(t.sunken)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                let thumb_size = 32.0;
                let (thumb_rect, _) =
                    ui.allocate_exact_size(vec2(thumb_size, thumb_size), Sense::hover());
                ui.painter()
                    .rect_filled(thumb_rect, CornerRadius::ZERO, t.bg);
                ui.painter().rect_stroke(
                    thumb_rect,
                    CornerRadius::ZERO,
                    t.hairline(),
                    StrokeKind::Inside,
                );
                paint_tile_thumbnail(
                    ui.painter(),
                    ui.ctx(),
                    thumb_rect,
                    project,
                    assets,
                    &TileRef::new(tileset.id, tile_id),
                );

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(
                        RichText::new(format!("Tile #{tile_id}"))
                            .font(FontId::new(12.0, theme::mono_bold_for(ui.ctx())))
                            .color(t.ink_strong),
                    );
                    let coord_text = match cols {
                        Some(c) if c > 0 => format!(
                            "Grid ({}, {}) • {}x{} px",
                            tile_id % c,
                            tile_id / c,
                            tileset.tile_size.x,
                            tileset.tile_size.y
                        ),
                        _ => format!("{}x{} px", tileset.tile_size.x, tileset.tile_size.y),
                    };
                    ui.label(RichText::new(coord_text).size(10.5).color(t.muted));
                });
            });
        });
}

fn render_custom_data_section(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    state: &mut TilesetViewState,
    commands: &mut Vec<Box<dyn Command>>,
    tile_id: u32,
) {
    let existing = tileset
        .custom_data
        .get(&tile_id)
        .map(|s| s.as_str())
        .filter(|s| !s.trim().is_empty());

    ui.label(
        RichText::new("CUSTOM DATA")
            .font(FontId::new(10.0, theme::mono_bold_for(ui.ctx())))
            .color(t.accent_ink),
    );

    Frame::new()
        .fill(t.sunken)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(data) = existing {
                ui.label(
                    RichText::new(data)
                        .font(FontId::new(11.0, theme::mono_for(ui.ctx())))
                        .color(t.ink),
                );
            } else {
                ui.label(
                    RichText::new("No custom data assigned.")
                        .size(11.0)
                        .color(t.faint),
                );
            }
        });

    ui.horizontal(|ui| {
        if button(ui, "Edit...", ButtonKind::Primary, true).clicked() {
            state.custom_data_dialog.open(tileset.id, tile_id, existing);
        }
        if button(ui, "Clear", ButtonKind::Secondary, existing.is_some()).clicked() {
            commands.push(Box::new(SetTileCustomDataCommand::new(
                tileset.id,
                tile_id,
                None::<String>,
            )));
        }
    });
}

fn render_tile_metadata_summary(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    project: &Project,
    tile_id: u32,
) {
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().min.y,
        t.hairline(),
    );
    ui.add_space(2.0);
    ui.label(
        RichText::new("TILE SUMMARY")
            .font(FontId::new(10.0, theme::mono_bold_for(ui.ctx())))
            .color(t.accent_ink),
    );

    let tags = tileset.marking.tags_at(tile_id);
    let marking_text = if tags.is_empty() {
        "None".to_string()
    } else if let Some(ed) = tileset
        .marking
        .source_enum
        .and_then(|id| project.defs.enum_def(id))
    {
        tags.iter()
            .map(|id| ed.value(id).map_or(*id, |v| v.name.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        tags.join(", ")
    };

    let collision_text = match tileset.collision(tile_id) {
        Some(col) if !col.shapes.is_empty() => format!("{} shape(s)", col.shapes.len()),
        _ => "None".to_string(),
    };

    let anim_text = if let Some(anim) = tileset.animations.get(&tile_id) {
        format!("Anchor ({} frames)", anim.frames.len())
    } else if let Some((&anchor, _)) = tileset
        .animations
        .iter()
        .find(|(_, a)| a.frames.contains(&tile_id))
    {
        format!("Frame of #{anchor}")
    } else {
        "None".to_string()
    };

    for (label, value) in [
        ("Marking", marking_text),
        ("Collision", collision_text),
        ("Animation", anim_text),
    ] {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{label}:")).size(11.0).color(t.muted));
            ui.label(RichText::new(value).size(11.0).color(t.ink_strong));
        });
    }
}

fn render_tileset_summary_card(
    ui: &mut Ui,
    t: &Tokens,
    tileset: &TilesetDef,
    total_tiles: Option<u32>,
) {
    ui.label(
        RichText::new("TILESET OVERVIEW")
            .font(FontId::new(10.0, theme::mono_bold_for(ui.ctx())))
            .color(t.accent_ink),
    );

    Frame::new()
        .fill(t.sunken)
        .stroke(t.hairline())
        .corner_radius(CornerRadius::ZERO)
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;

            let marked_count = tileset
                .marking
                .tags
                .values()
                .flatten()
                .copied()
                .collect::<std::collections::BTreeSet<u32>>()
                .len();
            let custom_count = tileset.custom_data.len();
            let collision_count = tileset.collisions.len();
            let anim_count = tileset.animations.len();

            if let Some(total) = total_tiles {
                summary_stat_row(ui, t, "Total tiles", &total.to_string());
            }
            summary_stat_row(ui, t, "Custom data", &custom_count.to_string());
            summary_stat_row(ui, t, "Marked tiles", &marked_count.to_string());
            summary_stat_row(ui, t, "Collisions", &collision_count.to_string());
            summary_stat_row(ui, t, "Animations", &anim_count.to_string());
        });

    ui.add_space(4.0);
    ui.label(
        RichText::new(
            "Click any tile in the atlas to inspect or edit custom data. Right-click to clear.",
        )
        .size(11.0)
        .color(t.muted),
    );
}

fn summary_stat_row(ui: &mut Ui, t: &Tokens, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(t.muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .font(FontId::new(11.0, theme::mono_bold_for(ui.ctx())))
                    .color(t.ink_strong),
            );
        });
    });
}
