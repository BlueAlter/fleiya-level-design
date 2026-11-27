use glam::{IVec2, UVec2, Vec2};
use lc_core::command::UndoStack;
use lc_core::defs::{
    EntityDef, EntityLimitBehavior, EntityLimitScope, EnumDef, EnumValueDef, LayerDef, TilesetRect,
};
use lc_core::fields::{AllowedRefs, FieldDef, FieldType, FieldValue, FieldValues};
use lc_core::id::{
    EntityDefId, EntityId, EnumDefId, IdGenerator, LayerDefId, LevelId, TilesetDefId,
};
use lc_core::layer::{EntityInstance, LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_entities::{
    can_place_with_limits, coerce_for_field_def, default_value_for_field, filter_entity_choices,
    next_entity_id, EntityFieldUpdate, PlaceEntityWithLimitsCommand, ReorderEntityDefCommand,
    ReorderEntityFieldCommand, SetEntityFieldValuesCommand,
};

fn build_test_project() -> (
    Project,
    LayerDefId,
    LevelId,
    LevelId,
    EntityDefId,
    EntityDefId,
) {
    let mut gen = IdGenerator::new();
    let mut project = Project::new("LDtk Parity Test");

    let layer_id = gen.next_layer_def_id();
    project.defs.layers.push(LayerDef {
        id: layer_id,
        name: "Entities".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::splat(16),
        tileset_id: None,
        int_grid_values: Vec::new(),
        ..Default::default()
    });

    let def_a = gen.next_entity_def_id();
    let mut ent_a = EntityDef::new(def_a, "Waypoint");
    ent_a.tags = vec!["nav".to_string()];
    ent_a.fields = vec![
        FieldDef {
            name: "next".to_string(),
            field_type: FieldType::EntityRef,
            symmetrical_ref: true,
            allowed_refs: AllowedRefs::OnlySame,
            ..Default::default()
        },
        FieldDef {
            name: "radius".to_string(),
            field_type: FieldType::Int,
            min: Some(4.0),
            max: Some(64.0),
            ..Default::default()
        },
    ];
    project.defs.entities.push(ent_a);

    let def_b = gen.next_entity_def_id();
    let mut ent_b = EntityDef::new(def_b, "Boss");
    ent_b.tags = vec!["enemy".to_string()];
    ent_b.max_count = 1;
    ent_b.limit_scope = EntityLimitScope::PerLevel;
    ent_b.limit_behavior = EntityLimitBehavior::MoveLastOne;
    project.defs.entities.push(ent_b);

    let world_id = gen.next_world_id();
    let mut world = World::new(world_id, "World");

    let level_1_id = gen.next_level_id();
    let mut level_1 = Level::new(level_1_id, "Level_1", UVec2::splat(256));
    level_1.layers.push(LayerInstance::new(
        layer_id,
        LayerData::Entities(Vec::new()),
    ));

    let level_2_id = gen.next_level_id();
    let mut level_2 = Level::new(level_2_id, "Level_2", UVec2::splat(256));
    level_2.layers.push(LayerInstance::new(
        layer_id,
        LayerData::Entities(Vec::new()),
    ));

    world.levels.push(level_1);
    world.levels.push(level_2);
    project.worlds.push(world);

    (project, layer_id, level_1_id, level_2_id, def_a, def_b)
}

#[test]
fn reorder_entity_def_and_field_apply_and_revert() {
    let (mut project, _, _, _, def_a, def_b) = build_test_project();
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(ReorderEntityDefCommand::new(def_b, 0)),
            &mut project,
        )
        .expect("reorder entity def");
    assert_eq!(project.defs.entities[0].id, def_b);
    assert_eq!(project.defs.entities[1].id, def_a);

    stack
        .execute(
            Box::new(ReorderEntityFieldCommand::new(def_a, 0, 1)),
            &mut project,
        )
        .expect("reorder entity field");
    assert_eq!(project.defs.entity(def_a).unwrap().fields[0].name, "radius");
    assert_eq!(project.defs.entity(def_a).unwrap().fields[1].name, "next");

    stack.undo(&mut project).expect("undo field reorder");
    stack.undo(&mut project).expect("undo def reorder");
    assert_eq!(project, snapshot);
}

