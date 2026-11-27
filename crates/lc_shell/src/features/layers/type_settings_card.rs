//! Type-specific Section 3 (`ENTITIES SETTINGS`, `AUTOLAYER SETTINGS`, `TILES SETTINGS`) and advanced operation modals.

use egui::{vec2, Button, ComboBox, CornerRadius, Id, RichText, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::theme::Tokens;
use lc_core::defs::LayerDef;
use lc_core::layer::LayerKind;
use lc_core::marking::known_tags;
use lc_feature_layers::{BakeAutoLayerCommand, MoveEntitiesToLayerCommand, UpdateLayerDefCommand};

use super::attributes_card::attr_row;
use super::state::LayerViewState;
use super::visuals_card::{apply_tag_event, render_autolayer_bindings};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::checkbox::styled_checkbox;
use crate::shared::ui::modal::{modal, ModalButton, ModalButtonKind, ModalSpec};
use crate::shared::ui::section_container::SectionContainer;
use crate::shared::ui::tag_row::tag_row;
use crate::shared::ui::workspace_metrics::{CONTROL_HEIGHT_COMPACT, CONTROL_HEIGHT_FORM};

/// Renders Section 3 for `Entities`, `Auto`, and `Tiles` layers (`<TYPE> SETTINGS`).
pub fn render_type_specific_section(
    app: &mut ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    let mut draft = def.clone();
    let mut changed = false;
    let mut collapse = state.collapse_kind_config;

    let title = match def.kind {
        LayerKind::Auto => "AUTOLAYER SETTINGS",
        LayerKind::Entities => "ENTITIES SETTINGS",
        LayerKind::Tiles => "TILES SETTINGS",
        LayerKind::IntGrid => return,
    };

    SectionContainer::new(title).use_toggle_collapse(true).show(
        ui,
        t,
        &mut collapse,
        |_| {},
        |ui| match draft.kind {
            LayerKind::Auto => {
                render_autolayer_bindings(app, ui, t, state, &mut draft, &mut changed, true);
            }
            LayerKind::Entities => {
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

                attr_row(ui, t, "Hide entity fields", |ui| {
                    if styled_checkbox(
                        ui,
                        t,
                        &mut draft.hide_fields_when_inactive,
                        "Hide entity fields when inactive",
                    )
                    .changed()
                    {
                        changed = true;
                    }
                });
                ui.add_space(5.0);
                attr_row(ui, t, "Required tags", |ui| {
                    ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                    for ev in tag_row(ui, &draft.required_tags, &all_tags) {
                        apply_tag_event(&mut draft.required_tags, ev, &mut changed);
                    }
                });
                ui.add_space(5.0);
                attr_row(ui, t, "Excluded tags", |ui| {
                    ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;
                    for ev in tag_row(ui, &draft.excluded_tags, &all_tags) {
                        apply_tag_event(&mut draft.excluded_tags, ev, &mut changed);
                    }
                });
                ui.add_space(6.0);
                let has_other_entities = app
                    .project
                    .defs
                    .layers
                    .iter()
                    .any(|l| l.id != def.id && l.kind == LayerKind::Entities);
                attr_row(ui, t, "Advanced operations", |ui| {
                    let move_btn = Button::new(
                        RichText::new("MOVE EXISTING ENTITIES TO ANOTHER LAYER")
                            .size(11.0)
                            .strong()
                            .color(if has_other_entities {
                                t.invert_fg
                            } else {
                                t.muted
                            }),
                    )
                    .fill(if has_other_entities {
                        t.accent
                    } else {
                        t.sunken
                    })
                    .corner_radius(CornerRadius::ZERO);
                    if ui
                        .add_enabled(
                            has_other_entities,
                            move_btn.min_size(vec2(
                                ui.available_width().max(220.0),
                                CONTROL_HEIGHT_COMPACT,
                            )),
                        )
                        .clicked()
                    {
                        let first_other = app
                            .project
                            .defs
                            .layers
                            .iter()
                            .find(|l| l.id != def.id && l.kind == LayerKind::Entities)
                            .map(|l| l.id);
                        state.pending_move_entities = Some((def.id, first_other));
                    }
                });
            }
            LayerKind::Tiles => {
                attr_row(ui, t, "Tile pivot point", |ui| {
                    render_tile_pivot_grid(ui, t, &mut draft.tile_pivot, &mut changed);
                });
            }
            LayerKind::IntGrid => {}
        },
    );
    state.collapse_kind_config = collapse;

    if changed && draft != *def {
        let stroke = def.id.0.wrapping_add(0x4C59_5207);
        app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(draft, stroke)));
    }

    render_layer_ops_modals(app, ui, state, def);
}

