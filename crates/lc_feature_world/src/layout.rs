//! Pure world-layout math for the world overview tab.
//!
//! Snapping, arrangement, and hit-testing deliberately live outside the egui
//! view so the layout rules stay deterministic and unit-testable.

use glam::{IVec2, UVec2};
use lc_core::id::LevelId;
use lc_core::level::Level;
use lc_core::world::{World, WorldLayout};

/// Gap inserted between consecutive levels in linear layouts (LDtk parity).
pub const LINEAR_SPACING: i32 = 48;

/// Resolves the effective world grid, falling back to the project grid.
pub fn world_grid(world: &World, fallback: UVec2) -> IVec2 {
    let grid = world.default_grid.unwrap_or(fallback);
    IVec2::new(grid.x.max(1) as i32, grid.y.max(1) as i32)
}

/// Snaps a scalar to the nearest multiple of `step` (at least 1).
pub fn snap_scalar(value: i32, step: i32) -> i32 {
    let step = i64::from(step.max(1));
    let value = i64::from(value);
    // `f64::round` rounds halves away from zero, matching editor expectation.
    let snapped = (value as f64 / step as f64).round() * step as f64;
    snapped.clamp(i64::from(i32::MIN) as f64, i64::from(i32::MAX) as f64) as i32
}

/// Snaps `desired` according to the world layout.
///
/// `GridVania` always snaps to the world grid, `Free` only snaps while grid
/// snapping is enabled, and the linear layouts lock the cross axis.
pub fn snap_position(layout: WorldLayout, grid: IVec2, desired: IVec2, grid_snap: bool) -> IVec2 {
    match layout {
        WorldLayout::Free => {
            if grid_snap {
                IVec2::new(
                    snap_scalar(desired.x, grid.x),
                    snap_scalar(desired.y, grid.y),
                )
            } else {
                desired
            }
        }
        WorldLayout::GridVania => IVec2::new(
            snap_scalar(desired.x, grid.x),
            snap_scalar(desired.y, grid.y),
        ),
        WorldLayout::LinearHorizontal => IVec2::new(desired.x, 0),
        WorldLayout::LinearVertical => IVec2::new(0, desired.y),
    }
}

/// Whether `point` lies inside the level rectangle.
///
/// Minimum edges are inclusive and maximum edges exclusive, matching the
/// pixel area actually covered by the level.
pub fn level_contains(level: &Level, point: IVec2) -> bool {
    let width = level.size_px.x.max(1) as i32;
    let height = level.size_px.y.max(1) as i32;
    point.x >= level.world_pos.x
        && point.y >= level.world_pos.y
        && point.x < level.world_pos.x.saturating_add(width)
        && point.y < level.world_pos.y.saturating_add(height)
}

/// Returns the level at `point` on `depth`, preferring the last (topmost) one.
pub fn pick_level_at(levels: &[Level], depth: i32, point: IVec2) -> Option<LevelId> {
    levels
        .iter()
        .rev()
        .find(|level| level.world_depth == depth && level_contains(level, point))
        .map(|level| level.id)
}

/// World-space bounds of all levels, or `None` when the world is empty.
pub fn world_bounds(world: &World) -> Option<(IVec2, IVec2)> {
    let mut iter = world.levels.iter();
    let first = iter.next()?;
    let mut min = first.world_pos;
    let mut max = level_max(first);
    for level in iter {
        min = min.min(level.world_pos);
        max = max.max(level_max(level));
    }
    Some((min, max))
}

fn level_max(level: &Level) -> IVec2 {
    level.world_pos + IVec2::new(level.size_px.x.max(1) as i32, level.size_px.y.max(1) as i32)
}

/// Depth values present in the world, always including `active`, sorted ascending.
pub fn depths(world: &World, active: i32) -> Vec<i32> {
    let mut values: Vec<i32> = world.levels.iter().map(|level| level.world_depth).collect();
    values.push(active);
    values.sort_unstable();
    values.dedup();
    values
}

/// Order of levels after a drag in a linear layout.
///
/// The dragged level is ranked by its preview position while every other level
/// keeps its stored position. Non-linear layouts keep the project order.
pub fn linear_order(
    levels: &[Level],
    layout: WorldLayout,
    dragged: LevelId,
    dragged_pos: IVec2,
) -> Vec<LevelId> {
    if !matches!(
        layout,
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical
    ) {
        return levels.iter().map(|level| level.id).collect();
    }

    let mut ranked: Vec<(i32, LevelId)> = levels
        .iter()
        .map(|level| {
            let pos = if level.id == dragged {
                dragged_pos
            } else {
                level.world_pos
            };
            let axis = match layout {
                WorldLayout::LinearVertical => pos.y,
                _ => pos.x,
            };
            (axis, level.id)
        })
        .collect();
    ranked.sort_by_key(|(axis, _)| *axis);
    ranked.into_iter().map(|(_, id)| id).collect()
}

