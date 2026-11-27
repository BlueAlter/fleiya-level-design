//! Integration and headless UI tests for lc_feature_layers.

use glam::UVec2;
use lc_core::command::{Command, DocKind, UndoStack};
use lc_core::defs::{IntGridValueDef, LayerDef};
use lc_core::grid::Grid;
use lc_core::id::{IdGenerator, LayerDefId, LevelId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_layers::commands::{
    RemoveLayerDefCommand, ReorderLayerDefsCommand, ToggleLayerVisibilityCommand,
    UpdateLayerDefCommand,
};
use lc_feature_layers::helpers::{
    count_layer_def_usage, generate_unique_layer_name, next_int_grid_value_id,
};
use lc_feature_layers::panel::LayersPanel;
use lc_feature_layers::view::LayerDefDocumentView;
use lc_ui_api::assets::NoopAssetService;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::registry::Registry;
use lc_ui_api::session::SessionState;
use lc_ui_api::view::{DocumentView, PanelId, PanelView};

fn setup_test_project_with_layers() -> (Project, LayerDefId, LayerDefId, LevelId) {
    let mut gen = IdGenerator::new();
    let mut project = Project::new("Test Project");

    let l1_id = gen.next_layer_def_id();
    let l1_def = LayerDef {
        id: l1_id,
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![IntGridValueDef {
            value: 1,
            name: "Solid".to_string(),
            color: [220, 50, 50, 255],
            ..Default::default()
        }],
        ..Default::default()
    };

    let l2_id = gen.next_layer_def_id();
    let l2_def = LayerDef {
        id: l2_id,
        name: "Tiles_Foreground".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
        ..Default::default()
    };

    project.defs.layers.push(l1_def);
    project.defs.layers.push(l2_def);

    let world_id = WorldId(1);
    let mut world = World::new(world_id, "MainWorld");

    let level_id = LevelId(10);
    let mut level = Level::new(level_id, "Level_0", UVec2::new(320, 240));

    level.layers.push(LayerInstance::new(
        l1_id,
        LayerData::IntGrid(Grid::new(20, 15, 0)),
    ));
    level.layers.push(LayerInstance::new(
        l2_id,
        LayerData::Tiles(Grid::new(20, 15, None)),
    ));

    world.levels.push(level);
    project.worlds.push(world);

    (project, l1_id, l2_id, level_id)
}

#[test]
fn test_factory_and_panel_registration() {
    let mut reg = Registry::new();
    lc_feature_layers::register(&mut reg);

    // Factory creates LayerDefDocumentView for DocKind::LayerDef
    let layer_id = LayerDefId::new(42);
    let doc_view = reg.create_document(DocKind::LayerDef(layer_id));
    assert!(doc_view.is_some());
    assert_eq!(doc_view.unwrap().doc_kind(), DocKind::LayerDef(layer_id));

    // Factory returns None for other DocKinds
    assert!(reg.create_document(DocKind::ProjectSettings).is_none());

    // Panel is registered
    let panel = reg.find_panel(&PanelId::new("layers"));
    assert!(panel.is_some());
    assert_eq!(panel.unwrap().title(), "Layers");
}

#[test]
fn test_layer_def_document_view_properties() {
    let (mut project, l1_id, _, _) = setup_test_project_with_layers();
    let mut view = LayerDefDocumentView::new(l1_id);

    assert_eq!(view.doc_kind(), DocKind::LayerDef(l1_id));
    assert!(view.is_valid(&project));
    assert_eq!(view.title(&project), "Layer: Collisions");

    // Headless UI render
    let undo = UndoStack::new();
    let session = SessionState::default();
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            view.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    // Invalidation when def is deleted
    project.defs.layers.clear();
    assert!(!view.is_valid(&project));
    assert_eq!(view.title(&project), "Layer (deleted)");
}

#[test]
fn test_layers_panel_empty_state_when_no_active_level() {
    let (project, _, _, _) = setup_test_project_with_layers();
    let mut panel = LayersPanel::new();
    assert_eq!(panel.id().as_str(), LayersPanel::PANEL_ID);
    assert_eq!(panel.title(), "Layers");

    let undo = UndoStack::new();
    let session = SessionState {
        active_level: None,
        ..Default::default()
    };
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        assert!(cx.active_level().is_none());
        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();
    }

    assert!(events.is_empty());
    assert!(pending_commands.is_empty());
}

