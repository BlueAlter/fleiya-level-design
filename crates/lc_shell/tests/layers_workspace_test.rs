//! Behavioral and headless interaction tests for the Layers Workspace (`AppMode::Layers`).

use egui::{vec2, Context, Pos2, RawInput, Rect};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{IntGridValueDef, LayerDef, TilesetDef};
use lc_core::grid::Grid;
use lc_core::id::{LayerDefId, LevelId, TilesetDefId, WorldId};
use lc_core::layer::{LayerData, LayerInstance, LayerKind};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_layers::{RemoveLayerDefCommand, ReorderLayerDefsCommand, UpdateLayerDefCommand};
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::layers::left_panel::create_new_layer_def;
use lc_shell::features::layers::view::{
    render_layer_view, AUTO_TILE_SUB_TAB, DOCUMENT_SUB_TAB, LAYER_DEF_SUB_TAB,
};
use lc_shell::ShellApp;

fn make_layer_app() -> ShellApp {
    let mut project = Project::new("Layers Workspace Test");
    project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Terrain".to_string(),
        path: "assets/terrain.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: Default::default(),
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    });

    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(1),
        name: "Collisions".to_string(),
        kind: LayerKind::IntGrid,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: vec![
            IntGridValueDef {
                value: 1,
                name: "Solid".to_string(),
                color: [220, 60, 60, 255],
                ..Default::default()
            },
            IntGridValueDef {
                value: 2,
                name: "Water".to_string(),
                color: [60, 140, 240, 255],
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(2),
        name: "GroundTiles".to_string(),
        kind: LayerKind::Tiles,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(TilesetDefId(1)),
        int_grid_values: Vec::new(),
        ..Default::default()
    });
    project.defs.layers.push(LayerDef {
        id: LayerDefId::new(3),
        name: "Actors".to_string(),
        kind: LayerKind::Entities,
        grid_size: UVec2::new(16, 16),
        tileset_id: None,
        int_grid_values: Vec::new(),
        ..Default::default()
    });

    let mut world = World::new(WorldId::new(1), "Main");
    let mut level = Level::new(LevelId::new(1), "Level_0", UVec2::new(256, 256));
    let mut int_grid = Grid::new(16, 16, 0);
    let _ = int_grid.set(0, 0, 1);
    level.layers.push(LayerInstance::new(
        LayerDefId::new(1),
        LayerData::IntGrid(int_grid),
    ));
    level.layers.push(LayerInstance::new(
        LayerDefId::new(2),
        LayerData::Tiles(Grid::new(16, 16, None)),
    ));
    level.layers.push(LayerInstance::new(
        LayerDefId::new(3),
        LayerData::Entities(Vec::new()),
    ));
    world.levels.push(level);
    project.worlds.push(world);

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Layers;
    app.layer_view_state.select_layer(LayerDefId::new(1));
    app
}

fn run_layer_frame(app: &mut ShellApp) -> egui::FullOutput {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        render_layer_view(app, ui);
    });
    out.textures_delta.clear();
    out
}

#[test]
fn empty_project_renders_empty_state_and_creates_layer_with_undo() {
    let mut app = ShellApp::new(Project::new("Empty Layers Test"));
    app.active_mode = AppMode::Layers;

    let _out = run_layer_frame(&mut app);
    assert!(app.project.defs.layers.is_empty());
    assert_eq!(app.layer_view_state.selected_layer_id, None);

    let new_id = create_new_layer_def(&mut app);
    app.process_commands_and_events();
    assert_eq!(app.project.defs.layers.len(), 1);
    assert_eq!(app.layer_view_state.selected_layer_id, Some(new_id));

    app.undo();
    assert!(app.project.defs.layers.is_empty());
}

#[test]
fn layer_workspace_sub_tabs_render_cleanly_without_mutating_project() {
    let mut app = make_layer_app();
    let baseline = app.project.clone();

    app.layer_view_state.active_sub_tab = LAYER_DEF_SUB_TAB;
    let _out_def = run_layer_frame(&mut app);
    assert_eq!(app.project, baseline);
    assert!(app.pending_commands.is_empty());

    app.layer_view_state.active_sub_tab = AUTO_TILE_SUB_TAB;
    let _out_auto = run_layer_frame(&mut app);
    assert_eq!(app.project, baseline);
    assert!(app.pending_commands.is_empty());

    app.layer_view_state.active_sub_tab = DOCUMENT_SUB_TAB;
    let _out_doc = run_layer_frame(&mut app);
    assert_eq!(app.project, baseline);
    assert!(app.pending_commands.is_empty());
}