#[test]
fn limit_behaviors_move_discard_and_prevent_revert_cleanly() {
    let (mut project, layer_id, level_1_id, _, _, def_b) = build_test_project();
    let mut stack = UndoStack::new();

    // 1. Place first Boss
    let e1 = EntityInstance {
        id: next_entity_id(&project),
        def_id: def_b,
        pos_px: IVec2::new(16, 16),
        size_px: UVec2::splat(16),
        fields: FieldValues::new(),
    };
    stack
        .execute(
            Box::new(PlaceEntityWithLimitsCommand::new(level_1_id, layer_id, e1)),
            &mut project,
        )
        .expect("place first");

    // 2. MoveLastOne: placing second moves the existing Boss
    let snapshot_after_first = project.clone();
    let e2 = EntityInstance {
        id: next_entity_id(&project),
        def_id: def_b,
        pos_px: IVec2::new(64, 48),
        size_px: UVec2::splat(16),
        fields: FieldValues::new(),
    };
    stack
        .execute(
            Box::new(PlaceEntityWithLimitsCommand::new(level_1_id, layer_id, e2)),
            &mut project,
        )
        .expect("move last one");

    let ents = match &project
        .level(level_1_id)
        .unwrap()
        .layer(layer_id)
        .unwrap()
        .data
    {
        LayerData::Entities(list) => list,
        _ => panic!("expected entities"),
    };
    assert_eq!(ents.len(), 1);
    assert_eq!(ents[0].pos_px, IVec2::new(64, 48));
    stack.undo(&mut project).expect("undo move last");
    assert_eq!(project, snapshot_after_first);

    // 3. DiscardOldOnes: removes oldest and inserts new one, undo restores exact instance
    project.defs.entity_mut(def_b).unwrap().limit_behavior = EntityLimitBehavior::DiscardOldOnes;
    let snapshot_before_discard = project.clone();
    let new_id = next_entity_id(&project);
    let e3 = EntityInstance {
        id: new_id,
        def_id: def_b,
        pos_px: IVec2::new(96, 96),
        size_px: UVec2::splat(16),
        fields: FieldValues::new(),
    };
    stack
        .execute(
            Box::new(PlaceEntityWithLimitsCommand::new(level_1_id, layer_id, e3)),
            &mut project,
        )
        .expect("discard old ones");
    let ents = match &project
        .level(level_1_id)
        .unwrap()
        .layer(layer_id)
        .unwrap()
        .data
    {
        LayerData::Entities(list) => list,
        _ => panic!("expected entities"),
    };
    assert_eq!(ents.len(), 1);
    assert_eq!(ents[0].id, new_id);
    stack.undo(&mut project).expect("undo discard");
    assert_eq!(project, snapshot_before_discard);

    // 4. PreventAdding
    project.defs.entity_mut(def_b).unwrap().limit_behavior = EntityLimitBehavior::PreventAdding;
    let world_id = project.worlds[0].id;
    assert!(!can_place_with_limits(
        &project, def_b, world_id, level_1_id, layer_id
    ));
    let e4 = EntityInstance {
        id: next_entity_id(&project),
        def_id: def_b,
        pos_px: IVec2::ZERO,
        size_px: UVec2::splat(16),
        fields: FieldValues::new(),
    };
    assert!(stack
        .execute(
            Box::new(PlaceEntityWithLimitsCommand::new(level_1_id, layer_id, e4)),
            &mut project
        )
        .is_err());
}