#[test]
fn test_layers_panel_with_active_level_and_selection() {
    let (project, l1_id, l2_id, level_id) = setup_test_project_with_layers();
    let mut panel = LayersPanel::new();

    let undo = UndoStack::new();
    let mut session = SessionState {
        active_level: Some(level_id),
        active_layer: Some(l1_id),
        ..Default::default()
    };
    let mut events = Vec::new();
    let mut pending_commands = Vec::new();
    let assets = NoopAssetService;

    let egui_ctx = egui::Context::default();
    {
        let mut cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );

        assert_eq!(cx.active_level(), Some(level_id));
        assert_eq!(cx.active_layer(), Some(l1_id));

        let mut output = egui_ctx.run_ui(Default::default(), |ui| {
            panel.ui(ui, &mut cx);
        });
        output.textures_delta.clear();

        // Emit layer selection event
        cx.emit(EditorEvent::SetActiveLayer(Some(l2_id)));
    }

    assert_eq!(events, vec![EditorEvent::SetActiveLayer(Some(l2_id))]);

    // Update session state as shell does and verify readability via cx.active_layer()
    session.active_layer = Some(l2_id);
    {
        let cx = EditorCtx::new(
            &project,
            &undo,
            &session,
            &mut events,
            &mut pending_commands,
            &assets,
        );
        assert_eq!(cx.active_layer(), Some(l2_id));
    }
}

#[test]
fn test_layer_visibility_command_revert() {
    let (mut project, l1_id, _, level_id) = setup_test_project_with_layers();

    assert!(
        project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    let cmd = ToggleLayerVisibilityCommand::new(level_id, l1_id);
    let mut undo = UndoStack::new();

    undo.execute(Box::new(cmd), &mut project).unwrap();
    assert!(
        !project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    undo.undo(&mut project).unwrap();
    assert!(
        project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );

    undo.redo(&mut project).unwrap();
    assert!(
        !project
            .level(level_id)
            .unwrap()
            .layer(l1_id)
            .unwrap()
            .visible
    );
}

#[test]
fn test_layer_reorder_command_revert() {
    let (mut project, l1_id, l2_id, level_id) = setup_test_project_with_layers();

    // Initial order: [l1, l2]
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );

    let mut undo = UndoStack::new();
    let cmd = ReorderLayerDefsCommand::new(vec![l2_id, l1_id]);
    undo.execute(Box::new(cmd), &mut project).unwrap();

    // Reordered: [l2, l1]
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l2_id, l1_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l2_id, l1_id]
    );

    // Undo reverts back to [l1, l2]
    undo.undo(&mut project).unwrap();
    assert_eq!(
        project.defs.layers.iter().map(|l| l.id).collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
    assert_eq!(
        project
            .level(level_id)
            .unwrap()
            .layers
            .iter()
            .map(|l| l.def_id)
            .collect::<Vec<_>>(),
        vec![l1_id, l2_id]
    );
}

#[test]
fn test_cascade_remove_and_revert() {
    let (mut project, l1_id, l2_id, level_id) = setup_test_project_with_layers();

    let mut undo = UndoStack::new();
    let cmd = RemoveLayerDefCommand::new(l1_id);
    undo.execute(Box::new(cmd), &mut project).unwrap();

    // Def removed and level layer removed
    assert_eq!(project.defs.layers.len(), 1);
    assert_eq!(project.defs.layers[0].id, l2_id);
    assert_eq!(project.level(level_id).unwrap().layers.len(), 1);
    assert_eq!(project.level(level_id).unwrap().layers[0].def_id, l2_id);

    // Revert restores l1_id at index 0
    undo.undo(&mut project).unwrap();
    assert_eq!(project.defs.layers.len(), 2);
    assert_eq!(project.defs.layers[0].id, l1_id);
    assert_eq!(project.level(level_id).unwrap().layers.len(), 2);
    assert_eq!(project.level(level_id).unwrap().layers[0].def_id, l1_id);
}

