//! Integration tests for entity layer commands, tools, and the floating
//! inspector section.

use egui::{Pos2, Rect, Sense, Vec2};
use glam::{IVec2, UVec2, Vec2 as GlamVec2};
use lc_core::command::{Command as _, DocKind, UndoStack};
use lc_core::defs::{EntityDef, LayerDef};
use lc_core::fields::{FieldDef, FieldType, FieldValue, FieldValues};
use lc_core::id::{EntityDefId, EntityId, IdGenerator, LayerDefId, LevelId, WorldId};
use lc_core::layer::{EntityInstance, LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_entities::field_defs::default_field_values;
use lc_feature_entities::field_editors::{field_value_editor, FieldEditorContext};
use lc_feature_entities::field_instance_commands::{
    EntityFieldUpdate, SetEntityFieldValuesCommand,
};
use lc_feature_entities::helpers::next_entity_id;
use lc_feature_entities::instance_commands::{
    AddEntityInstancesCommand, RemoveEntityInstancesCommand, SetEntityPositionsCommand,
    SetEntitySizesCommand,
};
use lc_feature_entities::panel::EntitiesPanel;
use lc_feature_entities::stroke::FocusStrokeTracker;
use lc_feature_entities::tools::{
    active_entities_layer, clamp_position, grid_size, local_pos, pick_entity_at, placement_def_id,
    selected_in_level, snap_to_grid, EntityCanvasTool,
};
use lc_feature_entities::{register, EntityInspectorSection};
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::canvas::{CanvasCtx, CanvasTool};
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::inspector::InspectorSection;
use lc_ui_api::registry::Registry;
use lc_ui_api::selection::Selection;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::PanelView;
use lc_ui_api::viewport::Viewport;

struct Fixture {
    project: Project,
    level_id: LevelId,
    layer_def_id: LayerDefId,
    player_def_id: EntityDefId,
    enemy_def_id: EntityDefId,
}

fn fixture() -> Fixture {
    let mut gen = IdGenerator::new();
    let mut project = Project::new("Entity Layer Test");

    let player_def_id = gen.next_entity_def_id();
    project.defs.entities.push(EntityDef {
        id: player_def_id,
        name: "Player".to_string(),
        size_px: UVec2::splat(16),
        color: [120, 200, 255, 255],
        pivot: GlamVec2::splat(0.5),
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
        color: [255, 110, 110, 255],
        pivot: GlamVec2::splat(0.5),
        tile: None,
        fields: vec![
            FieldDef {
                name: "health".to_string(),
                field_type: FieldType::Int,
                default_value: Some(FieldValue::Int(5)),
            },
            FieldDef {
                name: "name".to_string(),
                field_type: FieldType::String,
                default_value: None,
            },
        ],
    });

    let layer_def_id = gen.next_layer_def_id();
    project.defs.layers.push(LayerDef {
        id: layer_def_id,
        name: "Actors".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::splat(16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });

    let level_id = gen.next_level_id();
    let mut level = Level::new(level_id, "Arena", UVec2::new(256, 256));
    level.world_pos = IVec2::new(100, 50);

    let mut player_values = FieldValues::new();
    player_values.set("health", FieldValue::Int(10));
    let mut enemy_a = FieldValues::new();
    enemy_a.set("health", FieldValue::Int(5));
    // Weak reference used by the deletion test.
    enemy_a.set("target", FieldValue::EntityRef(Some(EntityId::new(3))));
    let mut enemy_b = FieldValues::new();
    enemy_b.set("health", FieldValue::Int(5));

    level.layers.push(LayerInstance::new(
        layer_def_id,
        LayerData::Entities(vec![
            EntityInstance {
                id: EntityId::new(1),
                def_id: player_def_id,
                pos_px: IVec2::new(16, 16),
                size_px: UVec2::splat(16),
                fields: player_values,
            },
            EntityInstance {
                id: EntityId::new(2),
                def_id: enemy_def_id,
                pos_px: IVec2::new(64, 64),
                size_px: UVec2::splat(16),
                fields: enemy_a,
            },
            EntityInstance {
                id: EntityId::new(3),
                def_id: enemy_def_id,
                pos_px: IVec2::new(96, 64),
                size_px: UVec2::splat(16),
                fields: enemy_b,
            },
        ]),
    ));
    level
        .fields
        .set("focus", FieldValue::EntityRef(Some(EntityId::new(3))));

    let mut world = World::new(WorldId::new(1), "World");
    world.levels.push(level);
    project.worlds.push(world);

    Fixture {
        project,
        level_id,
        layer_def_id,
        player_def_id,
        enemy_def_id,
    }
}

fn level<'a>(project: &'a Project, fixture: &Fixture) -> &'a Level {
    project.level(fixture.level_id).expect("level exists")
}

