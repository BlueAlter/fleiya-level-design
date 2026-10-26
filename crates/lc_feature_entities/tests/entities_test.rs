//! Integration and headless UI tests for `lc_feature_entities`.

use glam::{IVec2, UVec2, Vec2};
use lc_core::command::{DocKind, UndoStack};
use lc_core::defs::{EntityDef, EnumDef, EnumValueDef};
use lc_core::fields::{FieldDef, FieldType, FieldValue};
use lc_core::id::{EntityDefId, EntityId, EnumDefId, IdGenerator, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_entities::commands::{
    AddEntityDefCommand, RemoveEntityDefCommand, UpdateEntityDefCommand,
};
use lc_feature_entities::field_commands::{
    AddEntityFieldCommand, RemoveEntityFieldCommand, UpdateEntityFieldCommand,
};
use lc_feature_entities::field_editors::{
    field_type_editor, field_value_editor, EntityChoice, FieldEditorContext,
};
use lc_feature_entities::helpers::{count_entity_def_instances, next_entity_id};
use lc_feature_entities::register;
use lc_feature_entities::view::EntityDefDocumentView;
use lc_feature_entities::StrokeTracker;
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::DocumentView;

struct Fixture {
    project: Project,
    level_id: LevelId,
    entity_def_id: EntityDefId,
    enemy_def_id: EntityDefId,
    enum_id: EnumDefId,
}

/// Builds a project with two entity definitions, one entity layer and three
/// instances (two enemies, one player).
fn fixture() -> Fixture {
    let mut gen = IdGenerator::new();
    let mut project = Project::new("Entities Test");

    let enum_id = gen.next_enum_def_id();
    project.defs.enums.push(EnumDef {
        id: enum_id,
        name: "Faction".to_string(),
        values: vec![
            EnumValueDef {
                id: "Ally".to_string(),
                name: "Ally".to_string(),
                color: [0, 200, 255, 255],
                tile: None,
            },
            EnumValueDef {
                id: "Hostile".to_string(),
                name: "Hostile".to_string(),
                color: [255, 80, 80, 255],
                tile: None,
            },
        ],
    });

    let entity_def_id = gen.next_entity_def_id();
    project.defs.entities.push(EntityDef {
        id: entity_def_id,
        name: "Player".to_string(),
        size_px: UVec2::splat(16),
        color: [255, 255, 255, 255],
        pivot: Vec2::splat(0.5),
        tile: None,
        fields: vec![FieldDef {
            name: "health".to_string(),
            field_type: FieldType::Int,
            default_value: Some(FieldValue::Int(10)),
        }],
    });

    let enemy_def_id = gen.next_entity_def_id();
    project.defs.entities.push(EntityDef {
        id: enemy_def_id,
        name: "Enemy".to_string(),
        size_px: UVec2::splat(16),
        color: [255, 0, 0, 255],
        pivot: Vec2::splat(0.5),
        tile: None,
        fields: vec![FieldDef {
            name: "faction".to_string(),
            field_type: FieldType::Enum(enum_id),
            default_value: Some(FieldValue::Enum("Hostile".to_string())),
        }],
    });

    let layer_def_id = gen.next_layer_def_id();
    project.defs.layers.push(lc_core::defs::LayerDef {
        id: layer_def_id,
        name: "Actors".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::splat(16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });

    let level_id = gen.next_level_id();
    let mut level = Level::new(level_id, "Level", UVec2::new(512, 512));
    level.layers.push(LayerInstance::new(
        layer_def_id,
        LayerData::Entities(vec![
            EntityInstance {
                id: EntityId::new(1),
                def_id: enemy_def_id,
                pos_px: IVec2::new(16, 16),
                size_px: UVec2::splat(16),
                fields: {
                    let mut values = lc_core::fields::FieldValues::new();
                    values.set("faction", FieldValue::Enum("Hostile".to_string()));
                    values.set("ally", FieldValue::EntityRef(Some(EntityId::new(2))));
                    values
                },
            },
            EntityInstance {
                id: EntityId::new(2),
                def_id: entity_def_id,
                pos_px: IVec2::new(64, 16),
                size_px: UVec2::splat(16),
                fields: {
                    let mut values = lc_core::fields::FieldValues::new();
                    values.set("health", FieldValue::Int(10));
                    values
                },
            },
            EntityInstance {
                id: EntityId::new(3),
                def_id: enemy_def_id,
                pos_px: IVec2::new(96, 16),
                size_px: UVec2::splat(16),
                fields: {
                    let mut values = lc_core::fields::FieldValues::new();
                    values.set("faction", FieldValue::Enum("Ally".to_string()));
                    values
                },
            },
        ]),
    ));

    let mut world = World::new(WorldId::new(1), "World");
    world.levels.push(level);
    project.worlds.push(world);

    Fixture {
        project,
        level_id,
        entity_def_id,
        enemy_def_id,
        enum_id,
    }
}

fn entity(project: &Project, id: EntityId) -> &EntityInstance {
    project
        .level(project.worlds[0].levels[0].id)
        .and_then(|level| level.entity(id))
        .expect("entity exists")
}

#[test]
fn add_entity_def_applies_and_reverts() {
    let mut project = fixture().project;
    let original = project.clone();
    let def = EntityDef {
        id: EntityDefId::new(99),
        name: "Coin".to_string(),
        size_px: UVec2::splat(8),
        color: [255, 215, 0, 255],
        pivot: Vec2::splat(0.5),
        tile: None,
        fields: Vec::new(),
    };

    let mut stack = UndoStack::new();
    let focus = stack
        .execute(
            Box::new(AddEntityDefCommand::new(def.clone())),
            &mut project,
        )
        .expect("apply succeeds");
    assert_eq!(focus, Some(DocKind::EntityDef(def.id)));
    assert_eq!(project.defs.entity(def.id), Some(&def));

    stack.undo(&mut project).expect("undo exists");
    assert_eq!(project, original);
}

#[test]
fn update_entity_def_merges_one_stroke_and_reverts_to_original() {
    let mut project = fixture().project;
    let snapshot = project.clone();
    let original = project
        .defs
        .entity(EntityDefId::new(2))
        .cloned()
        .expect("def exists");

    let mut stack = UndoStack::new();
    let mut first = original.clone();
    first.name = "Player One".to_string();
    stack
        .execute(
            Box::new(UpdateEntityDefCommand::with_stroke(first, 7)),
            &mut project,
        )
        .expect("apply first");

    let mut second = original.clone();
    second.name = "Player Two".to_string();
    stack
        .execute(
            Box::new(UpdateEntityDefCommand::with_stroke(second, 7)),
            &mut project,
        )
        .expect("merge second");

    assert_eq!(
        project.defs.entity(original.id).map(|d| d.name.as_str()),
        Some("Player Two")
    );

    stack.undo(&mut project).expect("single merged undo");
    assert_eq!(project.defs.entity(original.id), Some(&original));
    assert_eq!(project, snapshot);
}

#[test]
fn update_entity_def_without_matching_stroke_keeps_two_steps() {
    let mut project = fixture().project;
    let def_id = EntityDefId::new(2);
    let original = project.defs.entity(def_id).cloned().expect("def exists");

    let mut stack = UndoStack::new();
    let mut first = original.clone();
    first.name = "First".to_string();
    stack
        .execute(
            Box::new(UpdateEntityDefCommand::with_stroke(first, 1)),
            &mut project,
        )
        .expect("apply first");

    let mut second = original.clone();
    second.name = "Second".to_string();
    stack
        .execute(
            Box::new(UpdateEntityDefCommand::with_stroke(second, 2)),
            &mut project,
        )
        .expect("apply second");

    stack.undo(&mut project).expect("undo second");
    assert_eq!(
        project.defs.entity(def_id).map(|d| d.name.as_str()),
        Some("First")
    );
    stack.undo(&mut project).expect("undo first");
    assert_eq!(project.defs.entity(def_id), Some(&original));
}

#[test]
fn remove_entity_def_blocks_while_used_and_cascades_when_forced() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let enemy_id = fixture.enemy_def_id;

    let mut stack = UndoStack::new();
    let err = stack
        .execute(
            Box::new(RemoveEntityDefCommand::safe(enemy_id)),
            &mut project,
        )
        .expect_err("safe delete must fail while instances exist");
    assert!(format!("{err}").contains("instance"));
    assert!(project.defs.entity(enemy_id).is_some());

    stack
        .execute(
            Box::new(RemoveEntityDefCommand::cascade(enemy_id)),
            &mut project,
        )
        .expect("cascade delete succeeds");
    assert!(project.defs.entity(enemy_id).is_none());
    assert!(project.worlds[0].levels[0]
        .iter_entities()
        .all(|e| e.def_id != enemy_id));
    assert_eq!(project.worlds[0].levels[0].iter_entities().count(), 1);
    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("health"),
        Some(&FieldValue::Int(10))
    );

    stack.undo(&mut project).expect("undo cascade");
    assert_eq!(project, snapshot);
}

