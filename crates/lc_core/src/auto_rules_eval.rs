//! Deterministic pure-Rust evaluator for LDtk-compatible Auto-Layer rules.

use crate::auto_rules::{
    AutoLayerRuleDef, AutoLayerRuleGroup, AutoRuleCheckerMode, AutoRuleTileMode,
    BiomeRequirementMode, EvaluatedAutoTile, ANY_NON_ZERO_PATTERN, GROUP_PATTERN_BASE,
};
use crate::defs::IntGridValueDef;
use crate::grid::Grid;

/// Evaluates `groups` against `source_grid` and returns generated auto-tiles in render order.
///
/// Matches LDtk semantics:
/// - Groups and rules execute top-to-bottom.
/// - `break_on_match` stops evaluating further rules for a cell `(cx, cy)` once matched.
/// - `flip_x` / `flip_y` test mirrored pattern orientations and propagate flip flags to output.
/// - `chance`, `perlin_active`, and candidate selection are deterministic given `seed`.
pub fn evaluate_auto_rules(
    source_grid: &Grid<i32>,
    int_grid_values: &[IntGridValueDef],
    groups: &[AutoLayerRuleGroup],
    level_biomes: &[String],
    killed_by_grid: Option<&Grid<i32>>,
    seed: u64,
) -> Vec<EvaluatedAutoTile> {
    let w = source_grid.width() as i32;
    let h = source_grid.height() as i32;
    if w <= 0 || h <= 0 {
        return Vec::new();
    }

    let mut out = Vec::new();

    for cy in 0..h {
        for cx in 0..w {
            if let Some(kill_grid) = killed_by_grid {
                if kill_grid.get(cx as u32, cy as u32).is_some_and(|&v| v != 0) {
                    continue;
                }
            }

            'cell_rules: for group in groups {
                if !group.active || !group_matches_biome(group, level_biomes) {
                    continue;
                }

                for rule in &group.rules {
                    if !rule.active || rule.tile_rects.is_empty() {
                        continue;
                    }
                    if !passes_spatial_filters(rule, cx, cy) {
                        continue;
                    }
                    if rule.chance < 1.0 {
                        let roll = deterministic_unit_f32(seed, rule.id.0, cx, cy, 0xA11C_E001);
                        if rule.chance <= 0.0 || roll >= rule.chance {
                            continue;
                        }
                    }

                    let Some((fx, fy)) =
                        match_rule_orientations(rule, source_grid, int_grid_values, cx, cy, seed)
                    else {
                        continue;
                    };

                    let pick_idx = if rule.tile_rects.len() == 1 {
                        0
                    } else {
                        let roll = deterministic_hash(seed, rule.id.0, cx, cy, 0x711E_5EED);
                        (roll as usize) % rule.tile_rects.len()
                    };

                    if let Some(rect) = rule.tile_rects.get(pick_idx) {
                        emit_tile_rect(rule, rect, cx, cy, fx, fy, &mut out);
                    }

                    if rule.break_on_match {
                        break 'cell_rules;
                    }
                }
            }
        }
    }

    out
}

fn group_matches_biome(group: &AutoLayerRuleGroup, level_biomes: &[String]) -> bool {
    if group.required_biome_values.is_empty() {
        return true;
    }
    let any_match = group.required_biome_values.iter().any(|req| {
        level_biomes
            .iter()
            .any(|b| b.eq_ignore_ascii_case(req.trim()))
    });
    match group.biome_requirement_mode {
        BiomeRequirementMode::Require => any_match,
        BiomeRequirementMode::Exclude => !any_match,
    }
}

fn passes_spatial_filters(rule: &AutoLayerRuleDef, cx: i32, cy: i32) -> bool {
    let xm = rule.x_modulo.max(1) as i32;
    let ym = rule.y_modulo.max(1) as i32;
    if (cx - rule.x_offset).rem_euclid(xm) != 0 || (cy - rule.y_offset).rem_euclid(ym) != 0 {
        return false;
    }

    match rule.checker {
        AutoRuleCheckerMode::None => {}
        AutoRuleCheckerMode::Horizontal => {
            if (cx + cy).rem_euclid(2) != 0 {
                return false;
            }
        }
        AutoRuleCheckerMode::Vertical => {
            if (cx + cy).rem_euclid(2) != 1 {
                return false;
            }
        }
    }

    if rule.perlin_active {
        let n = perlin_2d(
            cx as f32 * rule.perlin_scale.max(0.001),
            cy as f32 * rule.perlin_scale.max(0.001),
            rule.perlin_seed,
            rule.perlin_octaves.clamp(1, 6),
        );
        if n <= 0.0 {
            return false;
        }
    }

    true
}

