use glam::UVec2;
use lc_core::command::Command;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_feature_tileset::SetTileCustomDataCommand;

fn make_test_project() -> Project {
    let mut p = Project::new("CustomDataTest");
    let ts = TilesetDef::new(
        TilesetDefId(1),
        "GroundTiles",
        "ground.png",
        UVec2::new(16, 16),
    );
    p.defs.tilesets.push(ts);
    p
}

#[test]
fn test_set_tile_custom_data_apply_and_revert() {
    let mut p = make_test_project();

    // 1. Set custom data for tile 5
    let mut cmd1 = SetTileCustomDataCommand::new(
        TilesetDefId(1),
        5,
        Some("{\"collidable\": true}".to_string()),
    );
    cmd1.apply(&mut p).expect("should succeed");

    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&5)
            .map(|s| s.as_str()),
        Some("{\"collidable\": true}")
    );

    // 2. Revert command
    cmd1.revert(&mut p);
    assert!(!p
        .defs
        .tileset(TilesetDefId(1))
        .unwrap()
        .custom_data
        .contains_key(&5));

    // 3. Re-apply
    cmd1.apply(&mut p).expect("re-apply should succeed");
    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&5)
            .map(|s| s.as_str()),
        Some("{\"collidable\": true}")
    );

    // 4. Update custom data for the same tile
    let mut cmd2 = SetTileCustomDataCommand::new(
        TilesetDefId(1),
        5,
        Some("{\"collidable\": false, \"damage\": 10}".to_string()),
    );
    cmd2.apply(&mut p).expect("update should succeed");
    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&5)
            .map(|s| s.as_str()),
        Some("{\"collidable\": false, \"damage\": 10}")
    );

    // 5. Revert update restores original data
    cmd2.revert(&mut p);
    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&5)
            .map(|s| s.as_str()),
        Some("{\"collidable\": true}")
    );

    // 6. Clear custom data by passing None or empty string
    let mut cmd3 = SetTileCustomDataCommand::new(TilesetDefId(1), 5, None::<String>);
    cmd3.apply(&mut p).expect("clear should succeed");
    assert!(!p
        .defs
        .tileset(TilesetDefId(1))
        .unwrap()
        .custom_data
        .contains_key(&5));

    // Reverting clear restores data
    cmd3.revert(&mut p);
    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&5)
            .map(|s| s.as_str()),
        Some("{\"collidable\": true}")
    );
}

#[test]
fn test_set_tile_custom_data_merge() {
    let mut p = make_test_project();

    let mut cmd1 = SetTileCustomDataCommand::new(TilesetDefId(1), 10, Some("alpha".to_string()));
    cmd1.apply(&mut p).expect("apply cmd1");

    let cmd2 = SetTileCustomDataCommand::new(TilesetDefId(1), 10, Some("beta".to_string()));
    assert!(cmd1.merge(&cmd2), "merge should succeed for same tile");

    // After merge, new_data should be "beta"
    assert_eq!(cmd1.new_data(), Some("beta"));

    // Applying cmd1 again (as in redo/merging)
    cmd1.apply(&mut p).expect("apply merged");
    assert_eq!(
        p.defs
            .tileset(TilesetDefId(1))
            .unwrap()
            .custom_data
            .get(&10)
            .map(|s| s.as_str()),
        Some("beta")
    );

    // Merge for different tile fails
    let cmd3 = SetTileCustomDataCommand::new(TilesetDefId(1), 11, Some("gamma".to_string()));
    assert!(!cmd1.merge(&cmd3), "merge should fail for different tile");
}

#[test]
fn test_set_tile_custom_data_not_found() {
    let mut p = make_test_project();
    let mut cmd = SetTileCustomDataCommand::new(TilesetDefId(999), 0, Some("data".to_string()));
    let res = cmd.apply(&mut p);
    assert!(res.is_err(), "should return error when tileset not found");
}