fn entity<'a>(project: &'a Project, fixture: &Fixture, id: u64) -> &'a EntityInstance {
    level(project, fixture)
        .entity(EntityId::new(id))
        .expect("entity exists")
}

#[test]
fn add_entity_instances_applies_and_reverts() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let snapshot = project.clone();

    let def = project
        .defs
        .entity(fixture.player_def_id)
        .cloned()
        .expect("def");
    let instances = vec![
        EntityInstance {
            id: EntityId::new(10),
            def_id: def.id,
            pos_px: IVec2::new(32, 32),
            size_px: def.size_px,
            fields: default_field_values(&def, &project.defs.enums),
        },
        EntityInstance {
            id: EntityId::new(11),
            def_id: def.id,
            pos_px: IVec2::new(48, 48),
            size_px: def.size_px,
            fields: default_field_values(&def, &project.defs.enums),
        },
    ];

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(AddEntityInstancesCommand::new(
                fixture.level_id,
                fixture.layer_def_id,
                instances,
            )),
            &mut project,
        )
        .expect("place entities");

    assert_eq!(level(&project, &fixture).iter_entities().count(), 5);
    assert_eq!(
        entity(&project, &fixture, 10).fields.get("health"),
        Some(&FieldValue::Int(10))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn add_entity_instances_rejects_duplicate_ids() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let def = project
        .defs
        .entity(fixture.player_def_id)
        .cloned()
        .expect("def");
    let duplicate = EntityInstance {
        id: EntityId::new(2),
        def_id: def.id,
        pos_px: IVec2::ZERO,
        size_px: def.size_px,
        fields: FieldValues::new(),
    };

    let mut stack = UndoStack::new();
    let error = stack
        .execute(
            Box::new(AddEntityInstancesCommand::new(
                fixture.level_id,
                fixture.layer_def_id,
                vec![duplicate],
            )),
            &mut project,
        )
        .expect_err("duplicate id rejected");
    assert!(format!("{error}").contains("already exists"));
}

#[test]
fn remove_entity_instances_clears_refs_and_restores_them() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    // A default value pointing at the same entity is also scrubbed.
    if let Some(def) = project.defs.entity_mut(fixture.player_def_id) {
        def.fields.push(FieldDef {
            name: "rival".to_string(),
            field_type: FieldType::EntityRef,
            default_value: Some(FieldValue::EntityRef(Some(EntityId::new(3)))),
        });
    }
    // References nested inside arrays are scrubbed too, both on instances and
    // on level fields.
    if let Some(level) = project.level_mut(fixture.level_id) {
        if let Some(entity) = level.entity_mut(EntityId::new(2)) {
            entity.fields.set(
                "squad",
                FieldValue::Array(vec![
                    FieldValue::EntityRef(Some(EntityId::new(3))),
                    FieldValue::EntityRef(Some(EntityId::new(1))),
                ]),
            );
        }
        level.fields.set(
            "watch",
            FieldValue::Array(vec![FieldValue::EntityRef(Some(EntityId::new(3)))]),
        );
    }
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(RemoveEntityInstancesCommand::new(
                fixture.level_id,
                vec![EntityId::new(3)],
            )),
            &mut project,
        )
        .expect("delete entity");

    assert!(level(&project, &fixture).entity(EntityId::new(3)).is_none());
    assert_eq!(
        entity(&project, &fixture, 2).fields.get("target"),
        Some(&FieldValue::EntityRef(None))
    );
    assert_eq!(
        entity(&project, &fixture, 2).fields.get("squad"),
        Some(&FieldValue::Array(vec![
            FieldValue::EntityRef(None),
            FieldValue::EntityRef(Some(EntityId::new(1))),
        ]))
    );
    assert_eq!(
        level(&project, &fixture).fields.get("focus"),
        Some(&FieldValue::EntityRef(None))
    );
    assert_eq!(
        level(&project, &fixture).fields.get("watch"),
        Some(&FieldValue::Array(vec![FieldValue::EntityRef(None)]))
    );
    assert_eq!(
        project
            .defs
            .entity(fixture.player_def_id)
            .and_then(|def| def.field_def("rival"))
            .and_then(|field| field.default_value.clone()),
        Some(FieldValue::EntityRef(None))
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn remove_many_entities_restores_original_order() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(RemoveEntityInstancesCommand::new(
                fixture.level_id,
                vec![EntityId::new(1), EntityId::new(3)],
            )),
            &mut project,
        )
        .expect("delete entities");
    assert_eq!(
        level(&project, &fixture)
            .iter_entities()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        vec![EntityId::new(2)]
    );

    stack.undo(&mut project).expect("undo");
    assert_eq!(project, snapshot);
}

