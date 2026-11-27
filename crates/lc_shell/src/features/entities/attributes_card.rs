//! LDtk-parity Entity Identifier bar and unified toggleable `ENTITY SETTINGS` section.

use egui::{vec2, Align, CornerRadius, DragValue, Layout, RichText, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::{EntityDef, TilesetRect};
use lc_core::marking::known_tags;
use lc_feature_entities::UpdateEntityDefCommand;
use lc_ui_api::AssetService;

use super::state::{EntityTileSlot, EntityViewState};
use super::visuals_card::{render_limits_row, render_visuals_and_limits_rows};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::swatch::TileIconData;
use crate::shared::ui::tag_row::{tag_row, TagEvent};
use crate::shared::ui::text_field::{
    text_field, validate_identifier, FieldOutline, FieldResponse, FieldStyle,
};
use crate::shared::ui::tile_bounds::TileRect;
use crate::shared::ui::tile_picker::TilePicker;
use crate::shared::ui::workspace_metrics::{card_label, CONTROL_HEIGHT_FORM, FORM_LABEL_COL_W};

#[derive(Clone, Debug, Default)]
struct BufferedEditScratch {
    text: String,
    last_external: String,
    focused: bool,
}

fn load_buffered_scratch(ui: &Ui, buf_id: egui::Id, external_text: &str) -> BufferedEditScratch {
    let mut scratch = ui
        .ctx()
        .data_mut(|d| d.get_temp::<BufferedEditScratch>(buf_id))
        .unwrap_or_else(|| BufferedEditScratch {
            text: external_text.to_string(),
            last_external: external_text.to_string(),
            focused: false,
        });
    if !scratch.focused && scratch.last_external != external_text {
        scratch.text = external_text.to_string();
        scratch.last_external = external_text.to_string();
    }
    scratch
}

fn save_buffered_scratch(
    ui: &Ui,
    buf_id: egui::Id,
    external_text: &str,
    mut scratch: BufferedEditScratch,
    has_focus: bool,
    cancelled: bool,
) -> String {
    if cancelled {
        scratch.text = external_text.to_string();
        scratch.last_external = external_text.to_string();
        scratch.focused = false;
    } else {
        scratch.focused = has_focus;
        if !has_focus {
            scratch.last_external = external_text.to_string();
        }
    }
    let out = scratch.text.clone();
    ui.ctx().data_mut(|d| d.insert_temp(buf_id, scratch));
    out
}

/// Renders [`text_field`] with focus-buffered scratch storage so intermediate typing
/// (empty identifier, trailing spaces, commas) is never wiped between frames.
pub(crate) fn buffered_text_field(
    ui: &mut Ui,
    id_salt: impl egui::AsId,
    external_text: &str,
    style: FieldStyle<'_>,
) -> (FieldResponse, String) {
    let buf_id = ui.make_persistent_id(id_salt);
    let mut scratch = load_buffered_scratch(ui, buf_id, external_text);
    let res = text_field(ui, &mut scratch.text, style);
    let has_focus = res.response.has_focus();
    let cancelled = res.cancelled;
    let out = save_buffered_scratch(ui, buf_id, external_text, scratch, has_focus, cancelled);
    (res, out)
}

/// Renders a single-line [`egui::TextEdit`] with focus-buffered scratch storage.
pub(crate) fn buffered_singleline(
    ui: &mut Ui,
    id_salt: impl egui::AsId,
    external_text: &str,
    width: f32,
    hint: &str,
) -> (egui::Response, String) {
    let buf_id = ui.make_persistent_id(id_salt);
    let mut scratch = load_buffered_scratch(ui, buf_id, external_text);
    let resp = ui.add(
        egui::TextEdit::singleline(&mut scratch.text)
            .id(buf_id.with("edit"))
            .desired_width(width)
            .hint_text(hint),
    );
    let cancelled = resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape));
    let out = save_buffered_scratch(
        ui,
        buf_id,
        external_text,
        scratch,
        resp.has_focus(),
        cancelled,
    );
    (resp, out)
}

/// Renders a multi-line [`egui::TextEdit`] with focus-buffered scratch storage.
pub(crate) fn buffered_multiline(
    ui: &mut Ui,
    id_salt: impl egui::AsId,
    external_text: &str,
    width: f32,
) -> (egui::Response, String) {
    let buf_id = ui.make_persistent_id(id_salt);
    let mut scratch = load_buffered_scratch(ui, buf_id, external_text);
    let resp = ui.add(
        egui::TextEdit::multiline(&mut scratch.text)
            .id(buf_id.with("edit"))
            .desired_rows(3)
            .desired_width(width),
    );
    let cancelled = resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape));
    let out = save_buffered_scratch(
        ui,
        buf_id,
        external_text,
        scratch,
        resp.has_focus(),
        cancelled,
    );
    (resp, out)
}