#[test]
fn test_update_layer_def_kind_and_grid_resize_reverts_cleanly() {
    let (mut project, l1_id, _, level_id) = setup_test_project_with_layers();
    let original_project = project.clone();

    // Paint cell (0,0) in IntGrid layer
    {
        let layer = project
            .level_mut(level_id)
            .unwrap()
            .layer_mut(l1_id)
            .unwrap();
        if let LayerData::IntGrid(ref mut grid) = layer.data {
            grid.set(0, 0, 1).unwrap();
        }
    }
    let painted_project = project.clone();

    let mut undo = UndoStack::new();

    // 1. Resize grid_size (16 -> 32) preserves overlapping IntGrid cell and reverts cleanly
    let mut resized = project.defs.layer(l1_id).unwrap().clone();
    resized.grid_size = UVec2::new(32, 32);
    undo.execute(
        Box::new(UpdateLayerDefCommand::with_stroke(resized.clone(), 100)),
        &mut project,
    )
    .unwrap();
    if let LayerData::IntGrid(ref grid) =
        project.level(level_id).unwrap().layer(l1_id).unwrap().data
    {
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 7);
        assert_eq!(grid.get(0, 0), Some(&1));
    } else {
        panic!("expected IntGrid data");
    }

    // 2. Convert kind (IntGrid -> Tiles) updates instance data and reverts cleanly
    let mut converted = project.defs.layer(l1_id).unwrap().clone();
    converted.kind = LayerKind::Tiles;
    undo.execute(
        Box::new(UpdateLayerDefCommand::with_stroke(converted, 200)),
        &mut project,
    )
    .unwrap();
    assert!(matches!(
        project.level(level_id).unwrap().layer(l1_id).unwrap().data,
        LayerData::Tiles(_)
    ));

    // Undo kind conversion restores resized IntGrid
    undo.undo(&mut project).unwrap();
    assert!(matches!(
        project.level(level_id).unwrap().layer(l1_id).unwrap().data,
        LayerData::IntGrid(_)
    ));

    // Undo resize restores byte-identical painted project
    undo.undo(&mut project).unwrap();
    assert_eq!(project, painted_project);
    assert_ne!(project, original_project);
}

#[test]
fn test_layer_helpers_usage_and_naming() {
    let (mut project, l1_id, _, level_id) = setup_test_project_with_layers();
    let empty_stats = count_layer_def_usage(&project, l1_id);
    assert_eq!(empty_stats.total_instances, 1);
    assert_eq!(empty_stats.non_empty_levels, 0);
    assert_eq!(empty_stats.placed_items, 0);

    {
        let layer = project
            .level_mut(level_id)
            .unwrap()
            .layer_mut(l1_id)
            .unwrap();
        if let LayerData::IntGrid(ref mut grid) = layer.data {
            grid.set(1, 2, 1).unwrap();
            grid.set(2, 2, 1).unwrap();
        }
    }
    let stats = count_layer_def_usage(&project, l1_id);
    assert_eq!(stats.total_instances, 1);
    assert_eq!(stats.non_empty_levels, 1);
    assert_eq!(stats.placed_items, 2);

    let names = vec!["Collisions".to_string(), "Collisions_copy".to_string()];
    assert_eq!(
        generate_unique_layer_name(&names, "Collisions"),
        "Collisions_copy_2"
    );

    let values = project.defs.layer(l1_id).unwrap().int_grid_values.clone();
    assert_eq!(next_int_grid_value_id(&values), 2);
    assert_eq!(next_int_grid_value_id(&[]), 1);
}

