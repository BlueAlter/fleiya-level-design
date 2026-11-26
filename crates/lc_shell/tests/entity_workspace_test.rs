//! Behavioral and headless interaction tests for the Entity Workspace (`AppMode::Entities`).

use egui::{vec2, Context, Pos2, RawInput, Rect};
use glam::{UVec2, Vec2};
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{
    EntityDef, EntityLimitBehavior, EntityLimitScope, EntityRenderMode, TileRenderMode, TilesetDef,
};
use lc_core::fields::{
    AllowedRefs, EditorDisplayMode, EditorDisplayPos, EditorLinkStyle, FieldDef, FieldType,
    FieldValue, TextLanguageMode,
};
use lc_core::id::{EntityDefId, EntityId, EnumDefId, LayerDefId, LevelId, TilesetDefId, WorldId};
use lc_core::layer::{EntityInstance, LayerData, LayerInstance};
use lc_core::level::Level;
use lc_core::project::Project;
use lc_core::world::World;
use lc_feature_entities::{
    AddEntityFieldCommand, RemoveEntityDefCommand, ReorderEntityDefCommand,
    ReorderEntityFieldCommand, UpdateEntityDefCommand,
};
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::entities::left_panel::create_new_entity_def;
use lc_shell::features::entities::view::{
    render_entity_view, DOCUMENT_SUB_TAB, ENTITY_DEF_SUB_TAB,
};
use lc_shell::ShellApp;

fn make_entity_app() -> ShellApp {
    let mut project = Project::new("Entity Workspace Test");
    project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Sprites".to_string(),
        path: "assets/sprites.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: Default::default(),
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    });

    let mut player = EntityDef::new(EntityDefId::new(1), "Player");
    player.tags = vec!["actor".to_string(), "player".to_string()];
    player.size_px = UVec2::new(16, 24);
    player.pivot = Vec2::new(0.5, 1.0);
    player.render_mode = EntityRenderMode::Rectangle;
    player.tile_render_mode = TileRenderMode::NineSlice;
    player.nine_slice_borders = [4, 4, 4, 4];
    player.max_count = 1;
    player.limit_scope = EntityLimitScope::PerWorld;
    player.limit_behavior = EntityLimitBehavior::MoveLastOne;
    player.fields = vec![
        FieldDef {
            name: "hp".to_string(),
            field_type: FieldType::Int,
            default_value: Some(FieldValue::Int(100)),
            min: Some(0.0),
            max: Some(999.0),
            editor_display_mode: EditorDisplayMode::ValueOnly,
            editor_display_pos: EditorDisplayPos::Above,
            ..Default::default()
        },
        FieldDef {
            name: "target".to_string(),
            field_type: FieldType::EntityRef,
            can_be_null: true,
            allowed_refs: AllowedRefs::OnlyTags,
            allowed_ref_tags: vec!["actor".to_string()],
            editor_display_mode: EditorDisplayMode::RefLinkBetweenPivots,
            editor_link_style: EditorLinkStyle::CurvedArrow,
            ..Default::default()
        },
        FieldDef {
            name: "bio".to_string(),
            field_type: FieldType::Multilines,
            text_language_mode: Some(TextLanguageMode::Markdown),
            ..Default::default()
        },
    ];
    project.defs.entities.push(player);

    let enemy = EntityDef::new(EntityDefId::new(2), "Goblin");
    project.defs.entities.push(enemy);

    let mut world = World::new(WorldId::new(1), "Main");
    let mut level = Level::new(LevelId::new(1), "Level_0", UVec2::new(256, 256));
    let layer = LayerInstance::new(
        LayerDefId::new(1),
        LayerData::Entities(vec![EntityInstance {
            id: EntityId::new(1),
            def_id: EntityDefId::new(1),
            pos_px: glam::IVec2::new(32, 48),
            size_px: UVec2::new(16, 24),
            fields: Default::default(),
        }]),
    );
    level.layers.push(layer);
    world.levels.push(level);
    project.worlds.push(world);

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Entities;
    app.entity_view_state.select_entity(EntityDefId::new(1));
    app.entity_view_state.selected_field_idx = Some(0);
    app
}

fn run_entity_frame(app: &mut ShellApp) -> egui::FullOutput {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw_input, |ui| {
        render_entity_view(app, ui);
    });
    out.textures_delta.clear();
    out
}