fn match_rule_orientations(
    rule: &AutoLayerRuleDef,
    grid: &Grid<i32>,
    int_grid_values: &[IntGridValueDef],
    cx: i32,
    cy: i32,
    seed: u64,
) -> Option<(bool, bool)> {
    let mut candidates = [(false, false); 4];
    let mut count = 0usize;

    let variants = [
        (false, false, true),
        (true, false, rule.flip_x),
        (false, true, rule.flip_y),
        (true, true, rule.flip_x && rule.flip_y),
    ];

    for (fx, fy, allowed) in variants {
        if allowed && pattern_matches(rule, grid, int_grid_values, cx, cy, fx, fy) {
            if let Some(slot) = candidates.get_mut(count) {
                *slot = (fx, fy);
                count += 1;
            }
        }
    }

    if count == 0 {
        return None;
    }
    if count == 1 {
        return candidates.first().copied();
    }
    let idx = (deterministic_hash(seed, rule.id.0, cx, cy, 0xF11F_5EED) as usize) % count;
    candidates.get(idx).copied()
}

fn pattern_matches(
    rule: &AutoLayerRuleDef,
    grid: &Grid<i32>,
    int_grid_values: &[IntGridValueDef],
    cx: i32,
    cy: i32,
    flip_x: bool,
    flip_y: bool,
) -> bool {
    let sz = AutoLayerRuleDef::normalized_size(rule.size) as i32;
    let half = sz / 2;
    let w = grid.width() as i32;
    let h = grid.height() as i32;

    for py in 0..sz {
        for px in 0..sz {
            let pat_x = if flip_x { sz - 1 - px } else { px };
            let pat_y = if flip_y { sz - 1 - py } else { py };
            let cond = rule.pattern_at(pat_x as u8, pat_y as u8);
            if cond == 0 {
                continue;
            }

            let gx = cx + (px - half);
            let gy = cy + (py - half);
            let cell_val = if gx >= 0 && gy >= 0 && gx < w && gy < h {
                grid.get(gx as u32, gy as u32).copied().unwrap_or(0)
            } else if let Some(oob) = rule.out_of_bounds_value {
                oob
            } else {
                return false;
            };

            if !cell_satisfies_condition(cell_val, cond, int_grid_values) {
                return false;
            }
        }
    }

    true
}

fn cell_satisfies_condition(cell_val: i32, cond: i32, int_grid_values: &[IntGridValueDef]) -> bool {
    let require = cond > 0;
    let abs_cond = cond.saturating_abs();

    let matched = if abs_cond == ANY_NON_ZERO_PATTERN {
        cell_val != 0
    } else if abs_cond > GROUP_PATTERN_BASE {
        let group_uid = (abs_cond - GROUP_PATTERN_BASE) as u32;
        int_grid_values
            .iter()
            .any(|v| v.value == cell_val && v.group_uid == Some(group_uid))
    } else {
        cell_val == abs_cond
    };

    if require {
        matched
    } else {
        !matched
    }
}

