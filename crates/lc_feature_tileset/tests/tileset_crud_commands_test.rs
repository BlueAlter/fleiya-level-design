use glam::UVec2;
use lc_core::command::{Command, DocKind};
use lc_core::defs::{EnumDef, EnumValueDef, LayerDef, TilesetDef};
use lc_core::grid::Grid;
use lc_core::id::{IdGenerator, TilesetDefId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind, TileRef};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_tileset::{AddTilesetDefCommand, RemoveTilesetDefCommand, UpdateTilesetDefCommand};

fn setup_test_project() -> (Project, TilesetDefId, IdGenerator) {
    let mut gen = IdGenerator::new();
    let tileset_id = gen.next_tileset_def_id();
    let tileset = TilesetDef {
        id: tileset_id,
        name: "Dungeon".to_string(),
        path: "tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 1,
        padding: 2,
        tags: Vec::new(),
        marking: lc_core::defs::TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    };

    let mut project = Project::new("Test Project");
    project.defs.tilesets.push(tileset);
    (project, tileset_id, gen)
}

#[test]
fn test_add_tileset_def_command_apply_and_revert() {
    let (mut project, existing_id, mut gen) = setup_test_project();
    let new_id = gen.next_tileset_def_id();
    let new_def = TilesetDef {
        id: new_id,
        name: "Overworld".to_string(),
        path: "tiles/overworld.png".to_string(),
        tile_size: UVec2::new(32, 32),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: lc_core::defs::TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    };

    let mut cmd = AddTilesetDefCommand::new(new_def.clone());
    assert_eq!(cmd.focus(), Some(DocKind::TilesetDef(new_id)));

    // Apply
    cmd.apply(&mut project).expect("apply should succeed");
    assert_eq!(project.defs.tilesets.len(), 2);
    assert_eq!(project.defs.tileset(new_id), Some(&new_def));

    // Duplicate apply should fail
    let mut dup_cmd = AddTilesetDefCommand::new(new_def.clone());
    assert!(dup_cmd.apply(&mut project).is_err());

    // Revert
    cmd.revert(&mut project);
    assert_eq!(project.defs.tilesets.len(), 1);
    assert!(project.defs.tileset(new_id).is_none());
    assert!(project.defs.tileset(existing_id).is_some());
}

#[test]
fn test_update_tileset_def_command_apply_revert_merge() {
    let (mut project, tileset_id, _) = setup_test_project();
    let mut updated = project.defs.tileset(tileset_id).unwrap().clone();
    updated.name = "Dungeon Modified".to_string();
    updated.tile_size = UVec2::new(24, 24);

    let mut cmd1 = UpdateTilesetDefCommand::new(updated.clone());
    cmd1.apply(&mut project).expect("apply should succeed");
    assert_eq!(
        project.defs.tileset(tileset_id).unwrap().name,
        "Dungeon Modified"
    );
    assert_eq!(
        project.defs.tileset(tileset_id).unwrap().tile_size,
        UVec2::new(24, 24)
    );

    // Merge with consecutive edit
    let mut updated2 = updated.clone();
    updated2.name = "Dungeon Final".to_string();
    let cmd2 = UpdateTilesetDefCommand::new(updated2);

    assert!(cmd1.merge(&cmd2));
    assert_eq!(cmd1.updated().name, "Dungeon Final");

    // Revert restores initial state before cmd1
    cmd1.revert(&mut project);
    assert_eq!(project.defs.tileset(tileset_id).unwrap().name, "Dungeon");
    assert_eq!(
        project.defs.tileset(tileset_id).unwrap().tile_size,
        UVec2::new(16, 16)
    );
}

#[test]
fn test_remove_tileset_def_command_unreferenced() {
    let (mut project, tileset_id, _) = setup_test_project();
    let mut cmd = RemoveTilesetDefCommand::new(tileset_id);

    cmd.apply(&mut project).expect("apply should succeed");
    assert!(project.defs.tileset(tileset_id).is_none());
    assert!(project.defs.tilesets.is_empty());

    cmd.revert(&mut project);
    assert_eq!(project.defs.tilesets.len(), 1);
    assert_eq!(project.defs.tileset(tileset_id).unwrap().name, "Dungeon");
}

#[test]
fn test_remove_tileset_def_command_cascade_and_revert() {
    let (mut project, tileset_id, mut gen) = setup_test_project();

    // 1. LayerDef reference
    let layer_def_id = gen.next_layer_def_id();
    project.defs.layers.push(LayerDef {
        id: layer_def_id,
        name: "Foreground".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(tileset_id),
        int_grid_values: Vec::new(),
    });

    // 2. EnumDef reference
    let enum_id = gen.next_enum_def_id();
    project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "Biomes".to_string(),
        tileset_id: Some(tileset_id),
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Swamp".to_string(),
            name: "Swamp".to_string(),
            color: [50, 150, 50, 255],
            tile: Some(TileRef::new(tileset_id, 42)),
        }],
    });

    // 3. Tile layer in a level
    let world_id = gen.next_world_id();
    let level_id = gen.next_level_id();
    let mut tile_grid = Grid::new(2, 2, None);
    tile_grid
        .set(1, 1, Some(TileRef::new(tileset_id, 99)))
        .unwrap();

    let mut level = Level::new(level_id, "Level_0".to_string(), UVec2::new(32, 32));
    level.layers.push(LayerInstance {
        def_id: layer_def_id,
        visible: true,
        data: LayerData::Tiles(tile_grid),
    });

    let mut world = World::new(world_id, "World_0".to_string());
    world.levels.push(level);
    project.worlds.push(world);

    // Verify references exist
    assert!(RemoveTilesetDefCommand::is_referenced(&project, tileset_id));

    // Blocking remove (without cascade) must fail
    let mut blocking_cmd = RemoveTilesetDefCommand::with_cascade(tileset_id, false);
    assert!(blocking_cmd.apply(&mut project).is_err());

    // Cascading remove must succeed and clear references
    let mut cascade_cmd = RemoveTilesetDefCommand::new(tileset_id);
    cascade_cmd
        .apply(&mut project)
        .expect("cascade apply should succeed");

    assert!(project.defs.tileset(tileset_id).is_none());
    assert_eq!(project.defs.layer(layer_def_id).unwrap().tileset_id, None);
    assert_eq!(project.defs.enums[0].values[0].tile, None);
    let level_layer = &project.worlds[0].levels[0].layers[0];
    if let LayerData::Tiles(ref g) = level_layer.data {
        assert_eq!(g.get(1, 1), Some(&None));
    } else {
        panic!("expected Tiles layer");
    }

    // Revert must restore EVERYTHING identically
    cascade_cmd.revert(&mut project);

    assert!(project.defs.tileset(tileset_id).is_some());
    assert_eq!(
        project.defs.layer(layer_def_id).unwrap().tileset_id,
        Some(tileset_id)
    );
    assert_eq!(
        project.defs.enums[0].values[0].tile,
        Some(TileRef::new(tileset_id, 42))
    );
    let level_layer = &project.worlds[0].levels[0].layers[0];
    if let LayerData::Tiles(ref g) = level_layer.data {
        assert_eq!(g.get(1, 1), Some(&Some(TileRef::new(tileset_id, 99))));
    } else {
        panic!("expected Tiles layer");
    }
}
