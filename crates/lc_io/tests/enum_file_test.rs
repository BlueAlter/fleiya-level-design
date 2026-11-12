use std::fs;

use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::layer::TileRef;
use lc_io::enum_file::{
    convert_file_enum_to_def, convert_project_enums_to_file, hex_to_int, int_to_hex, int_to_rgba,
    read_enum_file, rect_to_tile_id, rgba_to_int, tile_id_to_rect, validate_enum_file_content,
    write_enum_file_atomic, FleiyaEnumDef, FleiyaEnumValueDef, FleiyaTileRect,
    CURRENT_ENUM_FILE_VERSION, ENUM_FILE_FORMAT,
};

fn create_sample_enums() -> (Vec<EnumDef>, Vec<TilesetDef>) {
    let tilesets = vec![
        TilesetDef {
            id: TilesetDefId(1),
            name: "Overworld".to_string(),
            path: "tilesets/overworld.png".to_string(),
            tile_size: glam::UVec2::new(16, 16),
            padding: 0,
            spacing: 0,
        },
        TilesetDef {
            id: TilesetDefId(2),
            name: "Dungeon".to_string(),
            path: "tilesets/dungeon.png".to_string(),
            tile_size: glam::UVec2::new(16, 16),
            padding: 0,
            spacing: 0,
        },
    ];

    let enums = vec![
        EnumDef {
            id: EnumDefId(10),
            name: "BiomeType".to_string(),
            tileset_id: Some(TilesetDefId(1)),
            tags: Vec::new(),
            values: vec![
                EnumValueDef {
                    id: "Plains".to_string(),
                    name: "Plains".to_string(),
                    color: [255, 193, 77, 255],
                    tile: Some(TileRef::new(TilesetDefId(1), 5)),
                },
                EnumValueDef {
                    id: "Forest".to_string(),
                    name: "Forest".to_string(),
                    color: [61, 214, 184, 255],
                    tile: Some(TileRef::new(TilesetDefId(1), 8)),
                },
            ],
        },
        EnumDef {
            id: EnumDefId(20),
            name: "ItemRarity".to_string(),
            tileset_id: Some(TilesetDefId(2)),
            tags: Vec::new(),
            values: vec![
                EnumValueDef {
                    id: "Common".to_string(),
                    name: "Common".to_string(),
                    color: [156, 163, 175, 255],
                    tile: None,
                },
                EnumValueDef {
                    id: "Rare".to_string(),
                    name: "Rare".to_string(),
                    color: [45, 129, 255, 255],
                    tile: Some(TileRef::new(TilesetDefId(2), 12)),
                },
                EnumValueDef {
                    id: "Epic".to_string(),
                    name: "Epic".to_string(),
                    color: [168, 85, 247, 255],
                    tile: None,
                },
            ],
        },
    ];

    (enums, tilesets)
}

#[test]
fn test_save_load_roundtrip_order() {
    let (enums, tilesets) = create_sample_enums();
    let resolver = |_| Some((256, 256));
    let file_data = convert_project_enums_to_file(&enums, &tilesets, None, &resolver);

    let tmp_dir = std::env::temp_dir().join("lc_test_enum_roundtrip");
    let _ = fs::create_dir_all(&tmp_dir);
    let file_path = tmp_dir.join("test_project.fleiyaenum");

    write_enum_file_atomic(&file_path, &file_data).expect("Failed to write enum file");
    let loaded = read_enum_file(&file_path).expect("Failed to read enum file");

    assert_eq!(loaded.format, ENUM_FILE_FORMAT);
    assert_eq!(loaded.version, CURRENT_ENUM_FILE_VERSION);
    assert_eq!(loaded.enums.len(), enums.len());

    // Check exact enum order preserved
    for (i, original) in enums.iter().enumerate() {
        let loaded_enum = &loaded.enums[i];
        assert_eq!(loaded_enum.identifier, original.name);
        assert_eq!(loaded_enum.values.len(), original.values.len());

        // Check exact value order preserved
        for (j, orig_val) in original.values.iter().enumerate() {
            let loaded_val = &loaded_enum.values[j];
            assert_eq!(loaded_val.id, orig_val.id);
        }
    }

    // Convert back into defs
    let mut restored = Vec::new();
    for fe in &loaded.enums {
        restored.push(convert_file_enum_to_def(
            fe,
            EnumDefId(fe.uid as u64),
            &tilesets,
            &resolver,
        ));
    }
    assert_eq!(restored, enums);

    let _ = fs::remove_file(file_path);
}

