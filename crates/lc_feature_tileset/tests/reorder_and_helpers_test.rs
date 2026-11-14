use glam::UVec2;
use lc_core::command::Command;
use lc_core::defs::{EnumDef, EnumValueDef, LayerDef, TilesetDef};
use lc_core::id::{EnumDefId, LayerDefId, TilesetDefId};
use lc_core::layer::{LayerKind, TileRef};
use lc_core::project::Project;
use lc_feature_tileset::{
    compute_tileset_grid, generate_unique_tileset_name, mode_availability, tileset_usage_counts,
    GridMetrics, ReorderTilesetDefCommand,
};

#[test]
fn test_reorder_tileset_def_command_apply_revert() {
    let mut p = Project::new("Test");
    for i in 1..=4 {
        p.defs.tilesets.push(TilesetDef {
            id: TilesetDefId(i),
            name: format!("TS_{i}"),
            path: format!("assets/ts_{i}.png"),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
        });
    }

    let original_order: Vec<TilesetDefId> = p.defs.tilesets.iter().map(|t| t.id).collect();
    assert_eq!(
        original_order,
        vec![
            TilesetDefId(1),
            TilesetDefId(2),
            TilesetDefId(3),
            TilesetDefId(4)
        ]
    );

    // Move TilesetDefId(1) from index 0 to index 2
    let mut cmd = ReorderTilesetDefCommand::new(TilesetDefId(1), 2);
    assert_eq!(cmd.tileset_id(), TilesetDefId(1));
    assert_eq!(cmd.new_index(), 2);
    assert!(!cmd.merge(&ReorderTilesetDefCommand::new(TilesetDefId(2), 0)));

    cmd.apply(&mut p).expect("apply should succeed");
    let after_apply: Vec<TilesetDefId> = p.defs.tilesets.iter().map(|t| t.id).collect();
    assert_eq!(
        after_apply,
        vec![
            TilesetDefId(2),
            TilesetDefId(3),
            TilesetDefId(1),
            TilesetDefId(4)
        ]
    );

    // Revert restores the exact order
    cmd.revert(&mut p);
    let after_revert: Vec<TilesetDefId> = p.defs.tilesets.iter().map(|t| t.id).collect();
    assert_eq!(after_revert, original_order);
}

#[test]
fn test_reorder_tileset_def_command_out_of_range() {
    let mut p = Project::new("Test");
    p.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "TS_1".to_string(),
        path: "assets/ts_1.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });

    let mut cmd_out_of_range = ReorderTilesetDefCommand::new(TilesetDefId(1), 5);
    assert!(cmd_out_of_range.apply(&mut p).is_err());

    let mut cmd_not_found = ReorderTilesetDefCommand::new(TilesetDefId(999), 0);
    assert!(cmd_not_found.apply(&mut p).is_err());
}

#[test]
fn test_compute_tileset_grid_standard() {
    // 256x192 with 16x16 tiles, 0 spacing, 0 padding
    let grid = compute_tileset_grid([256, 192], UVec2::new(16, 16), 0, 0);
    assert_eq!(
        grid,
        GridMetrics {
            cols: 16,
            rows: 12,
            total_tiles: 192,
            leftover_x: 0,
            leftover_y: 0,
        }
    );
}

#[test]
fn test_compute_tileset_grid_with_spacing_and_padding_and_leftover() {
    // 100x80 with 16x16 tiles, 2 spacing, 4 padding
    // eff_w = 100 - 8 = 92. stride_x = 18. (92 + 2) / 18 = 94 / 18 = 5 cols.
    // used_w = 4*2 + 5*16 + 4*2 = 8 + 80 + 8 = 96. leftover_x = 100 - 96 = 4.
    // eff_h = 80 - 8 = 72. stride_y = 18. (72 + 2) / 18 = 74 / 18 = 4 rows.
    // used_h = 4*2 + 4*16 + 3*2 = 8 + 64 + 6 = 78. leftover_y = 80 - 78 = 2.
    let grid = compute_tileset_grid([100, 80], UVec2::new(16, 16), 2, 4);
    assert_eq!(
        grid,
        GridMetrics {
            cols: 5,
            rows: 4,
            total_tiles: 20,
            leftover_x: 4,
            leftover_y: 2,
        }
    );
}

#[test]
fn test_compute_tileset_grid_zero_and_tiny() {
    // Zero tile size
    let g_zero = compute_tileset_grid([100, 100], UVec2::new(0, 0), 0, 0);
    assert_eq!(g_zero.total_tiles, 0);
    assert_eq!(g_zero.leftover_x, 100);

    // Image smaller than padding
    let g_tiny = compute_tileset_grid([5, 5], UVec2::new(16, 16), 0, 10);
    assert_eq!(g_tiny.cols, 0);
    assert_eq!(g_tiny.rows, 0);
    assert_eq!(g_tiny.total_tiles, 0);
}

#[test]
fn test_tileset_usage_counts() {
    let mut p = Project::new("Test");
    let ts1 = TilesetDefId(1);
    let ts2 = TilesetDefId(2);

    p.defs.layers.push(LayerDef {
        id: LayerDefId(10),
        name: "L1".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(ts1),
        int_grid_values: Vec::new(),
    });
    p.defs.layers.push(LayerDef {
        id: LayerDefId(11),
        name: "L2".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(ts1),
        int_grid_values: Vec::new(),
    });

    p.defs.enums.push(EnumDef {
        id: EnumDefId(20),
        name: "ItemType".to_string(),
        tileset_id: Some(ts1),
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Sword".to_string(),
            name: "Sword".to_string(),
            color: [255, 0, 0, 255],
            tile: Some(TileRef {
                tileset_id: ts1,
                tile_id: 0,
                flip_x: false,
                flip_y: false,
            }),
        }],
    });

    let (layers1, enums1) = tileset_usage_counts(&p, ts1);
    assert_eq!(layers1, 2);
    assert_eq!(enums1, 1);

    let (layers2, enums2) = tileset_usage_counts(&p, ts2);
    assert_eq!(layers2, 0);
    assert_eq!(enums2, 0);
}

#[test]
fn test_mode_availability() {
    let ts = TilesetDef {
        id: TilesetDefId(1),
        name: "World".to_string(),
        path: "world.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    };
    let modes = mode_availability(&ts);
    assert_eq!(modes.len(), 3);
    assert_eq!(modes[0].id, "None");
    assert!(modes[0].available);
    assert_eq!(modes[1].id, "Marking");
    assert!(!modes[1].available);
    assert_eq!(modes[2].id, "Animation");
    assert!(!modes[2].available);
}

#[test]
fn test_generate_unique_tileset_name() {
    let names = vec!["Dungeon".to_string(), "Dungeon copy".to_string()];
    assert_eq!(generate_unique_tileset_name(&names, "Forest"), "Forest");
    assert_eq!(
        generate_unique_tileset_name(&names, "Dungeon"),
        "Dungeon copy 2"
    );
    assert_eq!(
        generate_unique_tileset_name(&names, "Dungeon copy"),
        "Dungeon copy 2"
    );

    let names_with_2 = vec![
        "Dungeon".to_string(),
        "Dungeon copy".to_string(),
        "Dungeon copy 2".to_string(),
    ];
    assert_eq!(
        generate_unique_tileset_name(&names_with_2, "Dungeon"),
        "Dungeon copy 3"
    );
}
