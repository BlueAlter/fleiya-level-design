use lc_core::defs::{AnimationPlayMode, TileAnimationDef, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_core::Command;
use lc_feature_tileset::{
    AddAnimationFrameCommand, RemoveAnimationFrameCommand, RemoveTileAnimationCommand,
    SetTileAnimationCommand,
};

fn create_test_project() -> (Project, TilesetDefId) {
    let mut p = Project::new("Test");
    let ts_id = TilesetDefId(1);
    let ts = TilesetDef {
        id: ts_id,
        ..Default::default()
    };
    p.defs.tilesets.push(ts);
    (p, ts_id)
}

#[test]
fn test_set_tile_animation_apply_and_revert() {
    let (mut p, ts_id) = create_test_project();

    let mut anim = TileAnimationDef::new(10);
    anim.frames = vec![10, 11, 12];
    anim.frame_duration_ms = 120;
    anim.mode = AnimationPlayMode::PingPong;

    let mut cmd = SetTileAnimationCommand::new(ts_id, anim.clone());
    assert!(cmd.apply(&mut p).is_ok());

    let ts = p.defs.tileset(ts_id).unwrap();
    assert_eq!(ts.animations.get(&10), Some(&anim));

    cmd.revert(&mut p);
    let ts = p.defs.tileset(ts_id).unwrap();
    assert_eq!(ts.animations.get(&10), None);
}

#[test]
fn test_remove_tile_animation_apply_and_revert() {
    let (mut p, ts_id) = create_test_project();

    let anim = TileAnimationDef::new(5);
    p.defs
        .tileset_mut(ts_id)
        .unwrap()
        .animations
        .insert(5, anim.clone());

    let mut cmd = RemoveTileAnimationCommand::new(ts_id, 5);
    assert!(cmd.apply(&mut p).is_ok());

    let ts = p.defs.tileset(ts_id).unwrap();
    assert_eq!(ts.animations.get(&5), None);

    cmd.revert(&mut p);
    let ts = p.defs.tileset(ts_id).unwrap();
    assert_eq!(ts.animations.get(&5), Some(&anim));
}

#[test]
fn test_add_and_remove_animation_frame() {
    let (mut p, ts_id) = create_test_project();

    let mut cmd = AddAnimationFrameCommand::new(ts_id, 8, 9);
    assert!(cmd.apply(&mut p).is_ok());

    let ts = p.defs.tileset(ts_id).unwrap();
    let anim = ts.animations.get(&8).unwrap();
    assert_eq!(anim.frames, vec![8, 9]);

    let mut remove_cmd = RemoveAnimationFrameCommand::new(ts_id, 8, 1);
    assert!(remove_cmd.apply(&mut p).is_ok());

    let ts = p.defs.tileset(ts_id).unwrap();
    let anim = ts.animations.get(&8).unwrap();
    assert_eq!(anim.frames, vec![8]);

    remove_cmd.revert(&mut p);
    let ts = p.defs.tileset(ts_id).unwrap();
    let anim = ts.animations.get(&8).unwrap();
    assert_eq!(anim.frames, vec![8, 9]);
}