#[test]
fn test_tile_id_to_rect_and_back() {
    let tileset = TilesetDef {
        id: TilesetDefId(1),
        name: "TestTileset".to_string(),
        path: "test.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        padding: 0,
        spacing: 0,
    };
    let img_size = (256, 256);

    for tile_id in [0, 1, 5, 15, 16, 31, 255] {
        let rect = tile_id_to_rect(&tileset, tile_id, img_size);
        assert_eq!(rect.tileset_uid, 1);
        assert_eq!(rect.w, 16);
        assert_eq!(rect.h, 16);
        let back = rect_to_tile_id(&tileset, &rect, img_size);
        assert_eq!(back, Some(tile_id));
    }

    // Example from prompt: { "id": "Common", "color": 10265519, "tileRect": { "tilesetUid": 1, "x": 16, "y": 0, "w": 16, "h": 16 } }
    let prompt_rect = FleiyaTileRect {
        tileset_uid: 1,
        x: 16,
        y: 0,
        w: 16,
        h: 16,
    };
    assert_eq!(rect_to_tile_id(&tileset, &prompt_rect, img_size), Some(1));
    assert_eq!(tile_id_to_rect(&tileset, 1, img_size), prompt_rect);
}

#[test]
fn test_bound_tile_cleared_conditions() {
    let tilesets = vec![TilesetDef {
        id: TilesetDefId(1),
        name: "Dungeon".to_string(),
        path: "dungeon.png".to_string(),
        tile_size: glam::UVec2::new(16, 16),
        padding: 0,
        spacing: 0,
    }];
    let resolver = |_| Some((64, 64)); // 4x4 tiles, max x=48, max y=48

    // 1. Missing tileset
    let fe_missing_ts = FleiyaEnumDef {
        identifier: "Test".to_string(),
        uid: 1,
        icon_tileset_uid: Some(999),
        values: vec![FleiyaEnumValueDef {
            id: "V1".to_string(),
            color: 0xFFFFFF,
            tile_rect: Some(FleiyaTileRect {
                tileset_uid: 999, // tileset 999 doesn't exist
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            }),
        }],
        tags: vec![],
        external_rel_path: None,
        external_file_checksum: None,
    };
    let def1 = convert_file_enum_to_def(&fe_missing_ts, EnumDefId(1), &tilesets, &resolver);
    assert_eq!(def1.values[0].tile, None);

    // 2. Different tilesetUid (tilesetUid != iconTilesetUid)
    let fe_mismatched_uid = FleiyaEnumDef {
        identifier: "Test".to_string(),
        uid: 1,
        icon_tileset_uid: Some(2), // iconTilesetUid is 2
        values: vec![FleiyaEnumValueDef {
            id: "V1".to_string(),
            color: 0xFFFFFF,
            tile_rect: Some(FleiyaTileRect {
                tileset_uid: 1, // tileRect is 1
                x: 0,
                y: 0,
                w: 16,
                h: 16,
            }),
        }],
        tags: vec![],
        external_rel_path: None,
        external_file_checksum: None,
    };
    let def2 = convert_file_enum_to_def(&fe_mismatched_uid, EnumDefId(1), &tilesets, &resolver);
    assert_eq!(def2.values[0].tile, None);

    // 3. Rect outside tileset image (img is 64x64, x=64 is out of bounds)
    let fe_out_of_bounds = FleiyaEnumDef {
        identifier: "Test".to_string(),
        uid: 1,
        icon_tileset_uid: Some(1),
        values: vec![FleiyaEnumValueDef {
            id: "V1".to_string(),
            color: 0xFFFFFF,
            tile_rect: Some(FleiyaTileRect {
                tileset_uid: 1,
                x: 64, // 64 + 16 > 64
                y: 0,
                w: 16,
                h: 16,
            }),
        }],
        tags: vec![],
        external_rel_path: None,
        external_file_checksum: None,
    };
    let def3 = convert_file_enum_to_def(&fe_out_of_bounds, EnumDefId(1), &tilesets, &resolver);
    assert_eq!(def3.values[0].tile, None);

    // 4. Rect not aligned with grid (x=5)
    let fe_misaligned = FleiyaEnumDef {
        identifier: "Test".to_string(),
        uid: 1,
        icon_tileset_uid: Some(1),
        values: vec![FleiyaEnumValueDef {
            id: "V1".to_string(),
            color: 0xFFFFFF,
            tile_rect: Some(FleiyaTileRect {
                tileset_uid: 1,
                x: 5, // Not a multiple of 16
                y: 0,
                w: 16,
                h: 16,
            }),
        }],
        tags: vec![],
        external_rel_path: None,
        external_file_checksum: None,
    };
    let def4 = convert_file_enum_to_def(&fe_misaligned, EnumDefId(1), &tilesets, &resolver);
    assert_eq!(def4.values[0].tile, None);
}

