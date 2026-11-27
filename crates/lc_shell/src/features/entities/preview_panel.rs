//! Right-hand `LIVE PREVIEW` panel card matching Tileset Workspace's right panel layout.

use egui::{vec2, CornerRadius, FontId, Frame, Margin, RichText, ScrollArea, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_core::defs::{EntityDef, EntityRenderMode};
use lc_core::fields::{FieldType, FieldValue};
use lc_feature_entities::{
    analyze_field_visual_effects, format_field_value, PREVIEW_SAMPLE_ENTITY_ID_BASE,
};

use super::field_type_grid::field_type_color;
use super::preview::{preview_samples_for_entity, render_entity_live_preview};
use super::state::EntityViewState;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::section_container::SectionContainer;

/// Renders the fixed-width right `LIVE PREVIEW` card with entity metrics and per-field visual impact analysis.
pub fn render_right_preview_panel(
    app: &ShellApp,
    ui: &mut Ui,
    t: &Tokens,
    state: &mut EntityViewState,
    def: &EntityDef,
    card_w: f32,
    panel_h: f32,
) {
    // Match Tileset Workspace's right panel card geometry & chrome cleanly.
    let header_h = 24.0;
    let header_gap = 4.0;
    let card_h = (panel_h - header_h - header_gap).max(60.0);
    let mut dummy_collapsed = false;

    let samples = preview_samples_for_entity(app, def);
    let smart_rgba = def.smart_color(&samples, &app.project.defs.enums);
    let smart_overridden = smart_rgba != def.color;

    let resp = SectionContainer::new("LIVE PREVIEW")
        .use_outline(true)
        .use_toggle_collapse(false)
        .selected(state.focused_preview)
        .fill(t.panel)
        .margin(Margin::ZERO)
        .header_height(header_h)
        .header_gap(header_gap)
        .size(vec2(card_w, card_h))
        .show(
            ui,
            t,
            &mut dummy_collapsed,
            |_| {},
            |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        let inner_w = (card_w - 18.0).max(120.0);
                        ui.set_width(inner_w);
                        ui.set_max_width(inner_w);

                        ScrollArea::vertical()
                            .id_salt(("entity_preview_scroll", def.id))
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(inner_w);
                                ui.set_max_width(inner_w);

                                render_entity_live_preview(app, ui, t, def, &samples, 152.0);
                                ui.add_space(8.0);

                                ui.label(
                                    RichText::new(format!(
                                        "Size: {}x{} px",
                                        def.size_px.x, def.size_px.y
                                    ))
                                    .size(11.0)
                                    .color(t.muted),
                                );
                                let vis_label = if def.render_mode == EntityRenderMode::Tile {
                                    def.tileset_id
                                        .or_else(|| def.tile_rect.map(|r| r.tileset_id))
                                        .and_then(|id| app.project.defs.tileset(id))
                                        .map(|ts| format!("Tile: {}", ts.name))
                                        .unwrap_or_else(|| "Tile".to_string())
                                } else {
                                    format!("{:?}", def.render_mode)
                                };
                                ui.label(
                                    RichText::new(format!("Visual: {vis_label}"))
                                        .size(11.0)
                                        .color(t.muted),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "Pivot: ({:.2}, {:.2})",
                                        def.pivot.x, def.pivot.y
                                    ))
                                    .size(11.0)
                                    .color(t.muted),
                                );
                                if smart_overridden {
                                    ui.label(
                                        RichText::new(format!(
                                            "Smart Color: #{:02X}{:02X}{:02X}",
                                            smart_rgba[0], smart_rgba[1], smart_rgba[2]
                                        ))
                                        .size(11.0)
                                        .color(t.accent_ink),
                                    );
                                }
                                ui.label(
                                    RichText::new(format!("Fields: {}", def.fields.len()))
                                        .size(11.0)
                                        .color(t.muted),
                                );

                                if !def.fields.is_empty() {
                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new("FIELD SAMPLES")
                                            .font(FontId::new(10.5, theme::mono_bold_for(ui.ctx())))
                                            .color(t.accent_ink),
                                    );
                                    ui.add_space(4.0);

                                    for field in &def.fields {
                                        let Some(val) = samples.get(&field.name) else {
                                            continue;
                                        };
                                        let impacts = analyze_field_visual_effects(
                                            field,
                                            val,
                                            &app.project.defs.enums,
                                            16,
                                        );
                                        let affects =
                                            impacts.iter().any(|imp| imp.affects_entity());
                                        let type_col = field_type_color(field.base_type());
                                        let type_tag = preview_type_tag(&field.field_type);
                                        let val_txt = format_sample_summary(val);

                                        Frame::new()
                                            .fill(t.sunken)
                                            .stroke(t.hairline())
                                            .corner_radius(CornerRadius::ZERO)
                                            .inner_margin(Margin::symmetric(6, 4))
                                            .show(ui, |ui| {
                                                ui.set_width((inner_w - 14.0).max(80.0));
                                                ui.horizontal(|ui| {
                                                    ui.add(
                                                        egui::Label::new(
                                                            RichText::new(&field.name)
                                                                .size(11.0)
                                                                .strong()
                                                                .color(t.ink_strong),
                                                        )
                                                        .truncate(),
                                                    );
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.label(
                                                                RichText::new(type_tag)
                                                                    .size(10.0)
                                                                    .strong()
                                                                    .color(type_col),
                                                            );
                                                        },
                                                    );
                                                });
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(format!("Sample: {val_txt}"))
                                                            .size(10.5)
                                                            .color(t.muted),
                                                    )
                                                    .truncate(),
                                                );
                                                for imp in impacts {
                                                    ui.add(
                                                        egui::Label::new(
                                                            RichText::new(imp.label())
                                                                .size(10.5)
                                                                .color(if affects {
                                                                    t.accent_ink
                                                                } else {
                                                                    t.faint
                                                                }),
                                                        )
                                                        .truncate(),
                                                    );
                                                }
                                            });
                                        ui.add_space(4.0);
                                    }
                                }
                            });
                    });
            },
        );
    if resp.clicked_inside {
        state.focused_preview = true;
    }
}

fn preview_type_tag(ft: &FieldType) -> String {
    let base = match ft {
        FieldType::Array(inner) => inner.as_ref(),
        other => other,
    };
    let short = match base {
        FieldType::Int => "Int",
        FieldType::Float => "Float",
        FieldType::Bool => "Bool",
        FieldType::String => "Str",
        FieldType::Multilines => "Text",
        FieldType::Color => "Color",
        FieldType::Enum(_) => "Enum",
        FieldType::Point => "Point",
        FieldType::EntityRef => "Ref",
        FieldType::FilePath => "File",
        FieldType::Tile => "Tile",
        FieldType::Array(_) => "Arr",
    };
    if matches!(ft, FieldType::Array(_)) {
        format!("[{short}]")
    } else {
        short.to_string()
    }
}

fn format_sample_summary(v: &FieldValue) -> String {
    match v {
        FieldValue::EntityRef(Some(id)) if id.0 >= PREVIEW_SAMPLE_ENTITY_ID_BASE => {
            format!("Target#{}", id.0 - PREVIEW_SAMPLE_ENTITY_ID_BASE + 1)
        }
        FieldValue::Array(items) => {
            let joined = items
                .iter()
                .map(format_sample_summary)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{joined}]")
        }
        other => format_field_value(other),
    }
}