#[test]
fn cascade_clears_references_from_surviving_entities_and_restores_them() {
    let fixture = fixture();
    let mut project = fixture.project;
    let level_id = fixture.level_id;
    let snapshot = {
        let mut clone = project.clone();
        if let Some(player) = clone
            .level_mut(level_id)
            .and_then(|l| l.entity_mut(EntityId::new(2)))
        {
            player.fields.set(
                "friend",
                FieldValue::Array(vec![
                    FieldValue::EntityRef(Some(EntityId::new(3))),
                    FieldValue::EntityRef(Some(EntityId::new(1))),
                ]),
            );
        }
        clone
    };
    // Player (id 2) points at the second enemy (id 3) through an array field.
    if let Some(player) = project
        .level_mut(level_id)
        .and_then(|l| l.entity_mut(EntityId::new(2)))
    {
        player.fields.set(
            "friend",
            FieldValue::Array(vec![
                FieldValue::EntityRef(Some(EntityId::new(3))),
                FieldValue::EntityRef(Some(EntityId::new(1))),
            ]),
        );
    }

    let enemy_def_id = EntityDefId::new(3);
    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(RemoveEntityDefCommand::cascade(enemy_def_id)),
            &mut project,
        )
        .expect("cascade");

    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("friend"),
        Some(&FieldValue::Array(vec![
            FieldValue::EntityRef(None),
            FieldValue::EntityRef(None),
        ]))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn add_field_seeds_defaults_on_every_instance_and_reverts() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let field = FieldDef {
        name: "speed".to_string(),
        field_type: FieldType::Float,
        default_value: Some(FieldValue::Float(1.5)),
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(AddEntityFieldCommand::new(def_id, field)),
            &mut project,
        )
        .expect("add field");

    let def = project.defs.entity(def_id).expect("def");
    assert!(def.field_def("speed").is_some());
    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("speed"),
        Some(&FieldValue::Float(1.5))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn change_field_type_migrates_values_and_reverts() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let updated = FieldDef {
        name: "health".to_string(),
        field_type: FieldType::String,
        default_value: Some(FieldValue::String("10".to_string())),
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(UpdateEntityFieldCommand::new(def_id, "health", updated)),
            &mut project,
        )
        .expect("update field");

    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("health"),
        Some(&FieldValue::String("10".to_string()))
    );
    assert_eq!(
        project
            .defs
            .entity(def_id)
            .and_then(|def| def.field_def("health"))
            .map(|f| f.field_type.clone()),
        Some(FieldType::String)
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn rename_field_moves_values_and_reverts() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let renamed = FieldDef {
        name: "hp".to_string(),
        field_type: FieldType::Int,
        default_value: Some(FieldValue::Int(10)),
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(UpdateEntityFieldCommand::new(def_id, "health", renamed)),
            &mut project,
        )
        .expect("rename");

    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("health"),
        None
    );
    assert_eq!(
        entity(&project, EntityId::new(2)).fields.get("hp"),
        Some(&FieldValue::Int(10))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn remove_field_drops_values_and_restores_them_on_undo() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(RemoveEntityFieldCommand::new(def_id, "health")),
            &mut project,
        )
        .expect("remove field");

    assert!(entity(&project, EntityId::new(2))
        .fields
        .get("health")
        .is_none());
    assert!(project
        .defs
        .entity(def_id)
        .expect("def")
        .field_def("health")
        .is_none());

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn field_default_edits_merge_per_stroke() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let first = FieldDef {
        name: "health".to_string(),
        field_type: FieldType::Int,
        default_value: Some(FieldValue::Int(11)),
    };
    let second = FieldDef {
        name: "health".to_string(),
        field_type: FieldType::Int,
        default_value: Some(FieldValue::Int(12)),
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(UpdateEntityFieldCommand::with_stroke(
                def_id, "health", first, 5,
            )),
            &mut project,
        )
        .expect("first");
    stack
        .execute(
            Box::new(UpdateEntityFieldCommand::with_stroke(
                def_id, "health", second, 5,
            )),
            &mut project,
        )
        .expect("merge");

    assert_eq!(
        project
            .defs
            .entity(def_id)
            .and_then(|d| d.field_def("health"))
            .and_then(|f| f.default_value.clone()),
        Some(FieldValue::Int(12))
    );

    stack.undo(&mut project).expect("one merged step");
    assert_eq!(project, snapshot);
}