/// Level IDs ordered by their current position on the layout axis.
///
/// Non-linear layouts keep the project order.
pub fn axis_order(levels: &[Level], layout: WorldLayout) -> Vec<LevelId> {
    if !matches!(
        layout,
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical
    ) {
        return levels.iter().map(|level| level.id).collect();
    }

    let mut ranked: Vec<(i32, LevelId)> = levels
        .iter()
        .map(|level| {
            let axis = match layout {
                WorldLayout::LinearVertical => level.world_pos.y,
                _ => level.world_pos.x,
            };
            (axis, level.id)
        })
        .collect();
    ranked.sort_by_key(|(axis, _)| *axis);
    ranked.into_iter().map(|(_, id)| id).collect()
}

/// Positions for `order` when levels are arranged along a line.
pub fn arrange_linear(
    levels: &[Level],
    order: &[LevelId],
    layout: WorldLayout,
    spacing: i32,
) -> Vec<(LevelId, IVec2)> {
    if !matches!(
        layout,
        WorldLayout::LinearHorizontal | WorldLayout::LinearVertical
    ) {
        return Vec::new();
    }

    let spacing = spacing.max(0);
    let mut cursor: i32 = 0;
    let mut result = Vec::with_capacity(order.len());
    for id in order {
        let Some(level) = levels.iter().find(|level| level.id == *id) else {
            continue;
        };
        let pos = match layout {
            WorldLayout::LinearVertical => IVec2::new(0, cursor),
            _ => IVec2::new(cursor, 0),
        };
        result.push((*id, pos));
        let extent = match layout {
            WorldLayout::LinearVertical => level.size_px.y,
            _ => level.size_px.x,
        };
        cursor = cursor.saturating_add(extent.max(1) as i32 + spacing);
    }
    result
}