#[test]
fn test_auto_rule_group_and_rule_commands_apply_revert_merge() {
    use lc_core::auto_rules::{AutoLayerRuleDef, AutoLayerRuleGroup, AutoTileRect};
    use lc_core::id::{AutoRuleGroupId, AutoRuleId};
    use lc_feature_layers::{
        build_wizard_group, AddAutoRuleCommand, AddAutoRuleGroupCommand,
        ApplyAutoRuleWizardCommand, AutoRuleWizardPreset, AutoRuleWizardSpec,
        RemoveAutoRuleCommand, RemoveAutoRuleGroupCommand, ReorderAutoRuleGroupsCommand,
        ReorderAutoRulesCommand, UpdateAutoRuleCommand, UpdateAutoRuleGroupCommand,
    };

    let (mut project, l1_id, _, _) = setup_test_project_with_layers();
    let baseline = project.clone();

    let g1_id = AutoRuleGroupId(100);
    let g2_id = AutoRuleGroupId(101);
    let r1_id = AutoRuleId(200);
    let r2_id = AutoRuleId(201);

    let mut stack = UndoStack::new();
    stack
        .execute(
            Box::new(AddAutoRuleGroupCommand::new(
                l1_id,
                AutoLayerRuleGroup::new(g1_id, "Terrain"),
            )),
            &mut project,
        )
        .unwrap();
    stack
        .execute(
            Box::new(AddAutoRuleGroupCommand::new(
                l1_id,
                AutoLayerRuleGroup::new(g2_id, "Decor"),
            )),
            &mut project,
        )
        .unwrap();
    assert_eq!(project.defs.layer(l1_id).unwrap().auto_rule_groups.len(), 2);

    // Update group with stroke merge
    let mut g1_u1 = project
        .defs
        .layer(l1_id)
        .unwrap()
        .rule_group(g1_id)
        .unwrap()
        .clone();
    g1_u1.name = "Terrain_A".to_string();
    stack
        .execute(
            Box::new(UpdateAutoRuleGroupCommand::with_stroke(
                l1_id,
                g1_u1.clone(),
                77,
            )),
            &mut project,
        )
        .unwrap();
    g1_u1.name = "Terrain_B".to_string();
    stack
        .execute(
            Box::new(UpdateAutoRuleGroupCommand::with_stroke(l1_id, g1_u1, 77)),
            &mut project,
        )
        .unwrap();
    assert_eq!(
        project
            .defs
            .layer(l1_id)
            .unwrap()
            .rule_group(g1_id)
            .unwrap()
            .name,
        "Terrain_B"
    );

    // Reorder groups [g2, g1]
    stack
        .execute(
            Box::new(ReorderAutoRuleGroupsCommand::new(l1_id, vec![g2_id, g1_id])),
            &mut project,
        )
        .unwrap();
    assert_eq!(
        project.defs.layer(l1_id).unwrap().auto_rule_groups[0].id,
        g2_id
    );

    // Add rules, update with merge, reorder, remove
    let mut r1 = AutoLayerRuleDef::new(r1_id);
    r1.tile_rects.push(AutoTileRect::single(1));
    let mut r2 = AutoLayerRuleDef::new(r2_id);
    r2.tile_rects.push(AutoTileRect::single(2));
    stack
        .execute(
            Box::new(AddAutoRuleCommand::new(l1_id, g1_id, r1.clone())),
            &mut project,
        )
        .unwrap();
    stack
        .execute(
            Box::new(AddAutoRuleCommand::new(l1_id, g1_id, r2)),
            &mut project,
        )
        .unwrap();

    r1.chance = 0.5;
    stack
        .execute(
            Box::new(UpdateAutoRuleCommand::with_stroke(
                l1_id,
                g1_id,
                r1.clone(),
                88,
            )),
            &mut project,
        )
        .unwrap();
    r1.chance = 0.75;
    stack
        .execute(
            Box::new(UpdateAutoRuleCommand::with_stroke(l1_id, g1_id, r1, 88)),
            &mut project,
        )
        .unwrap();

    stack
        .execute(
            Box::new(ReorderAutoRulesCommand::new(
                l1_id,
                g1_id,
                vec![r2_id, r1_id],
            )),
            &mut project,
        )
        .unwrap();
    assert_eq!(
        project
            .defs
            .layer(l1_id)
            .unwrap()
            .rule_group(g1_id)
            .unwrap()
            .rules[0]
            .id,
        r2_id
    );

    stack
        .execute(
            Box::new(RemoveAutoRuleCommand::new(l1_id, g1_id, r2_id)),
            &mut project,
        )
        .unwrap();
    stack
        .execute(
            Box::new(RemoveAutoRuleGroupCommand::new(l1_id, g2_id)),
            &mut project,
        )
        .unwrap();

    // Wizard preset
    let wiz_group = build_wizard_group(
        AutoRuleGroupId(300),
        400,
        &AutoRuleWizardSpec {
            group_name: "WizardGroup".to_string(),
            int_grid_value: 1,
            preset: AutoRuleWizardPreset::SolidBorders3x3,
            sample_tiles: vec![10, 11, 12, 13],
            ..Default::default()
        },
    );
    assert_eq!(wiz_group.rules.len(), 4);
    stack
        .execute(
            Box::new(ApplyAutoRuleWizardCommand::new(l1_id, wiz_group)),
            &mut project,
        )
        .unwrap();

    // Undo everything back to exact baseline
    while stack.can_undo() {
        stack.undo(&mut project).unwrap();
    }
    assert_eq!(project, baseline);
}

