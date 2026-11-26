//! Visuals, Tile Rendering, 9-Slice, Alt UI Icon, and Instance Limits rows inside `ENTITY SETTINGS`.

use egui::{Color32, ComboBox, DragValue, RichText, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::defs::{
    EntityDef, EntityLimitBehavior, EntityLimitScope, EntityRenderMode, TileRenderMode, TilesetRect,
};

use super::attributes_card::{attr_row, open_tile_picker, resolve_tile_icon};
use super::state::{EntityTileSlot, EntityViewState};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::shared::ui::swatch::{color_swatch, tile_icon_button};
use crate::shared::ui::tileset_select::{tileset_select, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::ICON_BUTTON_SIZE_FORM;

/// Renders the Editor Visual, Opacities/Hollow, Alt UI Icon, and 9-Slice rows.
pub fn render_visuals_and_limits_rows(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    draft: &mut EntityDef,
    changed: &mut bool,
) {
    // Editor visual row: color + Shapes/Tilesets dropdown + display tile picker + tile render mode
    attr_row(ui, t, "Editor visual", |ui| {
        let mut col = Color32::from_rgba_unmultiplied(
            draft.color[0],
            draft.color[1],
            draft.color[2],
            draft.color[3],
        );
        if color_swatch(ui, ("entity_color", draft.id), &mut col).changed() {
            draft.color = [col.r(), col.g(), col.b(), 255];
            *changed = true;
        }
        ui.add_space(6.0);

        let active_ts_id = draft
            .tileset_id
            .or_else(|| draft.tile_rect.map(|r| r.tileset_id));
        let visual_label = if draft.render_mode == EntityRenderMode::Tile {
            active_ts_id
                .and_then(|id| app.project.defs.tileset(id))
                .map(|ts| ts.name.clone())
                .unwrap_or_else(|| "Tile (pick tileset)".to_string())
        } else {
            format!("{:?}", draft.render_mode)
        };

        ComboBox::from_id_salt(("entity_editor_visual", draft.id))
            .selected_text(visual_label)
            .show_ui(ui, |ui| {
                ui.label(RichText::new("SHAPES").size(10.0).strong().color(t.muted));
                for mode in [
                    EntityRenderMode::Rectangle,
                    EntityRenderMode::Ellipse,
                    EntityRenderMode::Cross,
                ] {
                    let selected = draft.render_mode == mode;
                    if ui.selectable_label(selected, format!("{mode:?}")).clicked() {
                        if draft.render_mode == EntityRenderMode::Tile {
                            draft.tile_opacity = 1.0;
                            draft.fill_opacity = 1.0;
                            draft.line_opacity = 1.0;
                        }
                        draft.render_mode = mode;
                        draft.tileset_id = None;
                        draft.tile_rect = None;
                        draft.tile = None;
                        *changed = true;
                    }
                }
                ui.separator();
                ui.label(RichText::new("TILESETS").size(10.0).strong().color(t.muted));
                if app.project.defs.tilesets.is_empty() {
                    ui.label(
                        RichText::new("(No tilesets in project)")
                            .size(11.0)
                            .color(t.faint),
                    );
                } else {
                    for ts in &app.project.defs.tilesets {
                        let selected = draft.render_mode == EntityRenderMode::Tile
                            && active_ts_id == Some(ts.id);
                        if ui.selectable_label(selected, &ts.name).clicked() {
                            if draft.render_mode != EntityRenderMode::Tile {
                                draft.tile_opacity = 1.0;
                                draft.fill_opacity = 0.08;
                                draft.line_opacity = 0.0;
                            }
                            draft.render_mode = EntityRenderMode::Tile;
                            draft.tileset_id = Some(ts.id);
                            if draft.tile_rect.is_some_and(|r| r.tileset_id != ts.id) {
                                draft.tile_rect = None;
                                draft.tile = None;
                            }
                            *changed = true;
                        }
                    }
                }
            });

        if draft.render_mode == EntityRenderMode::Tile || draft.tile_rect.is_some() {
            ui.add_space(6.0);
            let tile_icon = resolve_tile_icon(app, ui, draft.tile_rect);
            if tile_icon_button(ui, tile_icon)
                .on_hover_text("Pick display tile or multi-tile region")
                .clicked()
            {
                let seed_rect = draft
                    .tile_rect
                    .or_else(|| active_ts_id.map(|ts_id| TilesetRect::new(ts_id, 0, 0, 16, 16)));
                open_tile_picker(app, state, draft.id, EntityTileSlot::MainTile, seed_rect);
            }
            if draft.tile_rect.is_some()
                && icon_button(
                    ui,
                    Icon::Close,
                    ICON_BUTTON_SIZE_FORM,
                    IconButtonStyle::bordered(),
                )
                .on_hover_text("Clear display tile")
                .clicked()
            {
                draft.tile_rect = None;
                draft.tile = None;
                *changed = true;
            }
            ui.add_space(6.0);
            ComboBox::from_id_salt(("entity_tile_render_mode", draft.id))
                .selected_text(format!("{:?}", draft.tile_render_mode))
                .show_ui(ui, |ui| {
                    for mode in [
                        TileRenderMode::FitInside,
                        TileRenderMode::Cover,
                        TileRenderMode::Stretch,
                        TileRenderMode::Repeat,
                        TileRenderMode::FullSizeCropped,
                        TileRenderMode::FullSizeUncropped,
                        TileRenderMode::NineSlice,
                    ] {
                        if ui
                            .selectable_value(
                                &mut draft.tile_render_mode,
                                mode,
                                format!("{mode:?}"),
                            )
                            .clicked()
                        {
                            *changed = true;
                        }
                    }
                });
        }
    });
    ui.add_space(4.0);

    // Tile / Fill / Line opacities (%) + Hollow checkbox
    attr_row(ui, t, "", |ui| {
        ui.label(RichText::new("Tile=").color(t.muted));
        let mut tile_pct = (draft.tile_opacity * 100.0).round() as u32;
        if ui
            .add_enabled(
                draft.render_mode == EntityRenderMode::Tile,
                DragValue::new(&mut tile_pct).range(0..=100).suffix("%"),
            )
            .changed()
        {
            draft.tile_opacity = (tile_pct.min(100) as f32) / 100.0;
            *changed = true;
        }
        ui.add_space(6.0);
        ui.label(RichText::new("Fill=").color(t.muted));
        let mut fill_pct = (draft.fill_opacity * 100.0).round() as u32;
        if ui
            .add_enabled(
                !draft.hollow,
                DragValue::new(&mut fill_pct).range(0..=100).suffix("%"),
            )
            .changed()
        {
            draft.fill_opacity = (fill_pct.min(100) as f32) / 100.0;
            *changed = true;
        }
        ui.add_space(6.0);
        ui.label(RichText::new("Line=").color(t.muted));
        let mut line_pct = (draft.line_opacity * 100.0).round() as u32;
        if ui
            .add(DragValue::new(&mut line_pct).range(0..=100).suffix("%"))
            .changed()
        {
            draft.line_opacity = (line_pct.min(100) as f32) / 100.0;
            *changed = true;
        }
        ui.add_space(8.0);
        if styled_checkbox(ui, t, &mut draft.hollow, "Hollow").changed() {
            *changed = true;
        }
    });

    if draft.render_mode == EntityRenderMode::Tile
        && draft.tile_render_mode == TileRenderMode::NineSlice
    {
        ui.add_space(6.0);
        attr_row(ui, t, "9-Slice (px)", |ui| {
            for (idx, lbl) in ["Top", "Right", "Bottom", "Left"].iter().enumerate() {
                ui.label(RichText::new(*lbl).color(t.muted));
                if ui
                    .add(DragValue::new(&mut draft.nine_slice_borders[idx]).range(0..=512))
                    .changed()
                {
                    *changed = true;
                }
                ui.add_space(6.0);
            }
        });
    }
    ui.add_space(6.0);

    // Alt UI Icon using shared tileset_select dropdown + tile swatch picker
    attr_row(ui, t, "Alt UI icon", |ui| {
        let ts_items: Vec<TilesetSelectItem<lc_core::id::TilesetDefId>> = app
            .project
            .defs
            .tilesets
            .iter()
            .map(|ts| TilesetSelectItem::new(ts.id, ts.name.clone()))
            .collect();
        let mut ui_ts = draft.ui_tile_rect.map(|r| r.tileset_id);
        if tileset_select(ui, &ts_items, &mut ui_ts, true) {
            match ui_ts {
                Some(ts_id) => {
                    let tw = app
                        .project
                        .defs
                        .tileset(ts_id)
                        .map_or(16, |ts| ts.tile_size.x.max(1));
                    let th = app
                        .project
                        .defs
                        .tileset(ts_id)
                        .map_or(16, |ts| ts.tile_size.y.max(1));
                    draft.ui_tile_rect = Some(TilesetRect::new(ts_id, 0, 0, tw, th));
                }
                None => draft.ui_tile_rect = None,
            }
            *changed = true;
        }
        if let Some(rect) = draft.ui_tile_rect {
            ui.add_space(6.0);
            let ui_icon = resolve_tile_icon(app, ui, Some(rect));
            if tile_icon_button(ui, ui_icon)
                .on_hover_text("Pick Alt UI icon tile")
                .clicked()
            {
                open_tile_picker(app, state, draft.id, EntityTileSlot::UiTile, Some(rect));
            }
            if icon_button(
                ui,
                Icon::Close,
                ICON_BUTTON_SIZE_FORM,
                IconButtonStyle::bordered(),
            )
            .on_hover_text("Clear Alt UI icon")
            .clicked()
            {
                draft.ui_tile_rect = None;
                *changed = true;
            }
        }
    });
}

/// Renders the LDtk inline `Max count` + scope + limit behavior row.
pub fn render_limits_row(ui: &mut Ui, t: &Tokens, draft: &mut EntityDef, changed: &mut bool) {
    attr_row(ui, t, "Max count", |ui| {
        if ui
            .add(
                DragValue::new(&mut draft.max_count)
                    .range(0..=9999)
                    .speed(1.0),
            )
            .on_hover_text("0 = Unlimited")
            .changed()
        {
            *changed = true;
        }
        if draft.max_count == 0 {
            ui.label(RichText::new("(Unlimited)").size(11.5).color(t.faint));
        }
        ui.add_space(6.0);
        ComboBox::from_id_salt(("entity_limit_scope", draft.id))
            .selected_text(match draft.limit_scope {
                EntityLimitScope::PerLayer => "per layer",
                EntityLimitScope::PerLevel => "per level",
                EntityLimitScope::PerWorld => "per world",
            })
            .show_ui(ui, |ui| {
                for (scope, lbl) in [
                    (EntityLimitScope::PerLayer, "per layer"),
                    (EntityLimitScope::PerLevel, "per level"),
                    (EntityLimitScope::PerWorld, "per world"),
                ] {
                    if ui
                        .selectable_value(&mut draft.limit_scope, scope, lbl)
                        .clicked()
                    {
                        *changed = true;
                    }
                }
            });
        ui.label(RichText::new(", and").color(t.muted));
        ComboBox::from_id_salt(("entity_limit_behavior", draft.id))
            .selected_text(match draft.limit_behavior {
                EntityLimitBehavior::MoveLastOne => "move last one",
                EntityLimitBehavior::DiscardOldOnes => "discard older ones",
                EntityLimitBehavior::PreventAdding => "prevent adding",
            })
            .show_ui(ui, |ui| {
                for (beh, lbl) in [
                    (EntityLimitBehavior::MoveLastOne, "move last one"),
                    (EntityLimitBehavior::DiscardOldOnes, "discard older ones"),
                    (EntityLimitBehavior::PreventAdding, "prevent adding"),
                ] {
                    if ui
                        .selectable_value(&mut draft.limit_behavior, beh, lbl)
                        .clicked()
                    {
                        *changed = true;
                    }
                }
            });
        ui.label(
            RichText::new("when max is reached.")
                .size(11.5)
                .color(t.muted),
        );
    });
}
