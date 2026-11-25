//! Integration tests for collision commands in `lc_feature_tileset`.

use glam::UVec2;
use lc_core::command::Command;
use lc_core::defs::{CollisionShape, CollisionShapeGeometry, TileCollisionDef, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_feature_tileset::{
    BatchSetTileCollisionCommand, RemoveTileCollisionCommand, SetTileCollisionCommand,
};

fn make_test_project() -> Project {
    let mut p = Project::new("CollisionCommandTest");
    let ts = TilesetDef::new(
        TilesetDefId(1),
        "DungeonTiles",
        "dungeon.png",
        UVec2::new(16, 16),
    );
    p.defs.tilesets.push(ts);
    p
}

#[test]
fn test_set_tile_collision_apply_and_revert() {
    let mut p = make_test_project();
    let ts_id = TilesetDefId(1);

    // 1. Initially no collision
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(5));

    // 2. Set full-tile box collision on tile 5
    let col = TileCollisionDef::full_tile(UVec2::new(16, 16));
    let mut cmd = SetTileCollisionCommand::new(ts_id, 5, col.clone());
    cmd.apply(&mut p).expect("apply should succeed");

    assert!(p.defs.tileset(ts_id).unwrap().has_collision(5));
    let stored = p.defs.tileset(ts_id).unwrap().collision(5).unwrap();
    assert_eq!(stored.shapes.len(), 1);
    assert_eq!(stored.shapes[0].tag, "solid");

    // 3. Revert
    cmd.revert(&mut p);
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(5));

    // 4. Re-apply
    cmd.apply(&mut p).expect("re-apply should succeed");
    assert!(p.defs.tileset(ts_id).unwrap().has_collision(5));
}

#[test]
fn test_set_tile_collision_merge() {
    let mut p = make_test_project();
    let ts_id = TilesetDefId(1);

    let col1 = TileCollisionDef::full_tile(UVec2::new(16, 16));
    let mut cmd1 = SetTileCollisionCommand::new(ts_id, 3, col1);
    cmd1.apply(&mut p).unwrap();

    let mut col2 = TileCollisionDef::new();
    col2.shapes
        .push(CollisionShape::slope_up(1, UVec2::new(16, 16)));
    let cmd2 = SetTileCollisionCommand::new(ts_id, 3, col2);

    // Merge cmd2 into cmd1
    assert!(cmd1.merge(&cmd2));
    assert_eq!(cmd1.collision().shapes.len(), 1);
    match &cmd1.collision().shapes[0].geometry {
        CollisionShapeGeometry::Polygon { points } => assert_eq!(points.len(), 3),
        _ => panic!("Expected polygon geometry after merge"),
    }
}

#[test]
fn test_remove_tile_collision_apply_and_revert() {
    let mut p = make_test_project();
    let ts_id = TilesetDefId(1);

    // Populate tile 7 with collision
    let col = TileCollisionDef::full_tile(UVec2::new(16, 16));
    p.defs.tileset_mut(ts_id).unwrap().collisions.insert(7, col);
    assert!(p.defs.tileset(ts_id).unwrap().has_collision(7));

    // Remove collision
    let mut rm_cmd = RemoveTileCollisionCommand::new(ts_id, 7);
    rm_cmd.apply(&mut p).unwrap();
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(7));

    // Revert removal
    rm_cmd.revert(&mut p);
    assert!(p.defs.tileset(ts_id).unwrap().has_collision(7));
}

#[test]
fn test_batch_set_tile_collision_multi_paste_and_revert() {
    let mut p = make_test_project();
    let ts_id = TilesetDefId(1);

    // Tile 1 has an existing collision (slope)
    let old_slope = TileCollisionDef {
        shapes: vec![CollisionShape::slope_up(1, UVec2::new(16, 16))],
    };
    p.defs
        .tileset_mut(ts_id)
        .unwrap()
        .collisions
        .insert(1, old_slope.clone());

    // Tile 2 and Tile 3 are empty
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(2));
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(3));

    // Paste a new box collision onto tiles [1, 2, 3]
    let new_box = TileCollisionDef::full_tile(UVec2::new(16, 16));
    let mut batch_cmd = BatchSetTileCollisionCommand::new(ts_id, vec![1, 2, 3], new_box.clone());
    batch_cmd.apply(&mut p).unwrap();

    // All 3 tiles now have the new collision
    for tile in [1, 2, 3] {
        assert!(p.defs.tileset(ts_id).unwrap().has_collision(tile));
        assert_eq!(
            p.defs
                .tileset(ts_id)
                .unwrap()
                .collision(tile)
                .unwrap()
                .shapes[0]
                .geometry,
            new_box.shapes[0].geometry
        );
    }

    // Revert the batch paste
    batch_cmd.revert(&mut p);

    // Tile 1 restored to old slope
    assert_eq!(
        p.defs.tileset(ts_id).unwrap().collision(1).unwrap(),
        &old_slope
    );
    // Tiles 2 and 3 restored to empty
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(2));
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(3));
}

#[test]
fn test_collision_shape_with_custom_data_and_tags() {
    let mut p = make_test_project();
    let ts_id = TilesetDefId(1);

    let mut shape = CollisionShape::new_rect(10, 0.0, 8.0, 16.0, 8.0);
    shape.tag = "Hazard".to_string();
    shape.one_way = true;
    shape.custom_data = "damage=50;elem=fire".to_string();

    let col = TileCollisionDef {
        shapes: vec![shape],
    };

    let mut cmd = SetTileCollisionCommand::new(ts_id, 9, col.clone());
    cmd.apply(&mut p).unwrap();

    let ts = p.defs.tileset(ts_id).unwrap();
    let stored = ts.collision(9).unwrap();
    assert_eq!(stored.shapes.len(), 1);
    assert_eq!(stored.shapes[0].tag, "Hazard");
    assert!(stored.shapes[0].one_way);
    assert_eq!(stored.shapes[0].custom_data, "damage=50;elem=fire");

    cmd.revert(&mut p);
    assert!(!p.defs.tileset(ts_id).unwrap().has_collision(9));
}