#[test]
fn test_add_layer_respects_world_tags_and_remove_clears_auto_refs() {
    use lc_feature_layers::commands::AddLayerDefCommand;

    let mut project = Project::new("Multi-World Tags");
    let mut w_tundra = World::new(WorldId(1), "TundraWorld");
    w_tundra.included_layer_tags = vec!["tundra".to_string()];
    w_tundra
        .levels
        .push(Level::new(LevelId(1), "Tundra_0", UVec2::splat(64)));

    let mut w_desert = World::new(WorldId(2), "DesertWorld");
    w_desert.included_layer_tags = vec!["desert".to_string()];
    w_desert
        .levels
        .push(Level::new(LevelId(2), "Desert_0", UVec2::splat(64)));

    project.worlds.push(w_tundra);
    project.worlds.push(w_desert);

    let mut src_def = LayerDef::new(LayerDefId(1), "TundraCollision", LayerKind::IntGrid);
    src_def.tags = vec!["tundra".to_string()];
    let mut cmd_add_src = AddLayerDefCommand::new(src_def);
    cmd_add_src.apply(&mut project).unwrap();

    // TundraWorld received the instance; DesertWorld did not!
    assert_eq!(project.level(LevelId(1)).unwrap().layers.len(), 1);
    assert_eq!(project.level(LevelId(2)).unwrap().layers.len(), 0);

    let mut auto_def = LayerDef::new(LayerDefId(2), "SnowAuto", LayerKind::Auto);
    auto_def.tags = vec!["tundra".to_string()];
    auto_def.auto_source_layer_def_id = Some(LayerDefId(1));
    auto_def.auto_tiles_killed_by_other_layer_id = Some(LayerDefId(1));
    AddLayerDefCommand::new(auto_def)
        .apply(&mut project)
        .unwrap();

    let before_remove = project.clone();
    let mut cmd_rm = RemoveLayerDefCommand::new(LayerDefId(1));
    cmd_rm.apply(&mut project).unwrap();

    let snow = project.defs.layer(LayerDefId(2)).unwrap();
    assert_eq!(snow.auto_source_layer_def_id, None);
    assert_eq!(snow.auto_tiles_killed_by_other_layer_id, None);

    cmd_rm.revert(&mut project);
    assert_eq!(project, before_remove);
}