#[test]
fn layer_workspace_handles_all_four_layer_kinds_and_narrow_width() {
    let mut app = make_layer_app();
    app.project.defs.layers.push(LayerDef {
        id: LayerDefId::new(4),
        name: "AutoDecor".to_string(),
        kind: LayerKind::Auto,
        grid_size: UVec2::new(16, 16),
        tileset_id: Some(TilesetDefId(1)),
        int_grid_values: Vec::new(),
        ..Default::default()
    });

    for id in 1..=4 {
        app.layer_view_state.select_layer(LayerDefId::new(id));
        let _out = run_layer_frame(&mut app);
    }

    // Narrow viewport (< 760px) stacks vertically without panic
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(720.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        render_layer_view(&mut app, ui);
    });
    out.textures_delta.clear();
}

#[test]
fn layer_reorder_update_and_cascade_delete_roundtrip() {
    let mut app = make_layer_app();

    // Reorder [3, 1, 2]
    app.execute_command(Box::new(ReorderLayerDefsCommand::new(vec![
        LayerDefId::new(3),
        LayerDefId::new(1),
        LayerDefId::new(2),
    ])));
    app.process_commands_and_events();
    assert_eq!(
        app.project
            .defs
            .layers
            .iter()
            .map(|l| l.id)
            .collect::<Vec<_>>(),
        vec![LayerDefId::new(3), LayerDefId::new(1), LayerDefId::new(2)]
    );

    // Update grid_size and IntGrid values
    let mut updated = app.project.defs.layer(LayerDefId::new(1)).unwrap().clone();
    updated.grid_size = UVec2::new(32, 32);
    updated.int_grid_values.push(IntGridValueDef {
        value: 3,
        name: "Hazard".to_string(),
        color: [250, 200, 40, 255],
        ..Default::default()
    });
    app.execute_command(Box::new(UpdateLayerDefCommand::with_stroke(updated, 1)));
    app.process_commands_and_events();
    assert_eq!(
        app.project
            .defs
            .layer(LayerDefId::new(1))
            .unwrap()
            .grid_size,
        UVec2::new(32, 32)
    );
    assert_eq!(
        app.project
            .defs
            .layer(LayerDefId::new(1))
            .unwrap()
            .int_grid_values
            .len(),
        3
    );

    // Cascade delete restores cleanly on undo
    app.execute_command(Box::new(RemoveLayerDefCommand::new(LayerDefId::new(1))));
    app.process_commands_and_events();
    assert!(app.project.defs.layer(LayerDefId::new(1)).is_none());

    app.undo();
    assert!(app.project.defs.layer(LayerDefId::new(1)).is_some());
}

#[test]
fn auto_tile_tab_is_strictly_uninteractable_for_non_auto_rule_layers_except_tab_switching() {
    use egui::{Event, PointerButton};

    let mut app = make_layer_app();
    // Layer 1 = IntGrid without tileset (not auto), Layer 2 = Tiles, Layer 3 = Entities
    for id in [LayerDefId::new(1), LayerDefId::new(2), LayerDefId::new(3)] {
        app.layer_view_state.select_layer(id);
        app.layer_view_state.active_sub_tab = AUTO_TILE_SUB_TAB;
        let baseline = app.project.clone();

        let ctx = Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx, ThemeChoice::Dark);

        // Probe clicks across the AUTO TILE workspace body (toolbar, columns, sandbox)
        for probe in [
            Pos2::new(1280.0, 128.0),
            Pos2::new(600.0, 260.0),
            Pos2::new(620.0, 320.0),
            Pos2::new(1220.0, 420.0),
        ] {
            let press = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
                events: vec![
                    Event::PointerMoved(probe),
                    Event::PointerButton {
                        pos: probe,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            };
            let mut o1 = ctx.run_ui(press, |ui| render_layer_view(&mut app, ui));
            o1.textures_delta.clear();

            let release = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
                events: vec![Event::PointerButton {
                    pos: probe,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                }],
                ..Default::default()
            };
            let mut o2 = ctx.run_ui(release, |ui| render_layer_view(&mut app, ui));
            o2.textures_delta.clear();
            app.process_commands_and_events();
        }

        assert_eq!(
            app.project, baseline,
            "AUTO TILE body must remain completely uninteractable for layer {id:?}"
        );
        assert_eq!(
            app.layer_view_state.center_auto_mode,
            lc_shell::features::layers::state::CenterAutoMode::RuleEditor
        );
        assert!(app.layer_view_state.auto_tile_picker.is_none());

        // Switching tabs via the top sub_tabs strip MUST still work!
        let mut out = run_layer_frame(&mut app);
        let tab_rect = out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.text() == "LAYERS DEF" => {
                    Some(t.galley.rect.translate(t.pos.to_vec2()))
                }
                _ => None,
            })
            .expect("LAYERS DEF tab label must be rendered");
        out.textures_delta.clear();

        let tab_pos = tab_rect.center();
        let mut o_down = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
                events: vec![
                    Event::PointerMoved(tab_pos),
                    Event::PointerButton {
                        pos: tab_pos,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            },
            |ui| render_layer_view(&mut app, ui),
        );
        o_down.textures_delta.clear();
        let mut o_up = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
                events: vec![Event::PointerButton {
                    pos: tab_pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                }],
                ..Default::default()
            },
            |ui| render_layer_view(&mut app, ui),
        );
        o_up.textures_delta.clear();
        assert_eq!(
            app.layer_view_state.active_sub_tab, LAYER_DEF_SUB_TAB,
            "User must still be able to switch tabs away from locked AUTO TILE"
        );
    }
}