#[test]
fn move_command_merges_per_stroke_and_reverts() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntityPositionsCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), IVec2::new(32, 32))],
                7,
            )),
            &mut project,
        )
        .expect("first move");
    stack
        .execute(
            Box::new(SetEntityPositionsCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), IVec2::new(48, 48))],
                7,
            )),
            &mut project,
        )
        .expect("merge move");

    assert_eq!(entity(&project, &fixture, 1).pos_px, IVec2::new(48, 48));
    stack.undo(&mut project).expect("one merged step");
    assert_eq!(project, snapshot);
}

#[test]
fn move_command_with_different_stroke_keeps_two_steps() {
    let fixture = fixture();
    let mut project = fixture.project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntityPositionsCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), IVec2::new(32, 32))],
                1,
            )),
            &mut project,
        )
        .expect("first move");
    stack
        .execute(
            Box::new(SetEntityPositionsCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), IVec2::new(48, 48))],
                2,
            )),
            &mut project,
        )
        .expect("second move");

    stack.undo(&mut project).expect("undo second");
    assert_eq!(entity(&project, &fixture, 1).pos_px, IVec2::new(32, 32));
}

#[test]
fn resize_command_applies_reverts_and_merges() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let snapshot = project.clone();

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntitySizesCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), UVec2::new(32, 48))],
                3,
            )),
            &mut project,
        )
        .expect("resize");
    stack
        .execute(
            Box::new(SetEntitySizesCommand::with_stroke(
                fixture.level_id,
                vec![(EntityId::new(1), UVec2::new(64, 48))],
                3,
            )),
            &mut project,
        )
        .expect("merge resize");

    assert_eq!(entity(&project, &fixture, 1).size_px, UVec2::new(64, 48));
    stack.undo(&mut project).expect("single resize step");
    assert_eq!(project, snapshot);
}

#[test]
fn field_values_merge_per_stroke_and_revert() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let snapshot = project.clone();

    let update = |value: i64| {
        vec![
            EntityFieldUpdate {
                entity_id: EntityId::new(1),
                field_name: "health".to_string(),
                value: FieldValue::Int(value),
            },
            EntityFieldUpdate {
                entity_id: EntityId::new(2),
                field_name: "health".to_string(),
                value: FieldValue::Int(value),
            },
        ]
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::with_stroke(
                fixture.level_id,
                update(11),
                9,
            )),
            &mut project,
        )
        .expect("first edit");
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::with_stroke(
                fixture.level_id,
                update(12),
                9,
            )),
            &mut project,
        )
        .expect("merge edit");

    assert_eq!(
        entity(&project, &fixture, 1).fields.get("health"),
        Some(&FieldValue::Int(12))
    );
    assert_eq!(
        entity(&project, &fixture, 2).fields.get("health"),
        Some(&FieldValue::Int(12))
    );
    stack.undo(&mut project).expect("one merged edit");
    assert_eq!(project, snapshot);
}

