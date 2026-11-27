//! Right-side `LAYER PREVIEW` panel showing grid cells, kind representation, and usage stats.

use egui::{vec2, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::meta_row;
use lc_core::defs::LayerDef;
use lc_core::layer::LayerKind;
use lc_feature_layers::count_layer_def_usage;

use super::left_panel::{layer_kind_color, layer_kind_label};
use crate::features::editor::shell_app::ShellApp;
use crate::shared::ui::section_container::SectionContainer;

/// Renders the right `LAYER PREVIEW` section for `def`.
pub fn render_layer_preview_panel(app: &ShellApp, ui: &mut Ui, t: &Tokens, def: &LayerDef) {
    let mut always_open = false;
    SectionContainer::new("LAYER PREVIEW")
        .use_toggle_collapse(false)
        .show(
            ui,
            t,
            &mut always_open,
            |_| {},
            |ui| {
                render_layer_grid_preview(ui, t, def);
                ui.add_space(10.0);

                let stats = count_layer_def_usage(&app.project, def.id);
                let tileset_name = def
                    .tileset_id
                    .and_then(|id| app.project.defs.tileset(id))
                    .map(|ts| ts.name.as_str())
                    .unwrap_or("None");

                meta_row(ui, "Kind", layer_kind_label(def.kind));
                meta_row(
                    ui,
                    "Grid",
                    &format!("{}x{} px", def.grid_size.x, def.grid_size.y),
                );
                if matches!(
                    def.kind,
                    LayerKind::Tiles | LayerKind::Auto | LayerKind::IntGrid
                ) {
                    meta_row(ui, "Tileset", tileset_name);
                }
                if def.kind == LayerKind::IntGrid {
                    meta_row(
                        ui,
                        "Values",
                        &format!("{} defined", def.int_grid_values.len()),
                    );
                }
                let rule_count: usize = def.auto_rule_groups.iter().map(|g| g.rules.len()).sum();
                meta_row(
                    ui,
                    "Auto-Rules",
                    if def.uses_auto_rules() {
                        if rule_count == 0 {
                            "Enabled (0 rules)".to_string()
                        } else {
                            format!(
                                "{} groups / {} rules",
                                def.auto_rule_groups.len(),
                                rule_count
                            )
                        }
                    } else {
                        "Locked (Uninteractable)".to_string()
                    }
                    .as_str(),
                );
                if !app.project.worlds.is_empty() {
                    let matching_worlds = app
                        .project
                        .worlds
                        .iter()
                        .filter(|w| w.layer_matches(def))
                        .count();
                    meta_row(
                        ui,
                        "Worlds",
                        &format!("{}/{} matched", matching_worlds, app.project.worlds.len()),
                    );
                }
                meta_row(
                    ui,
                    "Levels",
                    &format!(
                        "{} ({} active)",
                        stats.total_instances, stats.non_empty_levels
                    ),
                );
                meta_row(ui, "Placed", &format!("{} items", stats.placed_items));
            },
        );
}

fn render_layer_grid_preview(ui: &mut Ui, t: &Tokens, def: &LayerDef) {
    let avail_w = ui.available_width().max(120.0);
    let height = 160.0;
    let (rect, _) = ui.allocate_exact_size(vec2(avail_w, height), Sense::hover());

    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::ZERO, t.canvas);
    draw_checkerboard(&painter, rect, 12.0, t);
    painter.rect_stroke(rect, CornerRadius::ZERO, t.hairline(), StrokeKind::Inside);

    let pad = 16.0;
    let inner = rect.shrink(pad);
    let gx = def.grid_size.x.max(1) as f32;
    let gy = def.grid_size.y.max(1) as f32;
    let aspect = (gx / gy).clamp(0.25, 4.0);

    let cols = 5usize;
    let rows = 4usize;
    let cell_h = (inner.height() / rows as f32).min(inner.width() / (cols as f32 * aspect));
    let cell_w = cell_h * aspect;
    let total_w = cell_w * cols as f32;
    let total_h = cell_h * rows as f32;
    let origin = Pos2::new(
        inner.center().x - total_w * 0.5,
        inner.center().y - total_h * 0.5,
    );

    let kind_col = layer_kind_color(def.kind);

    // Paint sample cells according to layer kind
    for r in 0..rows {
        for c in 0..cols {
            let cell_rect = Rect::from_min_size(
                Pos2::new(origin.x + c as f32 * cell_w, origin.y + r as f32 * cell_h),
                vec2(cell_w, cell_h),
            );
            match def.kind {
                LayerKind::IntGrid => {
                    if !def.int_grid_values.is_empty() && (r == rows - 1 || (r + c) % 3 == 0) {
                        let v_idx = (r + c) % def.int_grid_values.len();
                        let rgba = def.int_grid_values[v_idx].color;
                        let col = Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], 190);
                        painter.rect_filled(cell_rect.shrink(1.0), CornerRadius::ZERO, col);
                    }
                }
                LayerKind::Tiles => {
                    if (r == 1 && (1..=3).contains(&c)) || (r == 2 && c == 2) {
                        painter.rect_filled(
                            cell_rect.shrink(1.0),
                            CornerRadius::ZERO,
                            kind_col.gamma_multiply(0.35),
                        );
                    }
                }
                LayerKind::Entities => {
                    if (r == 1 && c == 1) || (r == 2 && c == 3) {
                        let box_rect = cell_rect.shrink(cell_w.min(cell_h) * 0.22);
                        painter.rect_filled(
                            box_rect,
                            CornerRadius::ZERO,
                            kind_col.gamma_multiply(0.35),
                        );
                        painter.rect_stroke(
                            box_rect,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, kind_col),
                            StrokeKind::Inside,
                        );
                    }
                }
                LayerKind::Auto => {
                    if (r + c) % 2 == 0 {
                        painter.rect_filled(
                            cell_rect.shrink(2.0),
                            CornerRadius::ZERO,
                            kind_col.gamma_multiply(0.22),
                        );
                    }
                }
            }
            painter.rect_stroke(
                cell_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, t.line.gamma_multiply(0.65)),
                StrokeKind::Inside,
            );
        }
    }
}