#[test]
fn stroke_tracker_forgets_quiet_frames() {
    let mut tracker = StrokeTracker::default();
    tracker.begin_frame();
    let first = tracker.id(1_usize);
    tracker.end_frame();

    tracker.begin_frame();
    tracker.end_frame();

    tracker.begin_frame();
    assert_ne!(tracker.id(1_usize), first);
}

#[test]
fn helpers_find_instances_and_allocate_ids() {
    let fixture = fixture();
    let project = fixture.project;
    assert_eq!(
        count_entity_def_instances(&project, fixture.enemy_def_id),
        2
    );
    assert_eq!(
        lc_feature_entities::find_entity_def_instances(&project, fixture.enemy_def_id).len(),
        2
    );
    assert_eq!(next_entity_id(&project), EntityId::new(4));
}

#[test]
fn field_editors_render_every_field_type_headlessly() {
    let fixture = fixture();
    let choices = vec![EntityChoice {
        id: EntityId::new(2),
        label: "Player #2".to_string(),
    }];
    let ctx = FieldEditorContext::new(&fixture.project.defs.enums, &choices);

    let mut values = [
        (FieldType::Int, FieldValue::Int(3)),
        (FieldType::Float, FieldValue::Float(1.25)),
        (FieldType::Bool, FieldValue::Bool(true)),
        (FieldType::String, FieldValue::String("hello".to_string())),
        (FieldType::Color, FieldValue::Color([1, 2, 3, 255])),
        (
            FieldType::Enum(fixture.enum_id),
            FieldValue::Enum("Ally".to_string()),
        ),
        (FieldType::Point, FieldValue::Point(IVec2::new(2, 3))),
        (
            FieldType::EntityRef,
            FieldValue::EntityRef(Some(EntityId::new(2))),
        ),
        (
            FieldType::Array(Box::new(FieldType::Int)),
            FieldValue::Array(vec![FieldValue::Int(1)]),
        ),
    ];

    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        for (field_type, value) in &mut values {
            let response = field_value_editor(ui, value, field_type, &ctx);
            assert!(!response.changed, "no interaction means no change");
        }
    });
    output.textures_delta.clear();
}

