//! Right panel shell for the Tileset workspace.

use egui::{
    vec2, Color32, CornerRadius, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::layer::TileRef;
use lc_core::project::Project;
use lc_feature_tileset::mode_availability;
use lc_ui_api::AssetService;

use crate::features::tileset::state::{TilesetViewState, CUSTOM_DATA_VALUE_ID};

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

/// Renders the mode dropdown selector (None, Marking, Animation) beside the Tile Atlas title.
pub fn render_mode_selector_combo(ui: &mut Ui, tileset: &TilesetDef, state: &mut TilesetViewState) {
    let modes = mode_availability(tileset);
    let current_label = modes
        .iter()
        .find(|m| m.id == state.active_mode)
        .map(|m| m.label)
        .unwrap_or("None");

    ui.spacing_mut().interact_size.y = 20.0;
    let combo_w = 110.0;

    egui::ComboBox::from_id_salt("tileset_mode_selector")
        .selected_text(current_label)
        .width(combo_w)
        .icon(|ui, rect, visuals, is_open| {
            let icon = if is_open {
                Icon::ChevronUp
            } else {
                Icon::ChevronDown
            };
            lc_canvas::icons::paint_icon(
                ui.painter(),
                rect.center(),
                icon,
                12.0,
                visuals.text_color(),
            );
        })
        .show_ui(ui, |ui| {
            for mode in modes {
                if mode.available {
                    if ui
                        .selectable_label(state.active_mode == mode.id, mode.label)
                        .clicked()
                    {
                        state.active_mode = mode.id.to_string();
                    }
                } else {
                    let btn = ui.add_enabled(false, egui::Button::new(mode.label));
                    if !mode.hint.is_empty() {
                        btn.on_disabled_hover_text(mode.hint);
                    }
                }
            }
        });
}

/// Paints the rectangular "Custom data" header row with #tile_id badge.
fn paint_custom_data_header_row(
    ui: &mut Ui,
    badge_text: &str,
    is_selected: bool,
) -> egui::Response {
    let custom_data_h = 30.0;
    let (cd_rect, cd_resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), custom_data_h), Sense::click());

    let cd_fill = if is_selected {
        Color32::from_rgb(34, 40, 48)
    } else if cd_resp.hovered() {
        Color32::from_rgb(38, 44, 52)
    } else {
        Color32::from_rgb(26, 30, 36)
    };

    ui.painter()
        .rect_filled(cd_rect, CornerRadius::ZERO, cd_fill);

    let stroke = if is_selected {
        Stroke::new(2.0, Color32::WHITE)
    } else {
        Stroke::new(1.0, Color32::from_rgb(50, 58, 68))
    };
    ui.painter()
        .rect_stroke(cd_rect, CornerRadius::ZERO, stroke, StrokeKind::Inside);

    let cd_text_pos = Pos2::new(cd_rect.min.x + 8.0, cd_rect.center().y);
    ui.painter().text(
        cd_text_pos,
        egui::Align2::LEFT_CENTER,
        "Custom data",
        egui::FontId::proportional(12.0),
        Color32::from_rgb(150, 170, 190),
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
    ui.painter().rect_filled(
        badge_rect,
        CornerRadius::ZERO,
        Color32::from_rgb(20, 24, 28),
    );
    ui.painter().rect_stroke(
        badge_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgb(70, 78, 88)),
        StrokeKind::Inside,
    );
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        egui::FontId::monospace(10.0),
        Color32::WHITE,
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
                ui.add_space(4.0);
                let active_tile_id = state
                    .canvas_states
                    .get(&tileset.id)
                    .and_then(|cs| cs.hovered_tile)
                    .or(state.selected_tile);
                let badge_text = active_tile_id
                    .map(|id| format!("#{id}"))
                    .unwrap_or_else(|| "#--".to_string());

                let cd_resp = paint_custom_data_header_row(ui, &badge_text, false);

                if let Some(tid) = active_tile_id {
                    if cd_resp.clicked() {
                        let existing = tileset.custom_data.get(&tid).map(|s| s.as_str());
                        state.custom_data_dialog.open(tileset.id, tid, existing);
                    }
                }

                ui.add_space(14.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Click any tile to edit custom data")
                            .size(11.0)
                            .color(t.faint),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Right-click to clear custom data")
                            .size(10.5)
                            .color(t.muted),
                    );
                });
                ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
            }
            "Marking" => {
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
                        let cd_resp = paint_custom_data_header_row(ui, &badge_text, is_cd_selected);

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

                                    // Selection outline (2px white border)
                                    if is_selected {
                                        ui.painter().rect_stroke(
                                            row_rect,
                                            CornerRadius::ZERO,
                                            Stroke::new(2.0, Color32::WHITE),
                                            StrokeKind::Inside,
                                        );
                                    } else if row_resp.hovered() {
                                        ui.painter().rect_stroke(
                                            row_rect,
                                            CornerRadius::ZERO,
                                            Stroke::new(1.0, Color32::from_white_alpha(70)),
                                            StrokeKind::Inside,
                                        );
                                    }

                                    // Dark square icon container
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
                                        Color32::from_rgb(22, 26, 32),
                                    );
                                    ui.painter().rect_stroke(
                                        icon_box_rect,
                                        CornerRadius::ZERO,
                                        Stroke::new(1.0, Color32::from_rgb(45, 52, 60)),
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

                                    // Label text in bold white
                                    let text_pos =
                                        Pos2::new(icon_box_rect.max.x + 8.0, row_rect.center().y);
                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::LEFT_CENTER,
                                        &val.name,
                                        egui::FontId::proportional(12.0),
                                        Color32::WHITE,
                                    );

                                    ui.add_space(2.0);
                                }
                            });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new("(Missing source enum)")
                                    .font(egui::FontId::proportional(12.0))
                                    .color(t.muted),
                            );
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("No source enum selected")
                                .font(egui::FontId::proportional(12.0))
                                .color(t.muted),
                        );
                    });
                }
            }
            "Collision" => {
                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Tile Collision").strong().color(t.ink_strong));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Collision boundaries and tags")
                            .size(11.0)
                            .color(t.faint),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("(Coming soon)")
                            .font(egui::FontId::proportional(11.0))
                            .color(t.muted),
                    );
                });
                ui.allocate_rect(ui.available_rect_before_wrap(), Sense::hover());
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
                            .font(egui::FontId::proportional(12.0))
                            .color(t.muted),
                    );
                });
            }
        }
    });
}
