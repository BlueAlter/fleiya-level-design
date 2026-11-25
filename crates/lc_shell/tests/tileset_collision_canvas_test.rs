//! Tests for tileset collision canvas interaction, overlay rendering, and focus/isolate mode.

use egui::{vec2, Context, Pos2, Rect};
use glam::{UVec2, Vec2};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_core::collision::{CollisionShape, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_shell::features::tileset::collision_interaction::{
    handle_collision_focus_interaction, hit_test_shapes, point_in_circle, point_in_polygon,
    point_in_rect,
};
use lc_shell::features::tileset::collision_overlay::{
    local_to_screen, render_tile_collision_overlay, screen_to_local, snap_local_point,
    CollisionOverlayContext,
};
use lc_shell::features::tileset::state::{CollisionSnapMode, CollisionTool, TilesetViewState};
use lc_shell::shared::ui::atlas_canvas::PickerView;
use lc_shell::shared::ui::tile_bounds::TileGrid;

#[test]
fn test_coordinate_conversion_roundtrip() {
    let tile_size = UVec2::new(16, 16);
    let tile_rect = Rect::from_min_size(Pos2::new(100.0, 200.0), vec2(64.0, 64.0));

    let local = Vec2::new(4.0, 8.0);
    let screen = local_to_screen(local, tile_rect, tile_size);

    assert!((screen.x - (100.0 + 16.0)).abs() < 0.001);
    assert!((screen.y - (200.0 + 32.0)).abs() < 0.001);

    let roundtrip = screen_to_local(screen, tile_rect, tile_size);
    assert!((roundtrip.x - local.x).abs() < 0.001);
    assert!((roundtrip.y - local.y).abs() < 0.001);
}

#[test]
fn test_snap_local_point_modes() {
    let tile_size = UVec2::new(16, 16);

    // Pixel snapping (1px grid, clamped to tile bounds)
    assert_eq!(
        snap_local_point(Vec2::new(3.4, 7.8), tile_size, CollisionSnapMode::Pixel),
        Vec2::new(3.0, 8.0)
    );
    assert_eq!(
        snap_local_point(Vec2::new(-2.0, 25.0), tile_size, CollisionSnapMode::Pixel),
        Vec2::new(0.0, 16.0)
    );

    // TileSize snapping (half-tile steps: 0, 8, 16 for 16px tile)
    assert_eq!(
        snap_local_point(Vec2::new(3.0, 7.0), tile_size, CollisionSnapMode::TileSize),
        Vec2::new(0.0, 8.0)
    );
    assert_eq!(
        snap_local_point(
            Vec2::new(11.0, 15.0),
            tile_size,
            CollisionSnapMode::TileSize
        ),
        Vec2::new(8.0, 16.0)
    );

    // Off/None snapping (unconstrained within tile bounds)
    let unconstrained =
        snap_local_point(Vec2::new(5.37, 12.82), tile_size, CollisionSnapMode::None);
    assert!((unconstrained.x - 5.37).abs() < 0.001);
    assert!((unconstrained.y - 12.82).abs() < 0.001);
}

#[test]
fn test_geometry_hit_testing() {
    // Rectangle
    assert!(point_in_rect(Vec2::new(5.0, 5.0), 0.0, 0.0, 10.0, 10.0));
    assert!(!point_in_rect(Vec2::new(15.0, 5.0), 0.0, 0.0, 10.0, 10.0));

    // Circle
    assert!(point_in_circle(Vec2::new(8.0, 8.0), 8.0, 8.0, 4.0));
    assert!(point_in_circle(Vec2::new(10.0, 8.0), 8.0, 8.0, 4.0));
    assert!(!point_in_circle(Vec2::new(15.0, 8.0), 8.0, 8.0, 4.0));

    // Polygon (triangle from (0, 16), (16, 16), (16, 0))
    let triangle = vec![
        Vec2::new(0.0, 16.0),
        Vec2::new(16.0, 16.0),
        Vec2::new(16.0, 0.0),
    ];
    assert!(point_in_polygon(Vec2::new(12.0, 14.0), &triangle));
    assert!(!point_in_polygon(Vec2::new(2.0, 2.0), &triangle));

    // Shape collection hit test priority (reverse order, top-most first)
    let mut col_def = TileCollisionDef::new();
    col_def
        .shapes
        .push(CollisionShape::new_rect(1, 0.0, 0.0, 16.0, 16.0));
    col_def
        .shapes
        .push(CollisionShape::new_rect(2, 4.0, 4.0, 8.0, 8.0));

    // Inside overlapping inner shape -> returns shape 2
    assert_eq!(hit_test_shapes(Vec2::new(6.0, 6.0), &col_def), Some(2));
    // Outside inner shape but inside outer shape -> returns shape 1
    assert_eq!(hit_test_shapes(Vec2::new(1.0, 1.0), &col_def), Some(1));
    // Outside both -> returns None
    assert_eq!(hit_test_shapes(Vec2::new(20.0, 20.0), &col_def), None);
}

#[test]
fn test_collision_overlay_context_capture() {
    let state = TilesetViewState {
        active_mode: "Collision".to_string(),
        collision_focus_tile: Some(5),
        selected_collision_shape_id: Some(2),
        collision_snap: CollisionSnapMode::TileSize,
        collision_tool: CollisionTool::Polygon,
        collision_poly_draft: vec![Vec2::new(0.0, 0.0), Vec2::new(8.0, 0.0)],
        ..Default::default()
    };

    let cx = CollisionOverlayContext::from_state(&state);
    assert_eq!(cx.focused_tile, Some(5));
    assert!(cx.in_collision_mode);
    assert_eq!(cx.selected_shape_id, Some(2));
    assert_eq!(cx.snap, CollisionSnapMode::TileSize);
    assert_eq!(cx.tool, CollisionTool::Polygon);
    assert_eq!(cx.poly_draft.len(), 2);
    assert_eq!(cx.drag_handle, None);
}

#[test]
fn test_hit_test_handles() {
    use lc_shell::features::tileset::collision_interaction::hit_test_handles;
    use lc_shell::features::tileset::state::CollisionHandleTarget;

    let tile_size = UVec2::new(16, 16);
    let tile_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), vec2(64.0, 64.0));

    let box_shape = CollisionShape::new_rect(1, 0.0, 0.0, 16.0, 16.0);
    assert_eq!(
        hit_test_handles(Pos2::new(2.0, 2.0), &box_shape, tile_rect, tile_size),
        Some(CollisionHandleTarget::RectCorner {
            shape_id: 1,
            corner_idx: 0
        })
    );
    assert_eq!(
        hit_test_handles(Pos2::new(63.0, 62.0), &box_shape, tile_rect, tile_size),
        Some(CollisionHandleTarget::RectCorner {
            shape_id: 1,
            corner_idx: 2
        })
    );
    assert_eq!(
        hit_test_handles(Pos2::new(32.0, 32.0), &box_shape, tile_rect, tile_size),
        None
    );

    let poly_shape = CollisionShape::slope_up(2, UVec2::new(16, 16));
    assert_eq!(
        hit_test_handles(Pos2::new(64.0, 1.0), &poly_shape, tile_rect, tile_size),
        Some(CollisionHandleTarget::PolygonVertex {
            shape_id: 2,
            vertex_idx: 1
        })
    );
}