/// Shared aligned attribute row with `FORM_LABEL_COL_W` label column.
///
/// Constrains child controls to the remaining row width and wraps onto subsequent
/// lines so child content never widens the parent card.
pub(crate) fn attr_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    let total_w = ui.available_width().max(FORM_LABEL_COL_W + 80.0);
    let ctrl_w = (total_w - FORM_LABEL_COL_W - 6.0).max(80.0);
    ui.allocate_ui_with_layout(
        vec2(total_w, CONTROL_HEIGHT_FORM),
        Layout::left_to_right(Align::TOP),
        |ui| {
            ui.set_max_width(total_w);
            ui.spacing_mut().item_spacing.x = 6.0;
            card_label(ui, label, t);
            let ctrl_origin = ui.cursor().min;
            let mut right = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(egui::Rect::from_min_size(
                        ctrl_origin,
                        vec2(ctrl_w, CONTROL_HEIGHT_FORM),
                    ))
                    .layout(Layout::left_to_right(Align::Center).with_main_wrap(true)),
            );
            let ret = control(&mut right);
            let used_h = (right.min_rect().max.y - ctrl_origin.y).max(CONTROL_HEIGHT_FORM);
            ui.allocate_exact_size(vec2(ctrl_w, used_h), Sense::hover());
            ret
        },
    )
    .inner
}

/// Shared attribute row whose controls wrap cleanly onto subsequent lines below the
/// first control when exceeding the container's fixed width (never widening the card).
pub(crate) fn attr_wrap_row<R>(
    ui: &mut Ui,
    t: &Tokens,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    attr_row(ui, t, label, control)
}

/// Top Entity Identifier + `Show identifier` bar (remains visible when `ENTITY SETTINGS` toggles).
pub fn render_entity_identifier_bar(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, def: &EntityDef) {
    let mut draft = def.clone();
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        card_label(ui, "Entity identifier", t);

        let other_names: Vec<&str> = app
            .project
            .defs
            .entities
            .iter()
            .filter(|other| other.id != draft.id)
            .map(|other| other.name.as_str())
            .collect();
        let id_salt = ("entity_name_bar", def.id);
        let current_scratch = ui
            .ctx()
            .data_mut(|d| d.get_temp::<BufferedEditScratch>(ui.make_persistent_id(id_salt)))
            .map(|s| s.text)
            .unwrap_or_else(|| draft.name.clone());
        let id_err_msg = validate_identifier(&current_scratch, &other_names, Some(&def.name))
            .err()
            .map(|e| e.to_string());
        let mut style = FieldStyle::default()
            .with_hint("Entity identifier")
            .with_outline_preset(FieldOutline::Hairline);
        if let Some(ref msg) = id_err_msg {
            style = style.with_error(msg);
        }
        let field_w = (ui.available_width() - 140.0).clamp(140.0, 420.0);
        ui.allocate_ui_with_layout(
            vec2(field_w, CONTROL_HEIGHT_FORM),
            Layout::left_to_right(Align::Center),
            |ui| {
                let (res, text) = buffered_text_field(ui, id_salt, &draft.name, style);
                if (res.committed || res.changed)
                    && validate_identifier(&text, &other_names, Some(&def.name)).is_ok()
                {
                    let trimmed = text.trim();
                    if trimmed != draft.name {
                        draft.name = trimmed.to_string();
                        changed = true;
                    }
                }
            },
        );

        if styled_checkbox(ui, t, &mut draft.show_name, "Show identifier").changed() {
            changed = true;
        }
    });

    if changed && draft != *def {
        app.execute_command(Box::new(UpdateEntityDefCommand::with_stroke(
            draft,
            def.id.0.wrapping_add(0x454E_5400),
        )));
    }
}

