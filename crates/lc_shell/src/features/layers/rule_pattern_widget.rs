//! Pattern matrix editor widget (`3x3`/`5x5`/`7x7` with diagonal hatch, tile icons, and `Icon::Prohibit`)
//! and Live 7x7 Sandbox Preview with purple rule-match highlighting.

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Id, PointerButton, Rect, Sense, Stroke,
    StrokeKind, TextureHandle, Ui,
};
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::icons::{paint_icon, Icon};
use lc_canvas::theme::Tokens;
use lc_core::auto_rules::{
    AutoLayerRuleDef, IntGridValueGroupDef, ANY_NON_ZERO_PATTERN, GROUP_PATTERN_BASE,
};
use lc_core::auto_rules_eval::evaluate_auto_rules;
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef, TilesetRect};
use lc_core::id::AutoRuleId;
use lc_ui_api::AssetService;

use super::state::LayerViewState;
use crate::features::editor::shell_app::ShellApp;

/// Returns display color and short label for a pattern condition integer.
pub fn condition_color_and_label(
    cond: i32,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
) -> (Color32, String) {
    let abs = cond.abs();
    if abs == 0 {
        return (Color32::TRANSPARENT, String::new());
    }
    if abs == ANY_NON_ZERO_PATTERN {
        return (Color32::from_rgb(230, 180, 60), "*".to_string());
    }
    if abs >= GROUP_PATTERN_BASE {
        let gid = (abs - GROUP_PATTERN_BASE) as u32;
        if let Some(g) = source_groups.iter().find(|g| g.uid == gid) {
            let col = g
                .color
                .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], 255))
                .unwrap_or(Color32::from_rgb(168, 85, 247));
            let short: String = g.name.chars().take(3).collect();
            return (col, format!("G:{short}"));
        }
        return (Color32::from_rgb(168, 85, 247), format!("G{gid}"));
    }
    if let Some(val) = source_values.iter().find(|v| v.value == abs) {
        let col = Color32::from_rgba_unmultiplied(val.color[0], val.color[1], val.color[2], 255);
        return (col, format!("{abs}"));
    }
    (Color32::from_rgb(148, 163, 184), format!("{abs}"))
}

/// Draws diagonal hatch lines inside `rect` (LDtk empty pattern cell aesthetic).
pub(crate) fn draw_diagonal_hatch(painter: &egui::Painter, rect: Rect, color: Color32) {
    let step = 7.0;
    let w = rect.width();
    let h = rect.height();
    let mut x = -h;
    while x < w {
        let p1 = pos2(rect.min.x + x, rect.min.y);
        let p2 = pos2(rect.min.x + x + h, rect.min.y + h);
        painter.line_segment([p1, p2], Stroke::new(1.0, color));
        x += step;
    }
}

/// Draws an IntGrid value's optional interface tile or colored swatch inside `rect`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_intgrid_value_visual(
    painter: &egui::Painter,
    rect: Rect,
    cond_abs: i32,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    icon_tileset: Option<&TilesetDef>,
    icon_tex: Option<&TextureHandle>,
    forbid: bool,
) {
    let (col, label) = condition_color_and_label(cond_abs, source_values, source_groups);

    if let (Some(ts), Some(tex)) = (icon_tileset, icon_tex) {
        if let Some(tile_rect) = source_values
            .iter()
            .find(|v| v.value == cond_abs)
            .and_then(|v| v.tile)
        {
            if tile_rect.tileset_id == ts.id {
                let sz = tex.size_vec2();
                if sz.x > 0.0 && sz.y > 0.0 {
                    let uv = Rect::from_min_max(
                        pos2(tile_rect.x as f32 / sz.x, tile_rect.y as f32 / sz.y),
                        pos2(
                            (tile_rect.x + tile_rect.w) as f32 / sz.x,
                            (tile_rect.y + tile_rect.h) as f32 / sz.y,
                        ),
                    );
                    let tint = if forbid {
                        Color32::from_rgba_unmultiplied(180, 90, 90, 145)
                    } else {
                        Color32::WHITE
                    };
                    painter.image(tex.id(), rect.shrink(1.5), uv, tint);
                    return;
                }
            }
        }
    }

    let fill = if forbid {
        col.gamma_multiply(0.18)
    } else {
        col.gamma_multiply(0.45)
    };
    painter.rect_filled(rect.shrink(1.5), CornerRadius::ZERO, fill);
    if !forbid {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &label,
            FontId::monospace(10.5),
            Color32::WHITE,
        );
    }
}