/// Default size for a level created from the world view (sixteen grid cells).
pub fn default_level_size(grid: IVec2) -> UVec2 {
    UVec2::new(
        grid.x.max(1).saturating_mul(16).max(16) as u32,
        grid.y.max(1).saturating_mul(16).max(16) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(id: u64, pos: IVec2, size: UVec2, depth: i32) -> Level {
        let mut level = Level::new(LevelId::new(id), format!("L{id}"), size);
        level.world_pos = pos;
        level.world_depth = depth;
        level
    }

    #[test]
    fn snap_scalar_rounds_on_both_sides_of_zero() {
        assert_eq!(snap_scalar(0, 16), 0);
        assert_eq!(snap_scalar(7, 16), 0);
        assert_eq!(snap_scalar(8, 16), 16);
        assert_eq!(snap_scalar(-7, 16), 0);
        assert_eq!(snap_scalar(-8, 16), -16);
        assert_eq!(snap_scalar(-9, 16), -16);
        assert_eq!(snap_scalar(5, 0), 5, "a zero step is treated as one");
    }

    #[test]
    fn free_layout_snaps_only_when_enabled() {
        let grid = IVec2::splat(16);
        assert_eq!(
            snap_position(WorldLayout::Free, grid, IVec2::new(23, -9), true),
            IVec2::new(16, -16)
        );
        assert_eq!(
            snap_position(WorldLayout::Free, grid, IVec2::new(23, -9), false),
            IVec2::new(23, -9)
        );
    }

    #[test]
    fn grid_vania_always_snaps() {
        let grid = IVec2::new(32, 16);
        assert_eq!(
            snap_position(WorldLayout::GridVania, grid, IVec2::new(50, 25), false),
            IVec2::new(64, 32)
        );
    }

    #[test]
    fn linear_layouts_lock_the_cross_axis() {
        let grid = IVec2::splat(16);
        assert_eq!(
            snap_position(
                WorldLayout::LinearHorizontal,
                grid,
                IVec2::new(77, 55),
                true
            ),
            IVec2::new(77, 0)
        );
        assert_eq!(
            snap_position(WorldLayout::LinearVertical, grid, IVec2::new(77, 55), true),
            IVec2::new(0, 55)
        );
    }

    #[test]
    fn arrange_linear_uses_size_and_spacing() {
        let levels = vec![
            level(1, IVec2::ZERO, UVec2::new(100, 50), 0),
            level(2, IVec2::ZERO, UVec2::new(80, 40), 0),
            level(3, IVec2::ZERO, UVec2::new(60, 30), 0),
        ];
        let order = vec![LevelId::new(1), LevelId::new(2), LevelId::new(3)];

        let horizontal = arrange_linear(&levels, &order, WorldLayout::LinearHorizontal, 10);
        assert_eq!(
            horizontal,
            vec![
                (LevelId::new(1), IVec2::new(0, 0)),
                (LevelId::new(2), IVec2::new(110, 0)),
                (LevelId::new(3), IVec2::new(200, 0)),
            ]
        );

        let vertical = arrange_linear(&levels, &order, WorldLayout::LinearVertical, 10);
        assert_eq!(
            vertical,
            vec![
                (LevelId::new(1), IVec2::new(0, 0)),
                (LevelId::new(2), IVec2::new(0, 60)),
                (LevelId::new(3), IVec2::new(0, 110)),
            ]
        );

        assert!(arrange_linear(&levels, &order, WorldLayout::Free, 10).is_empty());
    }

    #[test]
    fn linear_order_ranks_the_dragged_level_by_its_preview() {
        let levels = vec![
            level(1, IVec2::new(0, 0), UVec2::splat(100), 0),
            level(2, IVec2::new(100, 0), UVec2::splat(100), 0),
            level(3, IVec2::new(200, 0), UVec2::splat(100), 0),
        ];

        let order = linear_order(
            &levels,
            WorldLayout::LinearHorizontal,
            LevelId::new(1),
            IVec2::new(250, 999),
        );
        assert_eq!(
            order,
            vec![LevelId::new(2), LevelId::new(3), LevelId::new(1)]
        );

        let order = linear_order(
            &levels,
            WorldLayout::LinearHorizontal,
            LevelId::new(3),
            IVec2::new(-40, 0),
        );
        assert_eq!(
            order,
            vec![LevelId::new(3), LevelId::new(1), LevelId::new(2)]
        );
    }

    #[test]
    fn axis_order_sorts_along_the_layout_axis() {
        let levels = vec![
            level(1, IVec2::new(200, 30), UVec2::splat(50), 0),
            level(2, IVec2::new(0, 90), UVec2::splat(50), 0),
            level(3, IVec2::new(100, 60), UVec2::splat(50), 0),
        ];

        assert_eq!(
            axis_order(&levels, WorldLayout::LinearHorizontal),
            vec![LevelId::new(2), LevelId::new(3), LevelId::new(1)]
        );
        assert_eq!(
            axis_order(&levels, WorldLayout::LinearVertical),
            vec![LevelId::new(1), LevelId::new(3), LevelId::new(2)]
        );
        assert_eq!(
            axis_order(&levels, WorldLayout::Free),
            vec![LevelId::new(1), LevelId::new(2), LevelId::new(3)]
        );
    }

    #[test]
    fn default_level_size_is_sixteen_cells() {
        assert_eq!(default_level_size(IVec2::splat(16)), UVec2::splat(256));
        assert_eq!(default_level_size(IVec2::new(32, 8)), UVec2::new(512, 128));
    }

    #[test]
    fn pick_prefers_the_topmost_level_at_the_active_depth() {
        let levels = vec![
            level(1, IVec2::new(0, 0), UVec2::splat(100), 0),
            level(2, IVec2::new(50, 50), UVec2::splat(100), 0),
            level(3, IVec2::new(0, 0), UVec2::splat(100), 1),
        ];
        let point = IVec2::new(60, 60);

        assert_eq!(pick_level_at(&levels, 0, point), Some(LevelId::new(2)));
        assert_eq!(pick_level_at(&levels, 1, point), Some(LevelId::new(3)));
        assert_eq!(pick_level_at(&levels, 2, point), None);
        assert_eq!(pick_level_at(&levels, 2, IVec2::new(500, 500)), None);
    }

    #[test]
    fn level_contains_uses_exclusive_max_edges() {
        let l = level(1, IVec2::new(-10, -10), UVec2::new(10, 10), 0);
        assert!(level_contains(&l, IVec2::new(-10, -10)));
        assert!(level_contains(&l, IVec2::new(-1, -1)));
        assert!(!level_contains(&l, IVec2::ZERO));
    }

    #[test]
    fn depths_are_sorted_and_include_the_active_value() {
        let world = World {
            id: lc_core::id::WorldId::new(1),
            name: "W".to_string(),
            layout: WorldLayout::Free,
            default_grid: None,
            levels: vec![level(1, IVec2::ZERO, UVec2::splat(10), 2)],
        };
        assert_eq!(depths(&world, 0), vec![0, 2]);
        assert_eq!(depths(&world, 5), vec![2, 5]);
    }

    #[test]
    fn world_bounds_cover_every_level() {
        let world = World {
            id: lc_core::id::WorldId::new(1),
            name: "W".to_string(),
            layout: WorldLayout::Free,
            default_grid: None,
            levels: vec![
                level(1, IVec2::new(-20, 10), UVec2::new(40, 40), 0),
                level(2, IVec2::new(100, -50), UVec2::new(20, 20), 0),
            ],
        };
        assert_eq!(
            world_bounds(&world),
            Some((IVec2::new(-20, -50), IVec2::new(120, 50)))
        );
    }
}