#[test]
fn field_values_without_matching_stroke_keep_two_steps() {
    let fixture = fixture();
    let mut project = fixture.project.clone();

    let update = |value: i64| {
        vec![EntityFieldUpdate {
            entity_id: EntityId::new(1),
            field_name: "health".to_string(),
            value: FieldValue::Int(value),
        }]
    };

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::with_stroke(
                fixture.level_id,
                update(11),
                1,
            )),
            &mut project,
        )
        .expect("first");
    stack
        .execute(
            Box::new(SetEntityFieldValuesCommand::with_stroke(
                fixture.level_id,
                update(12),
                2,
            )),
            &mut project,
        )
        .expect("second");

    stack.undo(&mut project).expect("undo second");
    assert_eq!(
        entity(&project, &fixture, 1).fields.get("health"),
        Some(&FieldValue::Int(11))
    );
}

#[test]
fn tool_helpers_snap_pick_and_resolve_placement() {
    let fixture = fixture();
    let project = fixture.project.clone();

    assert_eq!(
        snap_to_grid(IVec2::new(31, 33), IVec2::splat(16)),
        IVec2::new(32, 32)
    );
    assert_eq!(
        clamp_position(IVec2::new(250, 250), UVec2::splat(16), UVec2::new(256, 256)),
        IVec2::new(240, 240)
    );

    let arena = level(&project, &fixture);
    assert_eq!(
        pick_entity_at(arena, IVec2::new(20, 20)),
        Some(EntityId::new(1))
    );
    assert_eq!(pick_entity_at(arena, IVec2::new(200, 200)), None);

    assert_eq!(
        grid_size(&project, arena, fixture.layer_def_id),
        IVec2::splat(16)
    );

    let session = SessionState {
        active_tool_name: format!("Place #{}", fixture.enemy_def_id.0),
        ..Default::default()
    };
    let undo = UndoStack::new();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;
    let cx = EditorCtx::new(
        &project,
        &undo,
        &session,
        &mut events,
        &mut pending,
        &assets,
    );
    assert_eq!(
        placement_def_id(&cx, cx.active_tool_name()),
        Some(fixture.enemy_def_id)
    );
    assert_eq!(
        next_entity_id(&project),
        EntityId::new(4),
        "duplicate and placement ids start after the highest existing id"
    );
}

struct CanvasClick {
    rect: Rect,
    pointer: Pos2,
    world_pos: IVec2,
    layer_def_id: LayerDefId,
    level_id: LevelId,
    modifiers: egui::Modifiers,
}

fn click_canvas(
    tool: &mut dyn CanvasTool,
    project: &Project,
    session: &SessionState,
    click: CanvasClick,
) -> (Vec<Box<dyn lc_core::command::Command>>, Vec<EditorEvent>) {
    let CanvasClick {
        rect,
        pointer,
        world_pos,
        layer_def_id,
        level_id,
        modifiers,
    } = click;
    let ctx = egui::Context::default();
    let undo = UndoStack::new();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;
    let viewport = Viewport::new(rect);

    for frame in 0..3 {
        let mut raw = egui::RawInput {
            screen_rect: Some(rect),
            time: Some(frame as f64 * 0.1),
            ..Default::default()
        };
        raw.events = match frame {
            0 => vec![egui::Event::PointerMoved(pointer)],
            1 => vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::PointerButton {
                    pos: pointer,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            _ => vec![egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
        };

        let mut output = ctx.run_ui(raw, |ui| {
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            let mut cx =
                EditorCtx::new(project, &undo, session, &mut events, &mut pending, &assets);
            let mut cv = CanvasCtx {
                viewport: &viewport,
                pointer_pos: Some(pointer),
                world_pos: Some(world_pos),
                grid_pos: None,
                modifiers,
                response: &response,
                active_layer_id: Some(layer_def_id),
                active_level_id: level_id,
            };
            tool.on_input(&mut cv, &mut cx);
        });
        output.textures_delta.clear();
    }

    (pending, events)
}

#[test]
fn entity_tool_places_on_canvas_click() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_tool_name: format!("Place #{}", fixture.enemy_def_id.0),
        active_layer: Some(fixture.layer_def_id),
        active_level: Some(fixture.level_id),
        ..Default::default()
    };

    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let (mut pending, events) = click_canvas(
        &mut tool,
        &project,
        &session,
        CanvasClick {
            rect,
            pointer: Pos2::new(50.0, 50.0),
            world_pos: IVec2::new(132, 82),
            layer_def_id: fixture.layer_def_id,
            level_id: fixture.level_id,
            modifiers: egui::Modifiers::default(),
        },
    );

    assert_eq!(pending.len(), 1, "placement emits one command");
    assert!(events
        .iter()
        .any(|event| matches!(event, EditorEvent::SelectionChanged(Selection::Entities(ids)) if ids.len() == 1)));

    let command = pending.remove(0);
    let mut stack = UndoStack::new();
    stack
        .execute(command, &mut project)
        .expect("placement command applies");
    let placed = level(&project, &fixture)
        .entity(EntityId::new(4))
        .expect("placed at next id");
    assert_eq!(placed.def_id, fixture.enemy_def_id);
    assert_eq!(placed.pos_px, IVec2::new(32, 32));
    assert_eq!(
        placed.fields.get("health"),
        Some(&FieldValue::Int(5)),
        "instance inherits definition default"
    );
}