#[test]
fn test_assistant_slots_bake_auto_layer_and_move_entities_commands() {
    use lc_core::entity::EntityDef;
    use lc_core::id::{AutoRuleGroupId, EntityDefId, EntityId, TilesetDefId};
    use lc_core::layer::EntityInstance;
    use lc_feature_layers::{
        build_wizard_group, ApplyAutoRuleWizardCommand, AutoRuleWizardPreset, AutoRuleWizardSpec,
        BakeAutoLayerCommand, MoveEntitiesToLayerCommand,
    };
    use std::collections::BTreeMap;

    let mut project = Project::new("Layer Ops Test");
    project
        .defs
        .entities
        .push(EntityDef::new(EntityDefId(1), "Coin"));

    let mut world = World::new(WorldId(1), "Main");
    let mut level = Level::new(LevelId(1), "L0", UVec2::new(64, 64));

    let mut src_def = LayerDef::new(LayerDefId(1), "Collisions", LayerKind::IntGrid);
    src_def.grid_size = UVec2::new(16, 16);
    src_def.int_grid_values.push(IntGridValueDef {
        value: 1,
        name: "walls".to_string(),
        color: [200, 140, 80, 255],
        ..Default::default()
    });

    let mut auto_def = LayerDef::new(LayerDefId(2), "WallsAuto", LayerKind::Auto);
    auto_def.grid_size = UVec2::new(16, 16);
    auto_def.tileset_id = Some(TilesetDefId(1));
    auto_def.auto_source_layer_def_id = Some(LayerDefId(1));

    let mut ent_a = LayerDef::new(LayerDefId(3), "EntitiesA", LayerKind::Entities);
    ent_a.grid_size = UVec2::new(16, 16);
    let mut ent_b = LayerDef::new(LayerDefId(4), "EntitiesB", LayerKind::Entities);
    ent_b.grid_size = UVec2::new(16, 16);

    project
        .defs
        .layers
        .extend([src_def, auto_def, ent_a, ent_b]);

    let mut ig = Grid::new(4, 4, 0);
    ig.set(1, 1, 1).unwrap();
    level
        .layers
        .push(LayerInstance::new(LayerDefId(1), LayerData::IntGrid(ig)));
    level
        .layers
        .push(LayerInstance::new(LayerDefId(2), LayerData::Auto));
    level.layers.push(LayerInstance::new(
        LayerDefId(3),
        LayerData::Entities(vec![EntityInstance {
            id: EntityId(10),
            def_id: EntityDefId(1),
            ..Default::default()
        }]),
    ));
    level.layers.push(LayerInstance::new(
        LayerDefId(4),
        LayerData::Entities(Vec::new()),
    ));
    world.levels.push(level);
    project.worlds.push(world);

    // 1. AssistantSlots wizard generates rules with outer value + break_on_match when !allow_stack
    let mut slots = BTreeMap::new();
    slots.insert(5usize, 42u32); // Center Fill -> tile 42
    slots.insert(15usize, 99u32); // Single Isolated -> tile 99
    let grp = build_wizard_group(
        AutoRuleGroupId(10),
        100,
        &AutoRuleWizardSpec {
            group_name: "Walls".to_string(),
            int_grid_value: 1,
            outer_int_grid_value: None,
            preset: AutoRuleWizardPreset::AssistantSlots,
            sample_tiles: Vec::new(),
            slot_tiles: slots.clone(),
            allow_stack: false,
        },
    );
    assert_eq!(grp.rules.len(), 2);
    assert!(grp.rules[0].break_on_match);
    let grp_stacked = build_wizard_group(
        AutoRuleGroupId(11),
        200,
        &AutoRuleWizardSpec {
            group_name: "Stacked".to_string(),
            int_grid_value: 1,
            outer_int_grid_value: None,
            preset: AutoRuleWizardPreset::AssistantSlots,
            sample_tiles: Vec::new(),
            slot_tiles: slots,
            allow_stack: true,
        },
    );
    assert!(!grp_stacked.rules[0].break_on_match);
    ApplyAutoRuleWizardCommand::new(LayerDefId(2), grp)
        .apply(&mut project)
        .unwrap();

    // 2. BakeAutoLayerCommand converts Auto layer to Tiles with evaluated tile 99 at (1,1)
    let before_bake = project.clone();
    let mut cmd_bake = BakeAutoLayerCommand::new(LayerDefId(2), 1);
    cmd_bake.apply(&mut project).unwrap();
    assert_eq!(
        project.defs.layer(LayerDefId(2)).unwrap().kind,
        LayerKind::Tiles
    );
    match &project
        .level(LevelId(1))
        .unwrap()
        .layer(LayerDefId(2))
        .unwrap()
        .data
    {
        LayerData::Tiles(grid) => {
            assert_eq!(
                grid.get(1, 1).and_then(|c| c.as_ref()).map(|t| t.tile_id),
                Some(99)
            );
        }
        other => panic!("expected baked Tiles layer, got {other:?}"),
    }
    cmd_bake.revert(&mut project);
    assert_eq!(project, before_bake);

    // 3. MoveEntitiesToLayerCommand moves entities and reverts cleanly
    let before_move = project.clone();
    let mut cmd_move = MoveEntitiesToLayerCommand::new(LayerDefId(3), LayerDefId(4));
    cmd_move.apply(&mut project).unwrap();
    match &project
        .level(LevelId(1))
        .unwrap()
        .layer(LayerDefId(3))
        .unwrap()
        .data
    {
        LayerData::Entities(v) => assert!(v.is_empty()),
        _ => panic!("expected empty EntitiesA"),
    }
    match &project
        .level(LevelId(1))
        .unwrap()
        .layer(LayerDefId(4))
        .unwrap()
        .data
    {
        LayerData::Entities(v) => assert_eq!(v.len(), 1),
        _ => panic!("expected 1 entity in EntitiesB"),
    }
    cmd_move.revert(&mut project);
    assert_eq!(project, before_move);
}