fn emit_tile_rect(
    rule: &AutoLayerRuleDef,
    rect: &crate::auto_rules::AutoTileRect,
    cx: i32,
    cy: i32,
    flip_x: bool,
    flip_y: bool,
    out: &mut Vec<EvaluatedAutoTile>,
) {
    match rule.tile_mode {
        AutoRuleTileMode::Single => {
            if let Some(&tile_id) = rect.tiles.first() {
                out.push(EvaluatedAutoTile {
                    x: cx,
                    y: cy,
                    tile_id,
                    flip_x,
                    flip_y,
                    alpha: rule.alpha.clamp(0.0, 1.0),
                    rule_id: rule.id,
                });
            }
        }
        AutoRuleTileMode::Stamp => {
            let sw = rect.w.max(1) as i32;
            let sh = rect.h.max(1) as i32;
            let ox = ((sw - 1) as f32 * rule.pivot_x.clamp(0.0, 1.0)).round() as i32;
            let oy = ((sh - 1) as f32 * rule.pivot_y.clamp(0.0, 1.0)).round() as i32;

            for ly in 0..rect.h {
                for lx in 0..rect.w {
                    let src_x = if flip_x { rect.w - 1 - lx } else { lx };
                    let src_y = if flip_y { rect.h - 1 - ly } else { ly };
                    if let Some(tile_id) = rect.tile_at(src_x, src_y) {
                        out.push(EvaluatedAutoTile {
                            x: cx + lx as i32 - ox,
                            y: cy + ly as i32 - oy,
                            tile_id,
                            flip_x,
                            flip_y,
                            alpha: rule.alpha.clamp(0.0, 1.0),
                            rule_id: rule.id,
                        });
                    }
                }
            }
        }
    }
}