#[test]
fn entity_tool_selects_on_click() {
    let fixture = fixture();
    let project = fixture.project.clone();
    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_tool_name: "Select".to_string(),
        active_layer: Some(fixture.layer_def_id),
        active_level: Some(fixture.level_id),
        ..Default::default()
    };

    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let (pending, events) = click_canvas(
        &mut tool,
        &project,
        &session,
        CanvasClick {
            rect,
            pointer: Pos2::new(20.0, 20.0),
            // World position of entity 1: level origin + local (20, 20).
            world_pos: IVec2::new(120, 70),
            layer_def_id: fixture.layer_def_id,
            level_id: fixture.level_id,
            modifiers: egui::Modifiers::default(),
        },
    );

    assert!(pending.is_empty());
    assert_eq!(
        events,
        vec![EditorEvent::SelectionChanged(Selection::Entities(vec![
            EntityId::new(1)
        ]))]
    );
}

#[test]
fn inspector_applies_to_entity_selection_and_renders_headlessly() {
    let fixture = fixture();
    let project = fixture.project.clone();
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;
    let mut section = EntityInspectorSection::new();
    let selection = Selection::Entities(vec![EntityId::new(1), EntityId::new(2)]);

    assert!(section.applies(&selection));
    assert!(!section.applies(&Selection::None));

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending,
            &assets,
        );
        section.ui(ui, &mut cx, &selection);
    });
    output.textures_delta.clear();

    assert!(events.is_empty(), "rendering does not emit events");
    assert!(pending.is_empty(), "rendering does not execute commands");
}

#[test]
fn register_exposes_panel_handler_and_inspector() {
    let mut registry = Registry::new();
    register(&mut registry);

    assert!(registry.find_panel(&EntitiesPanel::new().id()).is_some());
    assert_eq!(
        registry
            .layer_handler(LayerKind::Entities)
            .map(|handler| handler.kind()),
        Some(LayerKind::Entities)
    );
    assert!(registry
        .inspector_sections()
        .iter()
        .any(|section| section.applies(&Selection::Entities(vec![EntityId::new(1)]))));
    assert!(registry
        .create_document(DocKind::EntityDef(EntityDefId::new(1)))
        .is_some());
}