#[test]
fn symmetrical_entity_ref_updates_and_reverts_both_ends() {
    let (mut project, layer_id, level_1_id, _, def_a, _) = build_test_project();
    let id1 = project.allocate_entity_id();
    let id2 = project.allocate_entity_id();
    let id3 = project.allocate_entity_id();

    if let LayerData::Entities(list) = &mut project
        .level_mut(level_1_id)
        .unwrap()
        .layer_mut(layer_id)
        .unwrap()
        .data
    {
        for (idx, id) in [id1, id2, id3].into_iter().enumerate() {
            list.push(EntityInstance {
                id,
                def_id: def_a,
                pos_px: IVec2::new(idx as i32 * 32, 0),
                size_px: UVec2::splat(16),
                fields: FieldValues::new(),
            });
        }
    }

    let snapshot = project.clone();
    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::new(
                level_1_id,
                vec![EntityFieldUpdate {
                    entity_id: id1,
                    field_name: "next".to_string(),
                    value: FieldValue::EntityRef(Some(id2)),
                }],
            )),
            &mut project,
        )
        .expect("link id1 <-> id2");

    let lvl = project.level(level_1_id).unwrap();
    assert_eq!(
        lvl.entity(id1).unwrap().fields.get("next"),
        Some(&FieldValue::EntityRef(Some(id2)))
    );
    assert_eq!(
        lvl.entity(id2).unwrap().fields.get("next"),
        Some(&FieldValue::EntityRef(Some(id1)))
    );

    // Re-link id1 -> id3 must unlink id2
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::new(
                level_1_id,
                vec![EntityFieldUpdate {
                    entity_id: id1,
                    field_name: "next".to_string(),
                    value: FieldValue::EntityRef(Some(id3)),
                }],
            )),
            &mut project,
        )
        .expect("link id1 <-> id3");
    let lvl = project.level(level_1_id).unwrap();
    assert_eq!(
        lvl.entity(id2).unwrap().fields.get("next"),
        Some(&FieldValue::EntityRef(None))
    );
    assert_eq!(
        lvl.entity(id3).unwrap().fields.get("next"),
        Some(&FieldValue::EntityRef(Some(id1)))
    );

    stack.undo(&mut project).expect("undo second link");
    stack.undo(&mut project).expect("undo first link");
    assert_eq!(project, snapshot);
}

#[test]
fn allowed_refs_filters_and_smart_color_resolves() {
    let (mut project, layer_id, level_1_id, _, def_a, def_b) = build_test_project();
    let id_a = project.allocate_entity_id();
    let id_b = project.allocate_entity_id();
    if let LayerData::Entities(list) = &mut project
        .level_mut(level_1_id)
        .unwrap()
        .layer_mut(layer_id)
        .unwrap()
        .data
    {
        list.push(EntityInstance {
            id: id_a,
            def_id: def_a,
            pos_px: IVec2::ZERO,
            size_px: UVec2::splat(16),
            fields: FieldValues::new(),
        });
        list.push(EntityInstance {
            id: id_b,
            def_id: def_b,
            pos_px: IVec2::new(32, 0),
            size_px: UVec2::splat(16),
            fields: FieldValues::new(),
        });
    }

    let same_field = FieldDef {
        allowed_refs: AllowedRefs::OnlySame,
        ..FieldDef::new("ref", FieldType::EntityRef)
    };
    assert_eq!(filter_entity_choices(&project, def_a, &same_field).len(), 1);

    let any_field = FieldDef {
        allowed_refs: AllowedRefs::Any,
        ..FieldDef::new("ref", FieldType::EntityRef)
    };
    assert_eq!(filter_entity_choices(&project, def_a, &any_field).len(), 2);

    let tag_field = FieldDef {
        allowed_refs: AllowedRefs::OnlyTags,
        allowed_ref_tags: vec!["enemy".to_string()],
        ..FieldDef::new("ref", FieldType::EntityRef)
    };
    let tagged = filter_entity_choices(&project, def_a, &tag_field);
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].id, id_b);

    // SmartColor + nullable/clamping + new field types
    let enum_id = EnumDefId::new(1);
    let enums = vec![EnumDef {
        id: enum_id,
        name: "Rarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "Gold".to_string(),
            name: "Gold".to_string(),
            color: [255, 215, 0, 255],
            tile: None,
        }],
    }];
    let mut smart_def = EntityDef::new(EntityDefId::new(10), "Loot");
    smart_def.color = [100, 100, 100, 255];
    smart_def.fields.push(FieldDef {
        use_for_smart_color: true,
        ..FieldDef::new("rarity", FieldType::Enum(enum_id))
    });
    let mut fv = FieldValues::new();
    fv.set("rarity", FieldValue::Enum("Gold".to_string()));
    assert_eq!(smart_def.smart_color(&fv, &enums), [255, 215, 0, 255]);

    let nullable_int = FieldDef {
        can_be_null: true,
        min: Some(5.0),
        max: Some(10.0),
        ..FieldDef::new("n", FieldType::Int)
    };
    assert_eq!(
        default_value_for_field(&nullable_int, &enums),
        FieldValue::Null
    );
    assert_eq!(
        coerce_for_field_def(&FieldValue::Int(999), &nullable_int, &enums),
        FieldValue::Int(10)
    );
    assert_eq!(
        coerce_for_field_def(&FieldValue::Null, &nullable_int, &enums),
        FieldValue::Null
    );

    let tile_field = FieldDef::new("icon", FieldType::Tile);
    let rect = TilesetRect::new(TilesetDefId::new(1), 16, 32, 16, 16);
    assert_eq!(
        coerce_for_field_def(&FieldValue::Tile(Some(rect)), &tile_field, &enums),
        FieldValue::Tile(Some(rect))
    );
    let _ = Vec2::ZERO;
    let _ = EntityId::new(1);
}