fn deterministic_hash(seed: u64, rule_id: u64, x: i32, y: i32, salt: u64) -> u64 {
    let mut h = seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= rule_id.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= (x as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= (y as i64 as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}

fn deterministic_unit_f32(seed: u64, rule_id: u64, x: i32, y: i32, salt: u64) -> f32 {
    let bits = deterministic_hash(seed, rule_id, x, y, salt) >> 40;
    (bits as f32) / ((1u64 << 24) as f32)
}

fn perlin_2d(x: f32, y: f32, seed: i32, octaves: u32) -> f32 {
    let mut total = 0.0f32;
    let mut freq = 1.0f32;
    let mut amp = 1.0f32;
    let mut norm = 0.0f32;

    for oct in 0..octaves {
        total += single_perlin_2d(x * freq, y * freq, seed.wrapping_add(oct as i32 * 37)) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }

    if norm > 0.0 {
        total / norm
    } else {
        0.0
    }
}

fn single_perlin_2d(x: f32, y: f32, seed: i32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0.wrapping_add(1);
    let y1 = y0.wrapping_add(1);

    let sx = fade(x - x0 as f32);
    let sy = fade(y - y0 as f32);

    let n00 = grad_dot(seed, x0, y0, x - x0 as f32, y - y0 as f32);
    let n10 = grad_dot(seed, x1, y0, x - x1 as f32, y - y0 as f32);
    let n01 = grad_dot(seed, x0, y1, x - x0 as f32, y - y1 as f32);
    let n11 = grad_dot(seed, x1, y1, x - x1 as f32, y - y1 as f32);

    let ix0 = n00 + sx * (n10 - n00);
    let ix1 = n01 + sx * (n11 - n01);
    ix0 + sy * (ix1 - ix0)
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn grad_dot(seed: i32, gx: i32, gy: i32, dx: f32, dy: f32) -> f32 {
    let h = deterministic_hash(seed as i64 as u64, 0, gx, gy, 0x517C_C1B7);
    match h & 7 {
        0 => dx + dy,
        1 => -dx + dy,
        2 => dx - dy,
        3 => -dx - dy,
        4 => dx,
        5 => -dx,
        6 => dy,
        _ => -dy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auto_rules::{AutoTileRect, IntGridValueGroupDef};
    use crate::id::{AutoRuleGroupId, AutoRuleId};

    #[test]
    fn pattern_resize_preserves_center_and_neighbors() {
        let mut rule = AutoLayerRuleDef::new(AutoRuleId(1));
        rule.set_pattern_at(1, 1, 1);
        rule.set_pattern_at(1, 0, -1);

        rule.resize_pattern(5);
        assert_eq!(rule.size, 5);
        assert_eq!(rule.pattern.len(), 25);
        assert_eq!(rule.pattern_at(2, 2), 1);
        assert_eq!(rule.pattern_at(2, 1), -1);

        rule.resize_pattern(3);
        assert_eq!(rule.size, 3);
        assert_eq!(rule.pattern_at(1, 1), 1);
        assert_eq!(rule.pattern_at(1, 0), -1);
    }

    #[test]
    fn evaluator_matches_top_edge_with_break_on_match_and_oob() {
        let mut grid = Grid::new(3, 2, 0);
        let _ = grid.set(0, 1, 1);
        let _ = grid.set(1, 1, 1);
        let _ = grid.set(2, 1, 1);

        let mut top_rule = AutoLayerRuleDef::new(AutoRuleId(10));
        top_rule.set_pattern_at(1, 1, 1);
        top_rule.set_pattern_at(1, 0, -1);
        top_rule.tile_rects.push(AutoTileRect::single(5));
        top_rule.break_on_match = true;

        let mut fill_rule = AutoLayerRuleDef::new(AutoRuleId(11));
        fill_rule.set_pattern_at(1, 1, 1);
        fill_rule.tile_rects.push(AutoTileRect::single(9));

        let mut group = AutoLayerRuleGroup::new(AutoRuleGroupId(1), "Terrain");
        group.rules = vec![top_rule, fill_rule];

        let tiles = evaluate_auto_rules(&grid, &[], &[group], &[], None, 42);
        assert_eq!(tiles.len(), 3);
        assert!(tiles.iter().all(|t| t.tile_id == 5 && t.y == 1));
    }

    #[test]
    fn evaluator_supports_flip_x_and_group_matching() {
        let mut grid = Grid::new(3, 1, 0);
        let _ = grid.set(0, 0, 2);
        let _ = grid.set(1, 0, 2);

        let group_def = IntGridValueGroupDef::new(3, "Solids");
        let vals = vec![IntGridValueDef {
            value: 2,
            name: "Stone".to_string(),
            color: [100, 100, 100, 255],
            group_uid: Some(group_def.uid),
            tile: None,
        }];

        // Rule requires group at center and empty on left; flip_x allows empty on right!
        let mut edge = AutoLayerRuleDef::new(AutoRuleId(20));
        edge.set_pattern_at(1, 1, GROUP_PATTERN_BASE + group_def.uid as i32);
        edge.set_pattern_at(0, 1, -ANY_NON_ZERO_PATTERN);
        edge.flip_x = true;
        edge.out_of_bounds_value = Some(0);
        edge.tile_rects.push(AutoTileRect::single(12));

        let mut group = AutoLayerRuleGroup::new(AutoRuleGroupId(2), "Edges");
        group.rules = vec![edge];

        let tiles = evaluate_auto_rules(&grid, &vals, &[group], &[], None, 1);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles.first().map(|t| (t.x, t.flip_x)), Some((0, false)));
        assert_eq!(tiles.get(1).map(|t| (t.x, t.flip_x)), Some((1, true)));
    }

    #[test]
    fn evaluator_respects_modulo_checker_biome_and_killed_by() {
        let grid = Grid::new(4, 2, 1);
        let mut kill = Grid::new(4, 2, 0);
        let _ = kill.set(0, 0, 1);

        let mut rule = AutoLayerRuleDef::new(AutoRuleId(30));
        rule.set_pattern_at(1, 1, 1);
        rule.x_modulo = 2;
        rule.checker = AutoRuleCheckerMode::Horizontal;
        rule.tile_rects.push(AutoTileRect::single(7));

        let mut group = AutoLayerRuleGroup::new(AutoRuleGroupId(3), "TundraOnly");
        group.required_biome_values = vec!["Tundra".to_string()];
        group.rules = vec![rule];

        // Wrong biome -> 0 tiles
        let none = evaluate_auto_rules(
            &grid,
            &[],
            std::slice::from_ref(&group),
            &["Desert".to_string()],
            Some(&kill),
            99,
        );
        assert!(none.is_empty());

        // Matching biome -> (0,0) is killed, (2,0) matches (x%2==0 and (x+y)%2==0)
        let tundra = evaluate_auto_rules(
            &grid,
            &[],
            &[group],
            &["tundra".to_string()],
            Some(&kill),
            99,
        );
        assert_eq!(tundra.len(), 1);
        assert_eq!(tundra.first().map(|t| (t.x, t.y)), Some((2, 0)));
    }
}