#[test]
fn active_layer_and_local_position_helpers() {
    let fixture = fixture();
    let project = fixture.project.clone();
    let arena = level(&project, &fixture);
    let session = SessionState {
        active_layer: Some(fixture.layer_def_id),
        ..Default::default()
    };
    let undo = UndoStack::new();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;

    let ctx = egui::Context::default();
    let mut checked = false;
    let mut output = ctx.run_ui(Default::default(), |ui| {
        let response = ui.allocate_response(Vec2::new(10.0, 10.0), Sense::hover());
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending,
            &assets,
        );
        let viewport = Viewport::new(Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)));
        let cv = CanvasCtx {
            viewport: &viewport,
            pointer_pos: Some(Pos2::new(10.0, 10.0)),
            world_pos: Some(IVec2::new(132, 82)),
            grid_pos: None,
            modifiers: egui::Modifiers::default(),
            response: &response,
            active_layer_id: Some(fixture.layer_def_id),
            active_level_id: fixture.level_id,
        };

        assert_eq!(
            active_entities_layer(&cx, &cv, arena),
            Some(fixture.layer_def_id)
        );
        assert_eq!(local_pos(&cv, arena), Some(IVec2::new(32, 32)));
        assert_eq!(
            selected_in_level(
                &Selection::Entities(vec![EntityId::new(1), EntityId::new(99)]),
                arena
            ),
            vec![EntityId::new(1)]
        );
        checked = true;
    });
    output.textures_delta.clear();
    assert!(checked);
}

fn add_empty_entities_level(
    project: &mut Project,
    layer_def_id: LayerDefId,
    id: LevelId,
) -> LevelId {
    let mut level = Level::new(id, "Second", UVec2::splat(128));
    level.world_pos = IVec2::new(500, 0);
    level.layers.push(LayerInstance::new(
        layer_def_id,
        LayerData::Entities(Vec::new()),
    ));
    project
        .worlds
        .first_mut()
        .expect("fixture world")
        .levels
        .push(level);
    id
}

struct Shortcut {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    key: egui::Key,
    modifiers: egui::Modifiers,
    text_focused: bool,
}

fn run_shortcut(
    tool: &mut dyn CanvasTool,
    project: &Project,
    session: &SessionState,
    shortcut: Shortcut,
) -> (Vec<Box<dyn lc_core::command::Command>>, Vec<EditorEvent>) {
    let Shortcut {
        level_id,
        layer_def_id,
        key,
        modifiers,
        text_focused,
    } = shortcut;
    let ctx = egui::Context::default();
    let undo = UndoStack::new();
    let mut events = Vec::new();
    let mut pending = Vec::new();
    let assets = NoopAssetService;
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let viewport = Viewport::new(rect);
    let text_id = egui::Id::new("inspector-text");
    let mut text = String::from("Player");

    for frame in 0..3 {
        let mut raw = egui::RawInput {
            screen_rect: Some(rect),
            time: Some(frame as f64 * 0.1),
            ..Default::default()
        };
        raw.events = if frame == 1 {
            vec![
                egui::Event::ModifiersChanged(modifiers),
                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                },
            ]
        } else if frame == 2 {
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers,
            }]
        } else {
            Vec::new()
        };

        let mut output = ctx.run_ui(raw, |ui| {
            if text_focused {
                let _ = ui.add(egui::TextEdit::singleline(&mut text).id(text_id));
                if frame == 0 {
                    ui.memory_mut(|memory| memory.request_focus(text_id));
                }
            }
            let response = ui.allocate_rect(rect, Sense::click_and_drag());
            let mut cx =
                EditorCtx::new(project, &undo, session, &mut events, &mut pending, &assets);
            let mut cv = CanvasCtx {
                viewport: &viewport,
                pointer_pos: Some(rect.center()),
                world_pos: Some(IVec2::new(20, 20)),
                grid_pos: None,
                modifiers: egui::Modifiers::default(),
                response: &response,
                active_layer_id: Some(layer_def_id),
                active_level_id: level_id,
            };
            tool.on_input(&mut cv, &mut cx);
        });
        output.textures_delta.clear();
    }

    (pending, events)
}

#[test]
fn shortcuts_are_ignored_while_a_field_has_keyboard_focus() {
    let fixture = fixture();
    let project = fixture.project.clone();
    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_level: Some(fixture.level_id),
        active_layer: Some(fixture.layer_def_id),
        selection: Selection::Entities(vec![EntityId::new(1)]),
        ..Default::default()
    };

    let (pending, events) = run_shortcut(
        &mut tool,
        &project,
        &session,
        Shortcut {
            level_id: fixture.level_id,
            layer_def_id: fixture.layer_def_id,
            key: egui::Key::Backspace,
            modifiers: egui::Modifiers::default(),
            text_focused: true,
        },
    );

    assert!(
        pending.is_empty(),
        "Backspace in a focused field must not delete the entity"
    );
    assert!(events.is_empty(), "focused field must keep the selection");
}