/// Renders all EntityDef attributes inside a single unified toggleable `ENTITY SETTINGS` section.
pub fn render_entity_attributes(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
) {
    let mut draft = def.clone();
    let mut changed = false;

    let mut collapse_settings = state.collapse_settings;
    SectionContainer::new("ENTITY SETTINGS")
        .use_toggle_collapse(true)
        .show(
            ui,
            t,
            &mut collapse_settings,
            |_| {},
            |ui| {
                // Misc options
                attr_row(ui, t, "Misc options", |ui| {
                    if styled_checkbox(
                        ui,
                        t,
                        &mut draft.export_to_toc,
                        "Add to JSON Table-of-content",
                    )
                    .changed()
                    {
                        changed = true;
                    }
                    ui.add_space(12.0);
                    if styled_checkbox(
                        ui,
                        t,
                        &mut draft.allow_out_of_bounds,
                        "Can be out of level bounds",
                    )
                    .changed()
                    {
                        changed = true;
                    }
                });
                ui.add_space(6.0);

                // Documentation
                attr_row(ui, t, "Documentation", |ui| {
                    let external = draft.doc.clone().unwrap_or_default();
                    let style = FieldStyle::default()
                        .with_hint("Optional designer note")
                        .with_outline_preset(FieldOutline::Hairline);
                    let (res, text) =
                        buffered_text_field(ui, ("entity_doc", def.id), &external, style);
                    if res.changed || res.committed {
                        let next_doc = if res.committed {
                            let trimmed = text.trim().to_string();
                            (!trimmed.is_empty()).then_some(trimmed)
                        } else {
                            (!text.trim().is_empty()).then_some(text)
                        };
                        if next_doc != draft.doc {
                            draft.doc = next_doc;
                            changed = true;
                        }
                    }
                });
                ui.add_space(6.0);

                // Tags
                attr_row(ui, t, "Tags", |ui| {
                    ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                    let mut extra_tags: Vec<String> = app
                        .project
                        .defs
                        .entities
                        .iter()
                        .flat_map(|e| e.tags.clone())
                        .collect();
                    for tag in &app.tag_registry {
                        extra_tags.push(tag.name.clone());
                    }
                    let all_tags = known_tags(&app.project.defs.tilesets, &extra_tags);
                    for ev in tag_row(ui, &draft.tags, &all_tags) {
                        match ev {
                            TagEvent::Add(tag) => {
                                let clean = tag.trim().to_string();
                                if !clean.is_empty()
                                    && !draft.tags.iter().any(|t| t.eq_ignore_ascii_case(&clean))
                                {
                                    draft.tags.push(clean);
                                    changed = true;
                                }
                            }
                            TagEvent::Remove(tag) => {
                                let prev_len = draft.tags.len();
                                draft.tags.retain(|t| t != &tag);
                                if draft.tags.len() != prev_len {
                                    changed = true;
                                }
                            }
                            TagEvent::Rename { from, to } => {
                                for tag in &mut draft.tags {
                                    if *tag == from {
                                        *tag = to.clone();
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                });
                ui.add_space(6.0);

                // Size & Resizing
                attr_row(ui, t, "Size", |ui| {
                    let mut w = draft.size_px.x;
                    if ui
                        .add(DragValue::new(&mut w).range(1..=2048).suffix(" px"))
                        .changed()
                    {
                        draft.size_px.x = w.max(1);
                        changed = true;
                    }
                    ui.label(RichText::new("x").color(t.muted));
                    let mut h = draft.size_px.y;
                    if ui
                        .add(DragValue::new(&mut h).range(1..=2048).suffix(" px"))
                        .changed()
                    {
                        draft.size_px.y = h.max(1);
                        changed = true;
                    }
                });
                ui.add_space(4.0);
                attr_row(ui, t, "", |ui| {
                    ui.label(RichText::new("Resizable:").color(t.muted));
                    if styled_checkbox(ui, t, &mut draft.resizable_x, "Width").changed() {
                        changed = true;
                    }
                    ui.add_space(8.0);
                    if styled_checkbox(ui, t, &mut draft.resizable_y, "Height").changed() {
                        changed = true;
                    }
                    ui.add_space(8.0);
                    if draft.resizable_x && draft.resizable_y {
                        if styled_checkbox(ui, t, &mut draft.keep_aspect_ratio, "Keep aspect ratio")
                            .changed()
                        {
                            changed = true;
                        }
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Label::new(RichText::new("Keep aspect ratio").color(t.faint)),
                        );
                    }
                });
                ui.add_space(6.0);

                // Editor visual, Opacities/Hollow, 9-Slice, Alt UI icon
                render_visuals_and_limits_rows(app, ui, t, state, &mut draft, &mut changed);
                ui.add_space(6.0);

                // Pivot point (3x3 interactive grid + X/Y)
                attr_row(ui, t, "Pivot point", |ui| {
                    if render_pivot_grid_3x3(ui, t, &mut draft.pivot) {
                        changed = true;
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new("X").color(t.muted));
                    if ui
                        .add(
                            DragValue::new(&mut draft.pivot.x)
                                .speed(0.05)
                                .range(0.0..=1.0),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new("Y").color(t.muted));
                    if ui
                        .add(
                            DragValue::new(&mut draft.pivot.y)
                                .speed(0.05)
                                .range(0.0..=1.0),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                });
                ui.add_space(6.0);

                // Max count + limit scope + limit behavior
                render_limits_row(ui, t, &mut draft, &mut changed);
            },
        );
    state.collapse_settings = collapse_settings;

    if changed && draft != *def {
        app.execute_command(Box::new(UpdateEntityDefCommand::with_stroke(
            draft,
            def.id.0.wrapping_add(0x454E_5400),
        )));
    }
}

fn render_pivot_grid_3x3(ui: &mut Ui, t: &Tokens, pivot: &mut glam::Vec2) -> bool {
    let mut changed = false;
    let pts = [0.0_f32, 0.5_f32, 1.0_f32];
    let cell = 12.0;
    let gap = 2.0;
    let total = cell * 3.0 + gap * 2.0;

    let (grid_rect, _) = ui.allocate_exact_size(vec2(total, total), Sense::hover());
    for (row_idx, &py) in pts.iter().enumerate() {
        for (col_idx, &px) in pts.iter().enumerate() {
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(
                    grid_rect.min.x + col_idx as f32 * (cell + gap),
                    grid_rect.min.y + row_idx as f32 * (cell + gap),
                ),
                vec2(cell, cell),
            );
            let active = (pivot.x - px).abs() < 0.01 && (pivot.y - py).abs() < 0.01;
            let resp = ui.interact(
                cell_rect,
                ui.id().with(("pivot_cell", row_idx, col_idx)),
                Sense::click(),
            );
            let fill = if active {
                t.accent
            } else if resp.hovered() {
                t.line_strong
            } else {
                t.sunken
            };
            ui.painter().rect(
                cell_rect,
                CornerRadius::ZERO,
                fill,
                Stroke::new(1.0, t.line),
                StrokeKind::Inside,
            );
            if resp.clicked() {
                pivot.x = px;
                pivot.y = py;
                changed = true;
            }
        }
    }
    changed
}

/// Resolves an optional [`TilesetRect`] into [`TileIconData`] for [`tile_icon_button`].
pub fn resolve_tile_icon(
    app: &ShellApp,
    ui: &Ui,
    rect_opt: Option<TilesetRect>,
) -> Option<TileIconData> {
    let tr = rect_opt?;
    let ts = app.project.defs.tileset(tr.tileset_id)?;
    let tex = app.assets.load_texture(ui.ctx(), &ts.path).ok()?;
    let sz = tex.size_vec2();
    if sz.x <= 0.0 || sz.y <= 0.0 {
        return None;
    }
    let uv = egui::Rect::from_min_max(
        egui::pos2(tr.x as f32 / sz.x, tr.y as f32 / sz.y),
        egui::pos2((tr.x + tr.w) as f32 / sz.x, (tr.y + tr.h) as f32 / sz.y),
    );
    Some(TileIconData::new(tex.id(), uv))
}

/// Opens the shared [`TilePicker`] popup targeting `slot`.
pub fn open_tile_picker(
    app: &ShellApp,
    state: &mut EntityViewState,
    entity_id: lc_core::id::EntityDefId,
    slot: EntityTileSlot,
    current: Option<TilesetRect>,
) {
    let Some(first_ts) = current
        .and_then(|r| app.project.defs.tileset(r.tileset_id))
        .or_else(|| app.project.defs.tilesets.first())
    else {
        state.error_message = Some("Create a tileset in the Tilesets Workspace first.".to_string());
        return;
    };

    let selected_rect = current.map(|r| {
        let tw = first_ts.tile_size.x.max(1);
        let th = first_ts.tile_size.y.max(1);
        let gx = r.x.saturating_sub(first_ts.padding) / (tw + first_ts.spacing).max(1);
        let gy = r.y.saturating_sub(first_ts.padding) / (th + first_ts.spacing).max(1);
        let gw = (r.w / tw).max(1);
        let gh = (r.h / th).max(1);
        TileRect::from_cells((gx, gy), (gx + gw - 1, gy + gh - 1))
    });

    let mut picker = TilePicker::new();
    picker.open(None, Some(first_ts.id), selected_rect);
    state.tile_picker = Some((entity_id, slot, picker));
}