#[test]
fn test_collision_focus_interaction_box_creation() {
    let ctx = Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 400.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let mut state = TilesetViewState {
            active_mode: "Collision".to_string(),
            collision_focus_tile: Some(0),
            collision_tool: CollisionTool::Box,
            collision_snap: CollisionSnapMode::Pixel,
            collision_drag_start: Some(Vec2::new(2.0, 2.0)),
            collision_drag_current: Some(Vec2::new(10.0, 12.0)),
            ..Default::default()
        };

        let tileset = TilesetDef {
            id: TilesetDefId(1),
            name: "TestTileset".to_string(),
            path: "test.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: vec![],
            marking: Default::default(),
            custom_data: Default::default(),
            animations: Default::default(),
            collisions: Default::default(),
        };

        let grid = TileGrid {
            cols: 4,
            rows: 4,
            tile_w: 16,
            tile_h: 16,
            spacing: 0,
            padding: 0,
        };

        let canvas_view = PickerView {
            zoom: 1.0,
            offset: vec2(0.0, 0.0),
        };

        // Allocate a dummy response
        let (_, response) =
            ui.allocate_exact_size(vec2(100.0, 100.0), egui::Sense::click_and_drag());

        let cmds = handle_collision_focus_interaction(
            ui,
            &response,
            &tileset,
            &grid,
            canvas_view,
            &mut state,
        );

        // Pointer is not down -> drag completes and emits SetTileCollisionCommand
        assert_eq!(cmds.len(), 1);
        assert_eq!(state.selected_collision_shape_id, Some(1));
    });
    output.textures_delta.clear();
}

#[test]
fn test_collision_overlay_headless_rendering() {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let t = Tokens::get(&ctx);

    let raw_input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 400.0))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        let state = TilesetViewState {
            active_mode: "Collision".to_string(),
            collision_focus_tile: Some(0),
            collision_snap: CollisionSnapMode::Pixel,
            show_collision_overlay: true,
            ..Default::default()
        };

        let mut tileset = TilesetDef {
            id: TilesetDefId(1),
            name: "TestTileset".to_string(),
            path: "test.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: vec![],
            marking: Default::default(),
            custom_data: Default::default(),
            animations: Default::default(),
            collisions: Default::default(),
        };

        // Add shapes: one box, one polygon, one slope
        let mut col_def = TileCollisionDef::new();
        col_def
            .shapes
            .push(CollisionShape::new_rect(1, 0.0, 0.0, 16.0, 16.0));
        let mut one_way_shape = CollisionShape::new_rect(2, 0.0, 0.0, 16.0, 4.0);
        one_way_shape.one_way = true;
        col_def.shapes.push(one_way_shape);
        col_def
            .shapes
            .push(CollisionShape::slope_up(3, UVec2::new(16, 16)));
        tileset.collisions.insert(0, col_def);

        let tile_rect = Rect::from_min_size(Pos2::new(10.0, 10.0), vec2(64.0, 64.0));
        let cx = CollisionOverlayContext::from_state(&state);

        // Render overlay on focused tile
        render_tile_collision_overlay(ui.painter(), tile_rect, 0, &tileset, &cx, &t);

        // Render overlay on unfocused tile
        render_tile_collision_overlay(ui.painter(), tile_rect, 1, &tileset, &cx, &t);
    });
    output.textures_delta.clear();
}
