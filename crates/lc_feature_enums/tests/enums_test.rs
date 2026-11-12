use egui::Context;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::defs::{EntityDef, EnumDef, EnumValueDef};
use lc_core::fields::{FieldDef, FieldType, FieldValue, FieldValues};
use lc_core::id::{EntityDefId, EntityId, EnumDefId, LevelId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, TileRef};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_enums::commands::{
    field_type_references_enum, find_enum_def_references, find_enum_value_references,
    AddEnumDefCommand, AddEnumValueCommand, RemoveEnumDefCommand, RemoveEnumValueCommand,
    RenameEnumValueIdCommand, ReorderEnumValuesCommand, UpdateEnumDefCommand,
    UpdateEnumValueCommand,
};
use lc_feature_enums::view::EnumDefDocumentView;
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;

fn create_test_project_with_enum() -> (Project, EnumDefId) {
    let mut p = Project::new("EnumTestProject");
    let enum_id = EnumDefId::new(10);
    let enum_def = EnumDef {
        id: enum_id,
        name: "Difficulty".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![
            EnumValueDef {
                id: "Easy".to_string(),
                name: "Easy Mode".to_string(),
                color: [0, 255, 0, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Hard".to_string(),
                name: "Hardcore".to_string(),
                color: [255, 0, 0, 255],
                tile: Some(TileRef::new(lc_core::id::TilesetDefId::new(1), 5)),
            },
        ],
    };
    p.defs.enums.push(enum_def);
    (p, enum_id)
}

#[test]
fn test_enum_factory_and_document_registration() {
    let mut reg = Registry::new();
    lc_feature_enums::register(&mut reg);

    let (p, enum_id) = create_test_project_with_enum();

    let doc = reg.create_document(DocKind::EnumDef(enum_id));
    assert!(doc.is_some(), "Document factory must create EnumDef tab");
    let doc = doc.unwrap();

    assert_eq!(doc.doc_kind(), DocKind::EnumDef(enum_id));
    assert_eq!(doc.title(&p), "Enum: Difficulty");
    assert!(doc.is_valid(&p));

    assert!(reg.create_document(DocKind::ProjectSettings).is_none());
}

#[test]
fn test_add_update_remove_enum_def_commands() {
    let mut p = Project::new("AddUpdateTest");
    let mut undo = UndoStack::new();

    let enum_id = EnumDefId::new(20);
    let def = EnumDef {
        id: enum_id,
        name: "Direction".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: vec![EnumValueDef {
            id: "North".to_string(),
            name: "North".to_string(),
            color: [255, 255, 0, 255],
            tile: None,
        }],
    };

    // 1. Add
    undo.execute(Box::new(AddEnumDefCommand::new(def.clone())), &mut p)
        .expect("add enum");
    assert_eq!(p.defs.enums.len(), 1);
    assert_eq!(p.defs.enums[0].name, "Direction");

    // Duplicate add should fail
    let mut dup_cmd = AddEnumDefCommand::new(def.clone());
    assert!(dup_cmd.apply(&mut p).is_err());

    // 2. Update with stroke merge
    let mut updated = def.clone();
    updated.name = "Directional".to_string();
    let cmd1 = UpdateEnumDefCommand::with_stroke(updated.clone(), 100);
    undo.execute(Box::new(cmd1), &mut p).expect("update 1");

    updated.name = "CardinalDirection".to_string();
    let cmd2 = UpdateEnumDefCommand::with_stroke(updated.clone(), 100);
    undo.execute(Box::new(cmd2), &mut p).expect("update 2");

    assert_eq!(p.defs.enums[0].name, "CardinalDirection");

    // 3. Undo update
    let (lbl, _) = undo.undo(&mut p).expect("undo update");
    assert!(lbl.contains("Edit Enum"));
    assert_eq!(p.defs.enums[0].name, "Direction");

    // 4. Undo add
    undo.undo(&mut p).expect("undo add");
    assert!(p.defs.enums.is_empty());

    // 5. Redo add
    undo.redo(&mut p).expect("redo add");
    assert_eq!(p.defs.enums.len(), 1);
}

#[test]
fn test_enum_values_add_update_reorder_revert() {
    let (mut p, enum_id) = create_test_project_with_enum();
    let mut undo = UndoStack::new();

    // 1. Add value
    let new_val = EnumValueDef {
        id: "Normal".to_string(),
        name: "Normal Mode".to_string(),
        color: [200, 200, 0, 255],
        tile: None,
    };
    undo.execute(Box::new(AddEnumValueCommand::new(enum_id, new_val)), &mut p)
        .expect("add value");
    assert_eq!(p.defs.enum_def(enum_id).unwrap().values.len(), 3);

    // 2. Reorder values: ["Normal", "Easy", "Hard"]
    undo.execute(
        Box::new(ReorderEnumValuesCommand::new(
            enum_id,
            vec!["Normal".to_string(), "Easy".to_string(), "Hard".to_string()],
        )),
        &mut p,
    )
    .expect("reorder values");

    let values = &p.defs.enum_def(enum_id).unwrap().values;
    assert_eq!(values[0].id, "Normal");
    assert_eq!(values[1].id, "Easy");
    assert_eq!(values[2].id, "Hard");

    // 3. Update value display name & color
    let mut updated_hard = values[2].clone();
    updated_hard.name = "Insane".to_string();
    updated_hard.color = [50, 50, 50, 255];
    undo.execute(
        Box::new(UpdateEnumValueCommand::new(enum_id, updated_hard)),
        &mut p,
    )
    .expect("update value");

    assert_eq!(
        p.defs
            .enum_def(enum_id)
            .unwrap()
            .value("Hard")
            .unwrap()
            .name,
        "Insane"
    );

    // 4. Undo update
    undo.undo(&mut p).expect("undo update value");
    assert_eq!(
        p.defs
            .enum_def(enum_id)
            .unwrap()
            .value("Hard")
            .unwrap()
            .name,
        "Hardcore"
    );

    // 5. Undo reorder
    undo.undo(&mut p).expect("undo reorder");
    let original = &p.defs.enum_def(enum_id).unwrap().values;
    assert_eq!(original[0].id, "Easy");
    assert_eq!(original[1].id, "Hard");
    assert_eq!(original[2].id, "Normal");

    // 6. Undo add
    undo.undo(&mut p).expect("undo add value");
    assert_eq!(p.defs.enum_def(enum_id).unwrap().values.len(), 2);
}

#[test]
fn test_enum_deletion_blocked_when_referenced_and_cascade() {
    let (mut p, enum_id) = create_test_project_with_enum();

    // Attach enum to an entity field
    let entity_id = EntityDefId::new(100);
    let entity_def = EntityDef {
        id: entity_id,
        name: "Spawner".to_string(),
        size_px: glam::UVec2::new(16, 16),
        color: [255, 0, 0, 255],
        pivot: glam::Vec2::new(0.5, 0.5),
        tile: None,
        fields: vec![FieldDef {
            name: "difficulty_level".to_string(),
            field_type: FieldType::Enum(enum_id),
            default_value: Some(FieldValue::Enum("Easy".to_string())),
        }],
    };
    p.defs.entities.push(entity_def);

    // Verify reference detection
    let refs = find_enum_def_references(&p, enum_id);
    assert_eq!(refs.len(), 1);
    assert!(field_type_references_enum(
        &FieldType::Enum(enum_id),
        enum_id
    ));
    assert!(field_type_references_enum(
        &FieldType::Array(Box::new(FieldType::Enum(enum_id))),
        enum_id
    ));

    // 1. Safe deletion must be BLOCKED
    let mut safe_cmd = RemoveEnumDefCommand::safe(enum_id);
    assert!(
        safe_cmd.apply(&mut p).is_err(),
        "Safe deletion must be blocked if enum is referenced"
    );
    assert!(p.defs.enum_def(enum_id).is_some());

    // 2. Cascade deletion must succeed and clean up referencing field
    let mut undo = UndoStack::new();
    undo.execute(Box::new(RemoveEnumDefCommand::cascade(enum_id)), &mut p)
        .expect("cascade remove");
    assert!(p.defs.enum_def(enum_id).is_none());
    assert!(p.defs.entity(entity_id).unwrap().fields.is_empty());

    // 3. Revert must restore EnumDef AND the entity field
    undo.undo(&mut p).expect("undo cascade remove");
    assert!(p.defs.enum_def(enum_id).is_some());
    assert_eq!(p.defs.entity(entity_id).unwrap().fields.len(), 1);
    assert_eq!(
        p.defs.entity(entity_id).unwrap().fields[0].name,
        "difficulty_level"
    );
}

#[test]
fn test_rename_enum_value_id_cascades_to_instances() {
    let (mut p, enum_id) = create_test_project_with_enum();

    // Create an entity definition
    let entity_def_id = EntityDefId::new(50);
    p.defs.entities.push(EntityDef {
        id: entity_def_id,
        name: "Goblin".to_string(),
        size_px: glam::UVec2::new(16, 16),
        color: [0, 255, 0, 255],
        pivot: glam::Vec2::new(0.5, 0.5),
        tile: None,
        fields: vec![FieldDef {
            name: "mode".to_string(),
            field_type: FieldType::Enum(enum_id),
            default_value: Some(FieldValue::Enum("Easy".to_string())),
        }],
    });

    // Create a level with an entity instance
    let mut world = World::new(WorldId::new(1), "World");
    let level_id = LevelId::new(1);
    let mut level = Level::new(level_id, "Level", glam::UVec2::new(100, 100));

    let mut fields = FieldValues::new();
    fields.set("mode", FieldValue::Enum("Easy".to_string()));

    let entity_inst = lc_core::layer::EntityInstance {
        id: EntityId::new(1),
        def_id: entity_def_id,
        pos_px: glam::IVec2::new(10, 10),
        size_px: glam::UVec2::new(16, 16),
        fields,
    };
    level.layers.push(LayerInstance {
        def_id: lc_core::id::LayerDefId::new(1),
        visible: true,
        data: LayerData::Entities(vec![entity_inst]),
    });
    world.levels.push(level);
    p.worlds.push(world);

    // Verify value reference detection
    let val_refs = find_enum_value_references(&p, enum_id, "Easy");
    assert_eq!(val_refs.len(), 2, "1 in default + 1 in instance");

    // Rename "Easy" -> "Novice"
    let mut undo = UndoStack::new();
    undo.execute(
        Box::new(RenameEnumValueIdCommand::new(enum_id, "Easy", "Novice")),
        &mut p,
    )
    .expect("rename enum value id");

    // Verify EnumDef has new ID
    let def = p.defs.enum_def(enum_id).unwrap();
    assert!(def.value("Novice").is_some());
    assert!(def.value("Easy").is_none());

    // Verify default value updated
    assert_eq!(
        p.defs.entity(entity_def_id).unwrap().fields[0].default_value,
        Some(FieldValue::Enum("Novice".to_string()))
    );

    // Verify instance value updated
    if let LayerData::Entities(ref ents) = p.worlds[0].levels[0].layers[0].data {
        assert_eq!(
            ents[0].fields.get("mode"),
            Some(&FieldValue::Enum("Novice".to_string()))
        );
    } else {
        panic!("Layer should be entities");
    }

    // Undo rename
    undo.undo(&mut p).expect("undo rename");
    let def_restored = p.defs.enum_def(enum_id).unwrap();
    assert!(def_restored.value("Easy").is_some());
    assert!(def_restored.value("Novice").is_none());

    if let LayerData::Entities(ref ents) = p.worlds[0].levels[0].layers[0].data {
        assert_eq!(
            ents[0].fields.get("mode"),
            Some(&FieldValue::Enum("Easy".to_string()))
        );
    }
}

#[test]
fn test_remove_enum_value_blocked_and_cascade() {
    let (mut p, enum_id) = create_test_project_with_enum();

    // Default value in entity referencing "Easy"
    p.defs.entities.push(EntityDef {
        id: EntityDefId::new(77),
        name: "TestEnt".to_string(),
        size_px: glam::UVec2::new(16, 16),
        color: [0, 0, 0, 255],
        pivot: glam::Vec2::new(0.5, 0.5),
        tile: None,
        fields: vec![FieldDef {
            name: "diff".to_string(),
            field_type: FieldType::Enum(enum_id),
            default_value: Some(FieldValue::Enum("Easy".to_string())),
        }],
    });

    // 1. Safe remove should fail
    let mut safe_cmd = RemoveEnumValueCommand::safe(enum_id, "Easy");
    assert!(safe_cmd.apply(&mut p).is_err());

    // 2. Cascade remove should succeed and clear default value
    let mut undo = UndoStack::new();
    undo.execute(
        Box::new(RemoveEnumValueCommand::cascade(enum_id, "Easy")),
        &mut p,
    )
    .expect("cascade remove value");

    assert!(p.defs.enum_def(enum_id).unwrap().value("Easy").is_none());
    assert_eq!(
        p.defs.entity(EntityDefId::new(77)).unwrap().fields[0].default_value,
        None
    );

    // 3. Undo restores value and default
    undo.undo(&mut p).expect("undo remove value");
    assert!(p.defs.enum_def(enum_id).unwrap().value("Easy").is_some());
    assert_eq!(
        p.defs.entity(EntityDefId::new(77)).unwrap().fields[0].default_value,
        Some(FieldValue::Enum("Easy".to_string()))
    );
}

#[test]
fn test_enum_document_view_ui_rendering() {
    let (p, enum_id) = create_test_project_with_enum();
    let mut view = EnumDefDocumentView::new(enum_id);

    let ctx = Context::default();
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &p,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert!(!output.shapes.is_empty(), "UI should have emitted shapes");
    assert_eq!(view.enum_def_id(), enum_id);
}