#[test]
fn field_type_editor_reports_no_change_without_interaction() {
    let fixture = fixture();
    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let current = FieldType::Int;
        assert_eq!(
            field_type_editor(ui, &current, &fixture.project.defs.enums),
            None
        );
    });
    output.textures_delta.clear();
}

#[test]
fn entity_def_document_renders_headlessly_and_emits_nothing() {
    let fixture = fixture();
    let project = fixture.project;
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;

    let mut view = EntityDefDocumentView::new(fixture.entity_def_id);
    assert_eq!(view.title(&project), "Entity: Player".to_string());
    assert!(view.is_valid(&project));
    assert_eq!(view.doc_kind(), DocKind::EntityDef(fixture.entity_def_id));

    let egui_ctx = egui::Context::default();
    let mut output = egui_ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending,
            &assets,
        );
        view.ui(ui, &mut cx);
    });
    output.textures_delta.clear();

    assert!(events.is_empty());
    assert!(pending.is_empty());
}

#[test]
fn register_exposes_the_entity_def_document_factory() {
    let mut registry = Registry::new();
    register(&mut registry);

    let doc = registry
        .create_document(DocKind::EntityDef(EntityDefId::new(42)))
        .expect("entity def document factory");
    assert_eq!(doc.doc_kind(), DocKind::EntityDef(EntityDefId::new(42)));
    assert!(registry.create_document(DocKind::ProjectSettings).is_none());
}

#[test]
fn changing_field_type_migrates_the_stored_default_value() {
    let fixture = fixture();
    let mut project = fixture.project;
    let snapshot = project.clone();
    let def_id = fixture.entity_def_id;

    let updated = FieldDef {
        name: "health".to_string(),
        field_type: FieldType::String,
        // A stale default variant must be coerced with the new type.
        default_value: Some(FieldValue::Int(10)),
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(UpdateEntityFieldCommand::new(def_id, "health", updated)),
            &mut project,
        )
        .expect("update field");

    assert_eq!(
        project
            .defs
            .entity(def_id)
            .and_then(|def| def.field_def("health"))
            .and_then(|field| field.default_value.clone()),
        Some(FieldValue::String("10".to_string()))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn cascade_clears_refs_in_level_fields_and_field_defaults() {
    let fixture = fixture();
    let mut project = fixture.project;
    let level_id = fixture.level_id;
    let player_def_id = fixture.entity_def_id;

    project
        .level_mut(level_id)
        .expect("level")
        .fields
        .set("focus", FieldValue::EntityRef(Some(EntityId::new(1))));
    project
        .defs
        .entity_mut(player_def_id)
        .expect("def")
        .fields
        .push(FieldDef {
            name: "rival".to_string(),
            field_type: FieldType::EntityRef,
            default_value: Some(FieldValue::EntityRef(Some(EntityId::new(3)))),
        });
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(RemoveEntityDefCommand::cascade(fixture.enemy_def_id)),
            &mut project,
        )
        .expect("cascade");

    assert_eq!(
        project.level(level_id).expect("level").fields.get("focus"),
        Some(&FieldValue::EntityRef(None))
    );
    assert_eq!(
        project
            .defs
            .entity(player_def_id)
            .and_then(|def| def.field_def("rival"))
            .and_then(|field| field.default_value.clone()),
        Some(FieldValue::EntityRef(None))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}