#[test]
fn create_new_entity_def_populates_ldtk_defaults_and_undoes() {
    let mut app = ShellApp::new(Project::new("New Entity Test"));
    app.active_mode = AppMode::Entities;

    let id = create_new_entity_def(&mut app);
    app.process_commands_and_events();
    assert_eq!(app.project.defs.entities.len(), 1);
    assert_eq!(app.entity_view_state.selected_entity_id, Some(id));
    let def = app.project.defs.entity(id).expect("created entity");
    assert_eq!(def.size_px, UVec2::new(16, 16));
    assert!(!def.resizable_x && !def.resizable_y);

    app.undo();
    assert!(app.project.defs.entities.is_empty());
}

#[test]
fn add_all_eleven_ldtk_field_types_and_reorder() {
    let mut app = make_entity_app();
    let entity_id = EntityDefId::new(2);

    let all_types = [
        ("f_int", FieldType::Int),
        ("f_float", FieldType::Float),
        ("f_bool", FieldType::Bool),
        ("f_str", FieldType::String),
        ("f_multi", FieldType::Multilines),
        ("f_color", FieldType::Color),
        ("f_enum", FieldType::Enum(EnumDefId::new(1))),
        ("f_point", FieldType::Point),
        ("f_ref", FieldType::EntityRef),
        ("f_path", FieldType::FilePath),
        ("f_tile", FieldType::Tile),
    ];

    for (name, ft) in all_types {
        app.execute_command(Box::new(AddEntityFieldCommand::new(
            entity_id,
            FieldDef {
                name: name.to_string(),
                field_type: ft,
                ..Default::default()
            },
        )));
    }
    app.process_commands_and_events();
    let def = app.project.defs.entity(entity_id).expect("enemy def");
    assert_eq!(def.fields.len(), 11);

    app.execute_command(Box::new(ReorderEntityFieldCommand::new(entity_id, 0, 10)));
    app.process_commands_and_events();
    assert_eq!(
        app.project.defs.entity(entity_id).unwrap().fields[10].name,
        "f_int"
    );

    app.undo();
    assert_eq!(
        app.project.defs.entity(entity_id).unwrap().fields[0].name,
        "f_int"
    );
}

#[test]
fn update_and_reorder_and_cascade_delete_commands_work_in_shell() {
    let mut app = make_entity_app();

    let mut updated = app
        .project
        .defs
        .entity(EntityDefId::new(1))
        .unwrap()
        .clone();
    updated.pivot = Vec2::ZERO;
    updated.hollow = true;
    updated.allow_out_of_bounds = true;
    app.execute_command(Box::new(UpdateEntityDefCommand::new(updated)));
    app.process_commands_and_events();
    assert!(app.project.defs.entity(EntityDefId::new(1)).unwrap().hollow);

    app.execute_command(Box::new(ReorderEntityDefCommand::new(
        EntityDefId::new(1),
        1,
    )));
    app.process_commands_and_events();
    assert_eq!(app.project.defs.entities[0].id, EntityDefId::new(2));
    app.undo();
    assert_eq!(app.project.defs.entities[0].id, EntityDefId::new(1));

    // Player has 1 instance on Level_0, so cascade remove clears it cleanly and undo restores it.
    app.execute_command(Box::new(RemoveEntityDefCommand::cascade(EntityDefId::new(
        1,
    ))));
    app.process_commands_and_events();
    assert_eq!(app.project.defs.entities.len(), 1);
    assert_eq!(app.project.worlds[0].levels[0].iter_entities().count(), 0);
    app.undo();
    assert_eq!(app.project.defs.entities.len(), 2);
    assert_eq!(app.project.worlds[0].levels[0].iter_entities().count(), 1);
}

#[test]
fn entity_workspace_renders_empty_and_document_subtab_headlessly() {
    let mut empty = ShellApp::new(Project::new("Empty Entities"));
    empty.active_mode = AppMode::Entities;
    let out_empty = run_entity_frame(&mut empty);
    assert!(!out_empty.shapes.is_empty());

    let mut app = make_entity_app();
    app.entity_view_state.active_sub_tab = ENTITY_DEF_SUB_TAB;
    let out_def = run_entity_frame(&mut app);
    assert!(!out_def.shapes.is_empty());

    app.entity_view_state.active_sub_tab = DOCUMENT_SUB_TAB;
    let out_doc = run_entity_frame(&mut app);
    assert!(!out_doc.shapes.is_empty());
}