#[test]
fn shortcuts_delete_when_no_field_has_focus() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_level: Some(fixture.level_id),
        active_layer: Some(fixture.layer_def_id),
        selection: Selection::Entities(vec![EntityId::new(1)]),
        ..Default::default()
    };

    let (mut pending, events) = run_shortcut(
        &mut tool,
        &project,
        &session,
        Shortcut {
            level_id: fixture.level_id,
            layer_def_id: fixture.layer_def_id,
            key: egui::Key::Backspace,
            modifiers: egui::Modifiers::default(),
            text_focused: false,
        },
    );

    assert_eq!(
        pending.len(),
        1,
        "Backspace deletes without a focused field"
    );
    assert!(events
        .iter()
        .any(|event| matches!(event, EditorEvent::SelectionChanged(Selection::None))));

    let mut stack = UndoStack::new();
    stack
        .execute(pending.remove(0), &mut project)
        .expect("delete applies");
    assert!(level(&project, &fixture).entity(EntityId::new(1)).is_none());
}

#[test]
fn numeric_drag_keeps_its_stroke_across_a_pause() {
    let mut value = FieldValue::Int(5);
    let editor_ctx = FieldEditorContext::new(&[], &[]);
    let ctx = egui::Context::default();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0));
    let mut widget_rect = Rect::NOTHING;
    let mut responses = Vec::new();

    for frame in 0..4 {
        let center = widget_rect.center();
        let mut raw = egui::RawInput {
            screen_rect: Some(screen),
            time: Some(frame as f64 * 0.1),
            ..Default::default()
        };
        raw.events = match frame {
            1 => vec![
                egui::Event::PointerMoved(center),
                egui::Event::PointerButton {
                    pos: center,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            2 => vec![egui::Event::PointerMoved(center + Vec2::new(40.0, 0.0))],
            _ => Vec::new(),
        };

        let mut output = ctx.run_ui(raw, |ui| {
            let response = field_value_editor(ui, &mut value, &FieldType::Int, &editor_ctx);
            widget_rect = ui.min_rect();
            responses.push(response);
        });
        output.textures_delta.clear();
    }

    let dragging = responses[2];
    let paused = responses[3];
    assert!(dragging.changed && dragging.dragging, "drag frame");
    assert!(!paused.changed, "pause frame does not change the value");
    assert!(paused.dragging, "pause frame is still part of the drag");
    assert!(
        paused.keeps_stroke(),
        "a paused drag keeps its stroke alive"
    );

    let mut tracker: FocusStrokeTracker<String> = FocusStrokeTracker::default();
    tracker.begin_frame();
    let first = tracker
        .id(
            "value".to_string(),
            dragging.keeps_stroke(),
            dragging.changed,
        )
        .expect("stroke on change");
    tracker.end_frame();

    tracker.begin_frame();
    let resumed = tracker.id("value".to_string(), paused.keeps_stroke(), paused.changed);
    tracker.end_frame();
    assert_eq!(
        resumed,
        Some(first),
        "a pause inside one drag must not split the undo step"
    );
}

#[test]
fn shift_click_does_not_keep_ids_from_another_level() {
    let mut fixture = fixture();
    let layer_def_id = fixture.layer_def_id;
    let second = add_empty_entities_level(&mut fixture.project, layer_def_id, LevelId::new(42));
    let project = fixture.project.clone();
    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_tool_name: "Select".to_string(),
        active_level: Some(second),
        active_layer: Some(fixture.layer_def_id),
        selection: Selection::Entities(vec![EntityId::new(1)]),
        ..Default::default()
    };

    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
    let (pending, events) = click_canvas(
        &mut tool,
        &project,
        &session,
        CanvasClick {
            rect,
            pointer: Pos2::new(40.0, 40.0),
            world_pos: IVec2::new(540, 40),
            layer_def_id: fixture.layer_def_id,
            level_id: second,
            modifiers: egui::Modifiers {
                shift: true,
                ..Default::default()
            },
        },
    );

    assert!(pending.is_empty());
    assert_eq!(
        events,
        vec![EditorEvent::SelectionChanged(Selection::None)],
        "ids from level A must not survive a shift-click in level B"
    );
}

