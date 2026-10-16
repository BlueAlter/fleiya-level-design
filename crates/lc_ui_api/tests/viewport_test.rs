use glam::IVec2;
use lc_ui_api::viewport::{Viewport, DEFAULT_MAX_ZOOM, DEFAULT_MIN_ZOOM};

#[test]
fn test_viewport_defaults_and_zoom_limits() {
    let mut vp = Viewport::default();
    assert_eq!(vp.zoom, 1.0);
    assert_eq!(vp.min_zoom, DEFAULT_MIN_ZOOM);
    assert_eq!(vp.max_zoom, DEFAULT_MAX_ZOOM);

    // Zoom within limits
    vp.set_zoom(2.5);
    assert_eq!(vp.zoom, 2.5);

    // Clamping to lower bound
    vp.set_zoom(0.001);
    assert_eq!(vp.zoom, DEFAULT_MIN_ZOOM);

    // Clamping to upper bound
    vp.set_zoom(100.0);
    assert_eq!(vp.zoom, DEFAULT_MAX_ZOOM);

    // zoom_at clamping
    let pivot = egui::pos2(100.0, 100.0);
    vp.zoom_at(0.0001, pivot);
    assert_eq!(vp.zoom, DEFAULT_MIN_ZOOM);

    vp.zoom_at(10000.0, pivot);
    assert_eq!(vp.zoom, DEFAULT_MAX_ZOOM);
}

#[test]
fn test_screen_world_conversion_positive_and_negative() {
    let mut vp = Viewport::new(egui::Rect::from_min_size(
        egui::pos2(50.0, 100.0),
        egui::vec2(800.0, 600.0),
    ));
    vp.pan = egui::vec2(-120.0, 80.0);
    vp.set_zoom(2.0);

    // Test positive world coordinate
    let world_pos = IVec2::new(150, 220);
    let screen_pos = vp.world_to_screen(world_pos);
    let recovered_world = vp.screen_to_world(screen_pos);
    assert_eq!(recovered_world, world_pos);

    // Test negative world coordinates
    let neg_world_pos = IVec2::new(-350, -420);
    let neg_screen_pos = vp.world_to_screen(neg_world_pos);
    let recovered_neg = vp.screen_to_world(neg_screen_pos);
    assert_eq!(recovered_neg, neg_world_pos);

    // Test origin (0, 0)
    let origin = IVec2::ZERO;
    let origin_screen = vp.world_to_screen(origin);
    assert_eq!(vp.screen_to_world(origin_screen), origin);
}

#[test]
fn test_world_grid_conversion_negative_and_positive() {
    let level_pos = IVec2::new(100, 200);
    let grid_size = IVec2::new(16, 16);

    // Positive grid coordinates
    let grid = IVec2::new(5, 7);
    let world_top_left = Viewport::grid_to_world(grid, level_pos, grid_size);
    assert_eq!(world_top_left, IVec2::new(100 + 5 * 16, 200 + 7 * 16));

    // Exact top-left pixel
    assert_eq!(
        Viewport::world_to_grid(world_top_left, level_pos, grid_size),
        grid
    );
    // Inside cell pixel
    assert_eq!(
        Viewport::world_to_grid(world_top_left + IVec2::new(15, 15), level_pos, grid_size),
        grid
    );
    // Next cell pixel
    assert_eq!(
        Viewport::world_to_grid(world_top_left + IVec2::new(16, 0), level_pos, grid_size),
        grid + IVec2::new(1, 0)
    );

    // Negative grid coordinates: cell (-1, -1)
    let neg_grid = IVec2::new(-1, -1);
    let neg_world_top_left = Viewport::grid_to_world(neg_grid, level_pos, grid_size);
    assert_eq!(neg_world_top_left, IVec2::new(100 - 16, 200 - 16));

    // Top-left pixel of (-1, -1)
    assert_eq!(
        Viewport::world_to_grid(neg_world_top_left, level_pos, grid_size),
        neg_grid
    );
    // Interior pixel of (-1, -1)
    assert_eq!(
        Viewport::world_to_grid(neg_world_top_left + IVec2::new(8, 8), level_pos, grid_size),
        neg_grid
    );
    // Bottom-right edge pixel of (-1, -1)
    assert_eq!(
        Viewport::world_to_grid(
            neg_world_top_left + IVec2::new(15, 15),
            level_pos,
            grid_size
        ),
        neg_grid
    );

    // Deep negative grid: (-10, -25)
    let deep_neg_grid = IVec2::new(-10, -25);
    let deep_world = Viewport::grid_to_world(deep_neg_grid, level_pos, grid_size);
    assert_eq!(
        Viewport::world_to_grid(deep_world, level_pos, grid_size),
        deep_neg_grid
    );
}

#[test]
fn test_zoom_at_pivot_preserves_world_position() {
    let mut vp = Viewport::new(egui::Rect::from_min_size(
        egui::pos2(100.0, 50.0),
        egui::vec2(600.0, 400.0),
    ));
    vp.pan = egui::vec2(-50.0, 30.0);
    vp.set_zoom(1.5);

    let pivot = egui::pos2(350.0, 250.0);
    let world_before = vp.screen_to_world(pivot);

    // Zoom in
    vp.zoom_at(2.0, pivot);
    let world_after = vp.screen_to_world(pivot);
    assert_eq!(world_before, world_after);

    // Zoom out
    vp.zoom_at(0.5, pivot);
    let world_after_out = vp.screen_to_world(pivot);
    assert_eq!(world_before, world_after_out);
}

#[test]
fn test_screen_to_grid_and_grid_to_screen() {
    let mut vp = Viewport::new(egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(1000.0, 1000.0),
    ));
    vp.set_zoom(1.0);
    vp.pan = egui::Vec2::ZERO;

    let level_pos = IVec2::new(0, 0);
    let grid_size = IVec2::new(16, 16);

    let rect = vp.grid_to_screen(IVec2::new(2, 3), level_pos, grid_size);
    assert_eq!(rect.min, egui::pos2(32.0, 48.0));
    assert_eq!(rect.max, egui::pos2(48.0, 64.0));

    let cell = vp.screen_to_grid(egui::pos2(35.0, 50.0), level_pos, grid_size);
    assert_eq!(cell, IVec2::new(2, 3));
}