#[test]
fn test_integer_color_and_hex_exactness() {
    // Common item: #9ca3af -> 10265519
    let common_rgba = [156, 163, 175, 255];
    let common_int = rgba_to_int(common_rgba);
    assert_eq!(common_int, 10265519);
    assert_eq!(int_to_rgba(10265519), common_rgba);
    assert_eq!(int_to_hex(10265519), "#9ca3af");
    assert_eq!(hex_to_int("#9ca3af"), Some(10265519));

    // Rare item: #2d81ff -> 2982399
    let rare_rgba = [45, 129, 255, 255];
    let rare_int = rgba_to_int(rare_rgba);
    assert_eq!(rare_int, 2982399);
    assert_eq!(int_to_rgba(2982399), rare_rgba);
    assert_eq!(int_to_hex(2982399), "#2d81ff");
    assert_eq!(hex_to_int("#2d81ff"), Some(2982399));

    // Extremes: 0 (black), 0xFFFFFF (white)
    assert_eq!(int_to_hex(0), "#000000");
    assert_eq!(hex_to_int("#000000"), Some(0));
    assert_eq!(int_to_hex(0xFFFFFF), "#ffffff");
    assert_eq!(hex_to_int("#ffffff"), Some(0xFFFFFF));
}

#[test]
fn test_unknown_fields_are_ignored() {
    let json_with_unknown = r#"{
      "format": "fleiya.enum",
      "version": 1,
      "future_super_feature": "ignored_value",
      "enums": [
        {
          "identifier": "ItemRarity",
          "uid": 2,
          "iconTilesetUid": 1,
          "extra_enum_property": 12345,
          "values": [
            {
              "id": "Common",
              "color": 10265519,
              "tileRect": { "tilesetUid": 1, "x": 16, "y": 0, "w": 16, "h": 16, "extra_rect_prop": true },
              "extra_value_prop": "ignored"
            }
          ],
          "tags": []
        }
      ]
    }"#;

    let res = validate_enum_file_content(json_with_unknown, Some("test.fleiyaenum"));
    assert!(
        res.is_ok(),
        "Expected unknown fields to be ignored, got: {:?}",
        res
    );
    let parsed = res.unwrap();
    assert_eq!(parsed.enums[0].identifier, "ItemRarity");
    assert_eq!(parsed.enums[0].values[0].id, "Common");
    assert_eq!(parsed.enums[0].values[0].color, 10265519);
}

#[test]
fn test_validation_error_locations() {
    let invalid_json = r#"{
      "format": "fleiya.enum",
      "version": 1,
      "enums": [
        {
          "identifier": "",
          "uid": 1,
          "values": []
        },
        {
          "identifier": "Duplicate",
          "uid": 2,
          "values": [
            { "id": "", "color": 100 },
            { "id": "V2", "color": 16777216 },
            { "id": "V3", "color": 100, "tileRect": { "tilesetUid": 1, "x": 0 } }
          ]
        },
        {
          "identifier": "Duplicate",
          "uid": 3,
          "values": [
            { "id": "V1", "color": 100 },
            { "id": "V1", "color": 100 }
          ]
        }
      ]
    }"#;

    let res = validate_enum_file_content(invalid_json, None);
    assert!(res.is_err());
    let errors = res.unwrap_err();

    assert!(errors
        .iter()
        .any(|e| e == "Enum #1: field 'identifier' is empty."));
    assert!(errors
        .iter()
        .any(|e| e == "Enum #3: identifier 'Duplicate' duplicates Enum #2."));
    assert!(errors
        .iter()
        .any(|e| e == "Enum #2 'Duplicate' > value #1: field 'id' is empty."));
    assert!(errors.iter().any(|e| e
        .contains("Enum #2 'Duplicate' > value #2 'V2': color 16777216 is outside 0..=0xFFFFFF.")));
    assert!(errors
        .iter()
        .any(|e| e == "Enum #2 'Duplicate' > value #3 'V3': incomplete tileRect."));
    assert!(errors
        .iter()
        .any(|e| e.contains("Enum #3 'Duplicate' > value #2 'V1': duplicate value id")));
}

#[test]
fn test_broken_json_and_empty_enums() {
    let broken_json = "{ format: fleiya.enum ";
    let res = validate_enum_file_content(broken_json, None);
    assert!(res.is_err());
    let err = &res.unwrap_err()[0];
    assert!(
        err.starts_with("Broken JSON at line"),
        "Expected Broken JSON message, got: {}",
        err
    );

    let empty_enums = r#"{ "format": "fleiya.enum", "version": 1, "enums": [] }"#;
    let res2 = validate_enum_file_content(empty_enums, None);
    assert!(res2.is_err());
    assert!(res2
        .unwrap_err()
        .contains(&"The 'enums' list is empty.".to_string()));
}