/// Renders an interactive hatched `N x N` pattern matrix (`size` = 1/3/5/7) matching LDtk Screenshots 11–13.
#[allow(clippy::too_many_arguments)]
pub fn render_pattern_matrix(
    ui: &mut Ui,
    t: &Tokens,
    rule: &mut AutoLayerRuleDef,
    brush_value: i32,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
    icon_tileset: Option<&TilesetDef>,
    icon_tex: Option<&TextureHandle>,
    interactive: bool,
) -> bool {
    let mut changed = false;
    let sz = rule.size.clamp(1, 7) as usize;
    let half = sz / 2;
    let cell_px = match sz {
        1 => 46.0,
        3 => 40.0,
        5 => 34.0,
        _ => 29.0,
    };
    let gap = 2.0;
    let total = cell_px * sz as f32 + gap * (sz.saturating_sub(1)) as f32;
    let (grid_rect, _) = ui.allocate_exact_size(vec2(total, total), Sense::hover());

    // Frame around the pattern matrix
    let p = ui.painter();
    p.rect_filled(grid_rect.expand(4.0), CornerRadius::ZERO, t.sunken);
    p.rect_stroke(
        grid_rect.expand(4.0),
        CornerRadius::ZERO,
        t.hairline(),
        StrokeKind::Inside,
    );

    let hatch_col = Color32::from_white_alpha(14);
    for py in 0..sz {
        for px in 0..sz {
            let cell_rect = Rect::from_min_size(
                pos2(
                    grid_rect.min.x + px as f32 * (cell_px + gap),
                    grid_rect.min.y + py as f32 * (cell_px + gap),
                ),
                vec2(cell_px, cell_px),
            );
            let is_center = px == half && py == half;
            let cond = rule.pattern_at(px as u8, py as u8);
            let resp = ui.interact(
                cell_rect,
                Id::new(("rule_pat_cell", rule.id, sz, px, py)),
                if interactive {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );

            if ui.is_rect_visible(cell_rect) {
                let cp = ui.painter().with_clip_rect(cell_rect);
                cp.rect_filled(cell_rect, CornerRadius::ZERO, t.canvas);
                draw_diagonal_hatch(&cp, cell_rect, hatch_col);

                if cond > 0 {
                    paint_intgrid_value_visual(
                        &cp,
                        cell_rect,
                        cond,
                        source_values,
                        source_groups,
                        icon_tileset,
                        icon_tex,
                        false,
                    );
                } else if cond < 0 {
                    paint_intgrid_value_visual(
                        &cp,
                        cell_rect,
                        cond.abs(),
                        source_values,
                        source_groups,
                        icon_tileset,
                        icon_tex,
                        true,
                    );
                    paint_icon(
                        &cp,
                        cell_rect.center(),
                        Icon::Prohibit,
                        (cell_px * 0.56).clamp(13.0, 22.0),
                        Color32::from_rgb(245, 115, 70),
                    );
                }

                let border = if resp.hovered() && interactive {
                    Stroke::new(1.5, t.ink_strong)
                } else if is_center {
                    Stroke::new(1.5, Color32::from_rgb(245, 175, 55))
                } else {
                    t.hairline()
                };
                cp.rect_stroke(cell_rect, CornerRadius::ZERO, border, StrokeKind::Inside);
            }

            if interactive {
                let b = brush_value.abs().max(1);
                if resp.clicked_by(PointerButton::Primary) {
                    let next = if cond == b { 0 } else { b };
                    rule.set_pattern_at(px as u8, py as u8, next);
                    changed = true;
                } else if resp.clicked_by(PointerButton::Secondary) {
                    let next = if cond == -b { 0 } else { -b };
                    rule.set_pattern_at(px as u8, py as u8, next);
                    changed = true;
                }
            }
        }
    }

    changed
}

/// Draws a tiny `3x3` / `5x5` preview diagram of `rule`'s pattern inside a compact card.
pub fn paint_mini_pattern_preview(
    painter: &egui::Painter,
    rect: Rect,
    t: &Tokens,
    rule: &AutoLayerRuleDef,
    source_values: &[IntGridValueDef],
    source_groups: &[IntGridValueGroupDef],
) {
    let sz = rule.size.clamp(1, 7) as usize;
    let cw = rect.width() / sz as f32;
    let ch = rect.height() / sz as f32;
    painter.rect_filled(rect, CornerRadius::ZERO, t.sunken);
    for py in 0..sz {
        for px in 0..sz {
            let cond = rule.pattern_at(px as u8, py as u8);
            if cond == 0 {
                continue;
            }
            let r = Rect::from_min_size(
                pos2(rect.min.x + px as f32 * cw, rect.min.y + py as f32 * ch),
                vec2(cw, ch),
            );
            let (col, _) = condition_color_and_label(cond, source_values, source_groups);
            let fill = if cond < 0 { t.bad } else { col };
            painter.rect_filled(r.shrink(0.5), CornerRadius::ZERO, fill);
        }
    }
    painter.rect_stroke(rect, CornerRadius::ZERO, t.hairline(), StrokeKind::Inside);
}

/// Renders `tile_id` from `tileset` & `texture` cleanly into `rect`, honoring flip flags.
#[allow(clippy::too_many_arguments)]
pub fn draw_tile_in_rect(
    painter: &egui::Painter,
    rect: Rect,
    tileset: Option<&TilesetDef>,
    texture: Option<&TextureHandle>,
    tile_id: u32,
    flip_x: bool,
    flip_y: bool,
    tint: Color32,
) {
    let (Some(ts), Some(tex)) = (tileset, texture) else {
        painter.rect_filled(
            rect.shrink(1.0),
            CornerRadius::ZERO,
            tint.gamma_multiply(0.35),
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("{tile_id}"),
            FontId::monospace(9.5),
            Color32::WHITE,
        );
        return;
    };

    let tex_sz = tex.size_vec2();
    let tw = ts.tile_size.x.max(1);
    let th = ts.tile_size.y.max(1);
    let eff_w = (tex_sz.x as u32).saturating_sub(ts.padding * 2);
    let cols = (eff_w + ts.spacing)
        .checked_div(tw + ts.spacing)
        .unwrap_or(1)
        .max(1);
    let col = tile_id % cols;
    let row = tile_id / cols;
    let px = ts.padding + col * (tw + ts.spacing);
    let py = ts.padding + row * (th + ts.spacing);

    let mut u0 = px as f32 / tex_sz.x.max(1.0);
    let mut u1 = (px + tw) as f32 / tex_sz.x.max(1.0);
    let mut v0 = py as f32 / tex_sz.y.max(1.0);
    let mut v1 = (py + th) as f32 / tex_sz.y.max(1.0);
    if flip_x {
        std::mem::swap(&mut u0, &mut u1);
    }
    if flip_y {
        std::mem::swap(&mut v0, &mut v1);
    }
    painter.image(
        tex.id(),
        rect,
        Rect::from_min_max(pos2(u0, v0), pos2(u1, v1)),
        tint,
    );
}

/// Renders an IntGrid value's icon tile (`TilesetRect`) into `rect`, falling back to solid color.
pub fn draw_tileset_rect_in_rect(
    app: &ShellApp,
    ctx: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    ts_rect: Option<TilesetRect>,
    fallback_col: Color32,
) {
    if let Some(tr) = ts_rect {
        if let Some(ts) = app.project.defs.tileset(tr.tileset_id) {
            if let Ok(tex) = app.assets.load_texture(ctx, &ts.path) {
                let sz = tex.size_vec2();
                if sz.x > 0.0 && sz.y > 0.0 {
                    let uv = Rect::from_min_max(
                        pos2(tr.x as f32 / sz.x, tr.y as f32 / sz.y),
                        pos2((tr.x + tr.w) as f32 / sz.x, (tr.y + tr.h) as f32 / sz.y),
                    );
                    painter.image(tex.id(), rect, uv, Color32::WHITE);
                    return;
                }
            }
        }
    }
    painter.rect_filled(rect, CornerRadius::ZERO, fallback_col);
}

/// Renders the interactive 7x7 Live Auto-Rule Sandbox Preview with optional purple match highlight.
#[allow(clippy::too_many_arguments)]
pub fn render_sandbox_preview(
    ui: &mut Ui,
    t: &Tokens,
    state: &mut LayerViewState,
    def: &LayerDef,
    source_values: &[IntGridValueDef],
    tileset: Option<&TilesetDef>,
    tileset_tex: Option<&TextureHandle>,
    interactive: bool,
) {
    let biomes: Vec<String> = state
        .sandbox_biomes_text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect();

    let tiles = if state.show_render_preview {
        evaluate_auto_rules(
            &state.sandbox_grid,
            source_values,
            &def.auto_rule_groups,
            &biomes,
            None,
            state.sandbox_seed,
        )
    } else {
        Vec::new()
    };

    // Highlight cells matched by the currently hovered or selected rule (LDtk Screenshots 9-10)
    let highlight_id: Option<AutoRuleId> = state.hovered_rule_id.or(state.selected_rule_id);
    let highlighted_cells: Vec<(i32, i32)> = if let Some(hl_id) = highlight_id {
        if let Some(isolated) = isolated_single_rule_groups(def, hl_id) {
            evaluate_auto_rules(
                &state.sandbox_grid,
                source_values,
                &isolated,
                &biomes,
                None,
                state.sandbox_seed,
            )
            .into_iter()
            .map(|e| (e.x, e.y))
            .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let cell_px = 28.0;
    let total = cell_px * 7.0;
    let (grid_rect, _) = ui.allocate_exact_size(vec2(total, total), Sense::hover());

    {
        let p = ui.painter().with_clip_rect(grid_rect);
        draw_checkerboard(&p, grid_rect, 10.0, t);
    }

    for y in 0..7u32 {
        for x in 0..7u32 {
            let r = Rect::from_min_size(
                pos2(
                    grid_rect.min.x + x as f32 * cell_px,
                    grid_rect.min.y + y as f32 * cell_px,
                ),
                vec2(cell_px, cell_px),
            );
            let resp = ui.interact(
                r,
                Id::new(("sandbox_cell", def.id, x, y)),
                if interactive {
                    Sense::click_and_drag()
                } else {
                    Sense::hover()
                },
            );

            let v = state.sandbox_grid.get(x, y).copied().unwrap_or(0);
            if v > 0 {
                let col = source_values
                    .iter()
                    .find(|ig| ig.value == v)
                    .map(|ig| {
                        Color32::from_rgba_unmultiplied(ig.color[0], ig.color[1], ig.color[2], 140)
                    })
                    .unwrap_or(t.accent.gamma_multiply(0.35));
                ui.painter()
                    .rect_filled(r.shrink(1.0), CornerRadius::ZERO, col);
            }

            if interactive {
                if resp.clicked_by(PointerButton::Primary)
                    || (resp.dragged_by(PointerButton::Primary))
                {
                    let _ = state.sandbox_grid.set(x, y, state.sandbox_paint_value);
                } else if resp.clicked_by(PointerButton::Secondary)
                    || resp.dragged_by(PointerButton::Secondary)
                {
                    let _ = state.sandbox_grid.set(x, y, 0);
                }
            }
        }
    }

    // Paint generated Auto-Tiles
    for tile in &tiles {
        if tile.x < 0 || tile.x >= 7 || tile.y < 0 || tile.y >= 7 {
            continue;
        }
        let r = Rect::from_min_size(
            pos2(
                grid_rect.min.x + tile.x as f32 * cell_px,
                grid_rect.min.y + tile.y as f32 * cell_px,
            ),
            vec2(cell_px, cell_px),
        );
        let alpha = (tile.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        draw_tile_in_rect(
            ui.painter(),
            r,
            tileset,
            tileset_tex,
            tile.tile_id,
            tile.flip_x,
            tile.flip_y,
            Color32::from_white_alpha(alpha),
        );
    }

    // Purple highlight boxes for matched cells of active/hovered rule
    let purple = Color32::from_rgb(208, 64, 248);
    for (hx, hy) in highlighted_cells {
        if (0..7).contains(&hx) && (0..7).contains(&hy) {
            let r = Rect::from_min_size(
                pos2(
                    grid_rect.min.x + hx as f32 * cell_px,
                    grid_rect.min.y + hy as f32 * cell_px,
                ),
                vec2(cell_px, cell_px),
            );
            ui.painter().rect_filled(
                r,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(192, 56, 240, 48),
            );
            ui.painter().rect_stroke(
                r,
                CornerRadius::ZERO,
                Stroke::new(1.5, purple),
                StrokeKind::Inside,
            );
        }
    }

    // Subtle cell grid + outer hairline
    let p = ui.painter();
    for i in 1..7 {
        let off = i as f32 * cell_px;
        p.line_segment(
            [
                pos2(grid_rect.min.x + off, grid_rect.min.y),
                pos2(grid_rect.min.x + off, grid_rect.max.y),
            ],
            t.hairline(),
        );
        p.line_segment(
            [
                pos2(grid_rect.min.x, grid_rect.min.y + off),
                pos2(grid_rect.max.x, grid_rect.min.y + off),
            ],
            t.hairline(),
        );
    }
    p.rect_stroke(
        grid_rect,
        CornerRadius::ZERO,
        t.hairline(),
        StrokeKind::Inside,
    );
}

fn isolated_single_rule_groups(
    def: &LayerDef,
    target_rule: AutoRuleId,
) -> Option<Vec<lc_core::auto_rules::AutoLayerRuleGroup>> {
    for g in &def.auto_rule_groups {
        if let Some(r) = g.rule(target_rule) {
            let mut single_grp = g.clone();
            let mut active_rule = r.clone();
            active_rule.active = true;
            single_grp.active = true;
            single_grp.rules = vec![active_rule];
            return Some(vec![single_grp]);
        }
    }
    None
}