#[test]
fn bool_field_hides_non_applicable_settings_and_left_panel_renders_tag_groups() {
    let mut app = make_entity_app();
    // Add Bool field and select it; LDtk rules hide Can be null / Limits / Searchable for Bool.
    app.execute_command(Box::new(AddEntityFieldCommand::new(
        EntityDefId::new(1),
        FieldDef {
            name: "is_boss".to_string(),
            field_type: FieldType::Array(Box::new(FieldType::Bool)),
            ..Default::default()
        },
    )));
    app.process_commands_and_events();
    let fields_len = app
        .project
        .defs
        .entity(EntityDefId::new(1))
        .unwrap()
        .fields
        .len();
    app.entity_view_state.selected_field_idx = Some(fields_len - 1);
    app.entity_view_state.collapse_settings = true;

    let out = run_entity_frame(&mut app);
    let mut texts = Vec::new();
    fn collect_texts(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) => out.push(t.galley.text().to_string()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| collect_texts(s, out)),
            _ => {}
        }
    }
    for clipped in &out.shapes {
        collect_texts(&clipped.shape, &mut texts);
    }
    // Tag groups: Player has tags ["actor", "player"], Goblin has no tags -> UNTAGGED, ACTOR, PLAYER headers!
    assert!(texts.iter().any(|t| t == "UNTAGGED (1)"));
    assert!(texts.iter().any(|t| t == "ACTOR (1)"));
    assert!(texts.iter().any(|t| t == "PLAYER (1)"));
    // Array<Bool> shows Default state + Array length, NOT Can be null / Searchable / Limits
    assert!(texts.iter().any(|t| t == "Default state"));
    assert!(texts.iter().any(|t| t == "Array length"));
    assert!(!texts.iter().any(|t| t == "Can be null"));
    assert!(!texts.iter().any(|t| t == "Searchable"));
    assert!(!texts.iter().any(|t| t == "Limits"));
}

#[test]
fn clicking_single_value_opens_11_type_grid_popup_and_adds_field() {
    let mut app = make_entity_app();
    app.entity_view_state.collapse_settings = true;
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let full = Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0));

    // Frame 0: find "+ SINGLE VALUE" button position
    let mut out0 = ctx.run_ui(
        RawInput {
            screen_rect: Some(full),
            ..Default::default()
        },
        |ui| {
            render_entity_view(&mut app, ui);
        },
    );
    out0.textures_delta.clear();

    fn find_text_rect(shape: &egui::Shape, needle: &str) -> Option<Rect> {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == needle => {
                Some(Rect::from_min_size(t.pos, t.galley.size()))
            }
            egui::Shape::Vec(v) => v.iter().find_map(|s| find_text_rect(s, needle)),
            _ => None,
        }
    }
    let btn_rect = out0
        .shapes
        .iter()
        .find_map(|s| find_text_rect(&s.shape, "+ SINGLE VALUE"))
        .expect("+ SINGLE VALUE text must render");
    let click_pos = btn_rect.center();

    // Click "+ SINGLE VALUE" (press + release)
    for pressed in [true, false] {
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(full),
                events: vec![
                    egui::Event::PointerMoved(click_pos),
                    egui::Event::PointerButton {
                        pos: click_pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                render_entity_view(&mut app, ui);
            },
        );
        out.textures_delta.clear();
    }

    // Next frame renders the open popup containing all 11 type cards
    let mut out_open = ctx.run_ui(
        RawInput {
            screen_rect: Some(full),
            ..Default::default()
        },
        |ui| {
            render_entity_view(&mut app, ui);
        },
    );
    out_open.textures_delta.clear();

    let mut int_card_pos = None;
    let mut saw_boolean = false;
    let mut saw_point = false;
    for s in &out_open.shapes {
        if let Some(r) = find_text_rect(&s.shape, "Integer") {
            int_card_pos = Some(r.center());
        }
        if find_text_rect(&s.shape, "Boolean").is_some() {
            saw_boolean = true;
        }
        if find_text_rect(&s.shape, "Point").is_some() {
            saw_point = true;
        }
    }
    let int_pos = int_card_pos.expect("clicking + SINGLE VALUE must open the 11-type grid popup");
    assert!(saw_boolean && saw_point);

    // Click "Integer" card in the popup -> adds new Int field
    let prev_count = app
        .project
        .defs
        .entity(EntityDefId::new(1))
        .unwrap()
        .fields
        .len();
    for pressed in [true, false] {
        let mut out = ctx.run_ui(
            RawInput {
                screen_rect: Some(full),
                events: vec![
                    egui::Event::PointerMoved(int_pos),
                    egui::Event::PointerButton {
                        pos: int_pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                render_entity_view(&mut app, ui);
            },
        );
        out.textures_delta.clear();
    }
    app.process_commands_and_events();
    let new_count = app
        .project
        .defs
        .entity(EntityDefId::new(1))
        .unwrap()
        .fields
        .len();
    assert_eq!(new_count, prev_count + 1);
}