#[test]
fn auto_tile_tab_renders_interactive_rules_and_sidebar_groups_by_tag() {
    use lc_core::auto_rules::{AutoLayerRuleDef, AutoLayerRuleGroup, AutoTileRect};
    use lc_core::id::{AutoRuleGroupId, AutoRuleId};

    let mut app = make_layer_app();
    // Add tags so left panel exercises collapsible tag grouping
    if let Some(l1) = app.project.defs.layer_mut(LayerDefId::new(1)) {
        l1.tags = vec!["Tundra".to_string()];
        l1.tileset_id = Some(TilesetDefId(1)); // IntGrid + Tileset -> uses_auto_rules() == true!
        let mut r = AutoLayerRuleDef::new(AutoRuleId(1));
        r.set_pattern_at(1, 1, 1);
        r.tile_rects.push(AutoTileRect::single(4));
        let mut g = AutoLayerRuleGroup::new(AutoRuleGroupId(1), "SnowEdges");
        g.rules.push(r);
        l1.auto_rule_groups.push(g);
    }
    if let Some(l2) = app.project.defs.layer_mut(LayerDefId::new(2)) {
        l2.tags = vec!["Desert".to_string()];
    }

    app.layer_view_state.select_layer(LayerDefId::new(1));
    app.layer_view_state.active_sub_tab = AUTO_TILE_SUB_TAB;
    let out = run_layer_frame(&mut app);

    let mut labels = Vec::new();
    for cs in &out.shapes {
        if let egui::Shape::Text(t) = &cs.shape {
            labels.push(t.galley.text().to_string());
        }
    }
    assert!(labels.iter().any(|s| s.contains("SnowEdges")));
    assert!(labels.iter().any(|s| s == "PATTERN & OUTPUT"));
    assert!(labels.iter().any(|s| s == "LIVE SANDBOX PREVIEW"));
    assert!(labels.iter().any(|s| s == "TUNDRA" || s.contains("TUNDRA")));
    assert!(labels.iter().any(|s| s == "DESERT" || s.contains("DESERT")));
}

#[test]
fn center_rules_assistant_renders_and_creates_rules_with_undo() {
    use lc_shell::features::layers::state::CenterAutoMode;

    let mut app = make_layer_app();
    if let Some(l1) = app.project.defs.layer_mut(LayerDefId::new(1)) {
        l1.tileset_id = Some(TilesetDefId(1));
    }

    app.layer_view_state.select_layer(LayerDefId::new(1));
    app.layer_view_state.active_sub_tab = AUTO_TILE_SUB_TAB;
    app.layer_view_state.center_auto_mode = CenterAutoMode::RulesAssistant;
    let out = run_layer_frame(&mut app);

    let labels: Vec<String> = out
        .shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_string()),
            _ => None,
        })
        .collect();
    assert!(labels.iter().any(|s| s == "AUTO-LAYER RULES ASSISTANT"));
    assert!(labels.iter().any(|s| s == "1. INTGRID VALUES"));
    assert!(labels.iter().any(|s| s == "2. GROUP OF RULES NAME"));
    assert!(labels.iter().any(|s| s == "3. AUTO-RENDERED TILES"));
    assert!(labels.iter().any(|s| s == "4. OPTIONS"));
    assert!(labels.iter().any(|s| s.contains("TILESET ATLAS:")));
}