#[test]
fn all_11_field_types_and_arrays_sample_and_visual_effect_parity() {
    use lc_core::defs::{SmartTileSource, TilesetDef};
    use lc_core::fields::{EditorDisplayMode, EditorDisplayPos, EditorLinkStyle};
    use lc_core::layer::TileRef;
    use lc_feature_entities::{
        analyze_field_visual_effects, effective_preview_values, format_overlay_text,
        sample_field_value, FieldVisualImpact,
    };

    let enum_id = EnumDefId::new(7);
    let ts_id = TilesetDefId::new(3);
    let enums = vec![EnumDef {
        id: enum_id,
        name: "WeaponType".to_string(),
        tileset_id: Some(ts_id),
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Sword".to_string(),
                name: "Sword".to_string(),
                color: [220, 60, 80, 255],
                tile: Some(TileRef::new(ts_id, 1)),
            },
            EnumValueDef {
                id: "Bow".to_string(),
                name: "Bow".to_string(),
                color: [80, 200, 120, 255],
                tile: Some(TileRef::new(ts_id, 2)),
            },
        ],
    }];
    let tilesets = vec![TilesetDef::new(
        ts_id,
        "Icons",
        "assets/icons.png",
        UVec2::splat(16),
    )];

    let single_specs = [
        ("s_int", FieldType::Int, EditorDisplayMode::RadiusPx),
        ("s_float", FieldType::Float, EditorDisplayMode::RadiusGrid),
        ("s_bool", FieldType::Bool, EditorDisplayMode::ValueOnly),
        ("s_str", FieldType::String, EditorDisplayMode::NameAndValue),
        ("s_txt", FieldType::Multilines, EditorDisplayMode::ValueOnly),
        ("s_col", FieldType::Color, EditorDisplayMode::ValueOnly),
        (
            "s_enu",
            FieldType::Enum(enum_id),
            EditorDisplayMode::EntityTile,
        ),
        (
            "s_file",
            FieldType::FilePath,
            EditorDisplayMode::NameAndValue,
        ),
        ("s_tile", FieldType::Tile, EditorDisplayMode::ValueOnly),
        (
            "s_ref",
            FieldType::EntityRef,
            EditorDisplayMode::RefLinkBetweenPivots,
        ),
        ("s_pt", FieldType::Point, EditorDisplayMode::PointStar),
    ];

    let mut entity = EntityDef::new(EntityDefId::new(99), "Hero");
    for (idx, (name, ft, mode)) in single_specs.iter().enumerate() {
        let mut fd = FieldDef::new(*name, ft.clone());
        fd.editor_display_mode = *mode;
        fd.editor_display_pos = EditorDisplayPos::Above;
        fd.editor_link_style = EditorLinkStyle::CurvedArrow;
        if *name == "s_str" {
            fd.editor_cut_long_values = true;
        }
        if *name == "s_col" {
            fd.use_for_smart_color = true;
        }
        entity.fields.push(fd);

        let mut arr_fd = FieldDef::new(format!("a_{name}"), FieldType::Array(Box::new(ft.clone())));
        arr_fd.editor_display_mode = match idx % 3 {
            0 => EditorDisplayMode::ArrayCountWithLabel,
            1 => EditorDisplayMode::ArrayCountNoLabel,
            _ => *mode,
        };
        if *name == "s_pt" {
            arr_fd.editor_display_mode = EditorDisplayMode::PointPathLoop;
        }
        entity.fields.push(arr_fd);
    }
    let mut hidden_fd = FieldDef::new("hidden_data", FieldType::Int);
    hidden_fd.editor_display_mode = EditorDisplayMode::Hidden;
    entity.fields.push(hidden_fd);

    let mut level_tile_fd = FieldDef::new("lvl_tile", FieldType::Tile);
    level_tile_fd.editor_display_mode = EditorDisplayMode::LevelTile;
    entity.fields.push(level_tile_fd);

    let samples = effective_preview_values(&entity, &enums, &tilesets);
    assert_eq!(samples.iter().count(), entity.fields.len());

    // All 11 single + 11 array active fields produce non-null samples & visual impacts
    for field in &entity.fields {
        let val = samples.get(&field.name).expect("sample must exist");
        assert!(!matches!(val, FieldValue::Null), "{}", field.name);
        if field.is_array() {
            match val {
                FieldValue::Array(items) => assert_eq!(items.len(), 3, "{}", field.name),
                other => panic!("expected array sample for {}, got {other:?}", field.name),
            }
        }
        let impacts = analyze_field_visual_effects(field, val, &enums, 16);
        let affects = impacts.iter().any(FieldVisualImpact::affects_entity);
        if field.name == "hidden_data" || field.name == "lvl_tile" {
            assert!(!affects, "{} should NOT alter entity visuals", field.name);
        } else {
            assert!(affects, "{} SHOULD alter entity visuals", field.name);
        }
    }

    // Verify smart_color & smart_tile resolution match generated sample values
    let expected_color = match samples.get("s_col") {
        Some(FieldValue::Color(c)) => *c,
        other => panic!("expected color sample, got {other:?}"),
    };
    assert_eq!(entity.smart_color(&samples, &enums), expected_color);
    assert_eq!(
        entity.smart_tile(&samples, &enums),
        Some(SmartTileSource::Ref(TileRef::new(ts_id, 1)))
    );

    // Verify LDtk Bool ValueOnly `+name` / `-name` formatting
    let bool_fd = entity.field_def("s_bool").unwrap();
    assert_eq!(
        format_overlay_text(
            bool_fd,
            &FieldValue::Bool(true),
            EditorDisplayMode::ValueOnly
        ),
        "+s_bool"
    );
    assert_eq!(
        format_overlay_text(
            bool_fd,
            &FieldValue::Bool(false),
            EditorDisplayMode::ValueOnly
        ),
        "-s_bool"
    );

    // Verify Array Sample respects array_min_length / array_max_length and min/max clamping
    let mut bounded_arr = FieldDef::new("bounded", FieldType::Array(Box::new(FieldType::Int)));
    bounded_arr.min = Some(50.0);
    bounded_arr.max = Some(60.0);
    bounded_arr.array_min_length = Some(4);
    let bounded_val = sample_field_value(&bounded_arr, &enums, &tilesets, 0);
    match bounded_val {
        FieldValue::Array(items) => {
            assert_eq!(items.len(), 4);
            for item in items {
                match item {
                    FieldValue::Int(n) => assert!((50..=60).contains(&n)),
                    other => panic!("expected int item, got {other:?}"),
                }
            }
        }
        other => panic!("expected array, got {other:?}"),
    }
}