fn render_tile_pivot_grid(ui: &mut Ui, t: &Tokens, pivot: &mut glam::Vec2, changed: &mut bool) {
    let cell = 20.0;
    let gap = 2.0;
    let total = cell * 3.0 + gap * 2.0;
    let (grid_rect, _) = ui.allocate_exact_size(vec2(total, total), Sense::hover());

    for row in 0..3 {
        for col in 0..3 {
            let px = col as f32 * 0.5;
            let py = row as f32 * 0.5;
            let r = egui::Rect::from_min_size(
                egui::pos2(
                    grid_rect.min.x + col as f32 * (cell + gap),
                    grid_rect.min.y + row as f32 * (cell + gap),
                ),
                vec2(cell, cell),
            );
            let resp = ui.interact(r, Id::new(("tile_pivot_cell", row, col)), Sense::click());
            let is_sel = (pivot.x - px).abs() < 0.1 && (pivot.y - py).abs() < 0.1;
            let fill = if is_sel {
                t.accent
            } else if resp.hovered() {
                t.panel
            } else {
                t.sunken
            };
            let stroke = if is_sel {
                Stroke::new(1.5, t.ink_strong)
            } else {
                t.hairline()
            };
            let p = ui.painter();
            p.rect_filled(r, CornerRadius::ZERO, fill);
            p.rect_stroke(r, CornerRadius::ZERO, stroke, StrokeKind::Inside);
            if resp.clicked() && !is_sel {
                *pivot = glam::Vec2::new(px, py);
                *changed = true;
            }
        }
    }
}

/// Renders confirmation modals for `BakeAutoLayerCommand` and `MoveEntitiesToLayerCommand`.
pub(crate) fn render_layer_ops_modals(
    app: &mut ShellApp,
    ui: &mut Ui,
    state: &mut LayerViewState,
    def: &LayerDef,
) {
    if state.pending_bake_auto == Some(def.id) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum BakeAction {
            Cancel,
            Bake,
        }
        let buttons = [
            ModalButton::new(BakeAction::Cancel, "Cancel", ModalButtonKind::Normal),
            ModalButton::new(BakeAction::Bake, "Bake to Tiles", ModalButtonKind::Danger),
        ];
        match modal(
            ui.ctx(),
            "bake_auto_layer_modal",
            ModalSpec {
                title: "Bake Auto-Layer to Tiles?".to_string(),
                body: |ui| {
                    ui.label(format!(
                        "Bake '{}' into a static Tiles layer across all levels? This flattens rules into painted tiles (undoable).",
                        def.name
                    ));
                },
                buttons: &buttons,
                width: 390.0,
                default_button: Some(BakeAction::Cancel),
            },
        ) {
            Some(BakeAction::Cancel) => state.pending_bake_auto = None,
            Some(BakeAction::Bake) => {
                let seed = state.sandbox_seed;
                state.pending_bake_auto = None;
                app.execute_command(Box::new(BakeAutoLayerCommand::new(def.id, seed)));
            }
            None => {}
        }
    }

    if let Some((from_id, mut target_id)) = state.pending_move_entities {
        if from_id == def.id {
            #[derive(Clone, Copy, PartialEq, Eq)]
            enum MoveAction {
                Cancel,
                Move,
            }
            let targets: Vec<(lc_core::id::LayerDefId, String)> = app
                .project
                .defs
                .layers
                .iter()
                .filter(|l| l.id != from_id && l.kind == LayerKind::Entities)
                .map(|l| (l.id, l.name.clone()))
                .collect();
            let buttons = [
                ModalButton::new(MoveAction::Cancel, "Cancel", ModalButtonKind::Normal),
                ModalButton::new(MoveAction::Move, "Move Entities", ModalButtonKind::Danger),
            ];
            let res = modal(
                ui.ctx(),
                "move_entities_layer_modal",
                ModalSpec {
                    title: "Move Existing Entities".to_string(),
                    body: |ui| {
                        ui.label(format!("Move all placed entities from '{}' to:", def.name));
                        ui.add_space(6.0);
                        let sel_name = target_id
                            .and_then(|tid| targets.iter().find(|(id, _)| *id == tid))
                            .map(|(_, n)| n.as_str())
                            .unwrap_or("-- Select target layer --");
                        ComboBox::from_id_salt(("move_entities_target", def.id))
                            .selected_text(sel_name)
                            .width(240.0)
                            .show_ui(ui, |ui| {
                                for (id, name) in &targets {
                                    ui.selectable_value(&mut target_id, Some(*id), name);
                                }
                            });
                    },
                    buttons: &buttons,
                    width: 380.0,
                    default_button: Some(MoveAction::Cancel),
                },
            );
            state.pending_move_entities = Some((from_id, target_id));
            match res {
                Some(MoveAction::Cancel) => state.pending_move_entities = None,
                Some(MoveAction::Move) => {
                    if let Some(tid) = target_id {
                        state.pending_move_entities = None;
                        app.execute_command(Box::new(MoveEntitiesToLayerCommand::new(
                            from_id, tid,
                        )));
                    }
                }
                None => {}
            }
        }
    }
}