#[test]
fn duplicate_keeps_copies_on_the_source_layer() {
    let fixture = fixture();
    let mut project = fixture.project.clone();
    let extra_layer = LayerDefId::new(99);
    project.defs.layers.push(LayerDef {
        id: extra_layer,
        name: "Props".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::splat(16),
        tileset_id: None,
        int_grid_values: Vec::new(),
    });
    let def = project
        .defs
        .entity(fixture.player_def_id)
        .cloned()
        .expect("player def");
    let fields = default_field_values(&def, &project.defs.enums);
    project
        .level_mut(fixture.level_id)
        .expect("level")
        .layers
        .push(LayerInstance::new(
            extra_layer,
            LayerData::Entities(vec![EntityInstance {
                id: EntityId::new(4),
                def_id: def.id,
                pos_px: IVec2::new(16, 16),
                size_px: def.size_px,
                fields,
            }]),
        ));

    let mut tool = EntityCanvasTool::new();
    let session = SessionState {
        active_level: Some(fixture.level_id),
        // The active layer is deliberately the other Entities layer.
        active_layer: Some(fixture.layer_def_id),
        selection: Selection::Entities(vec![EntityId::new(4)]),
        ..Default::default()
    };

    let (mut pending, events) = run_shortcut(
        &mut tool,
        &project,
        &session,
        Shortcut {
            level_id: fixture.level_id,
            layer_def_id: fixture.layer_def_id,
            key: egui::Key::D,
            modifiers: egui::Modifiers {
                command: true,
                ..Default::default()
            },
            text_focused: false,
        },
    );

    assert_eq!(pending.len(), 1, "one copy command");
    assert!(events.iter().any(|event| matches!(
        event,
        EditorEvent::SelectionChanged(Selection::Entities(ids)) if ids == &vec![EntityId::new(5)]
    )));

    let mut stack = UndoStack::new();
    stack
        .execute(pending.remove(0), &mut project)
        .expect("duplicate applies");

    let level = level(&project, &fixture);
    let source_layer = level.layer(extra_layer).expect("source layer");
    let active_layer = level.layer(fixture.layer_def_id).expect("active layer");
    let LayerData::Entities(source_entities) = &source_layer.data else {
        panic!("source layer is not an Entities layer");
    };
    assert!(source_entities.iter().any(|e| e.id == EntityId::new(4)));
    assert!(
        source_entities.iter().any(|e| e.id == EntityId::new(5)),
        "the copy stays on the source layer"
    );
    let LayerData::Entities(active_entities) = &active_layer.data else {
        panic!("active layer is not an Entities layer");
    };
    assert!(
        !active_entities.iter().any(|e| e.id == EntityId::new(5)),
        "the copy is not regrouped onto the active layer"
    );
}

#[test]
fn add_and_remove_commands_do_not_merge() {
    let fixture = fixture();
    let project = fixture.project.clone();
    let def = project
        .defs
        .entity(fixture.player_def_id)
        .cloned()
        .expect("player def");
    let instance = |id: u64| EntityInstance {
        id: EntityId::new(id),
        def_id: def.id,
        pos_px: IVec2::ZERO,
        size_px: def.size_px,
        fields: FieldValues::new(),
    };

    let mut add =
        AddEntityInstancesCommand::new(fixture.level_id, fixture.layer_def_id, vec![instance(10)]);
    let other_add =
        AddEntityInstancesCommand::new(fixture.level_id, fixture.layer_def_id, vec![instance(11)]);
    assert!(!add.merge(&other_add));

    let mut remove = RemoveEntityInstancesCommand::new(fixture.level_id, vec![EntityId::new(1)]);
    let other_remove = RemoveEntityInstancesCommand::new(fixture.level_id, vec![EntityId::new(2)]);
    assert!(!remove.merge(&other_remove));
}
