//! Tests for Tileset Workspace Step B5: Tab-switch glitch between DEF and DOCUMENT tabs.

use egui::{vec2, Context, Rect, Shape};
use glam::UVec2;
use lc_canvas::theme::{self, ThemeChoice};
use lc_core::defs::{TileMarking, TilesetDef};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::tileset::state::TilesetCard;
use lc_shell::features::tileset::view::{
    render_tileset_view, ANIMATION_SUB_TAB, COLLISION_SUB_TAB, DOCUMENT_SUB_TAB, MARKING_SUB_TAB,
    TILESET_DEF_SUB_TAB,
};
use lc_shell::ShellApp;

fn collect_all_shapes(shape: &Shape, rects: &mut Vec<Rect>, texts: &mut Vec<(String, Rect)>) {
    match shape {
        Shape::Rect(r) => rects.push(r.rect),
        Shape::Text(t) => {
            texts.push((
                t.galley.text().to_string(),
                Rect::from_min_size(t.pos, t.galley.size()),
            ));
        }
        Shape::Vec(v) => {
            for s in v {
                collect_all_shapes(s, rects, texts);
            }
        }
        _ => {}
    }
}

fn render_frame(
    ctx: &Context,
    app: &mut ShellApp,
    full_rect: Rect,
) -> (Vec<Rect>, Vec<(String, Rect)>) {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            ..Default::default()
        },
        |ui| {
            render_tileset_view(app, ui);
        },
    );
    output.textures_delta.clear();
    let mut rects = Vec::new();
    let mut texts = Vec::new();
    for shape in &output.shapes {
        collect_all_shapes(&shape.shape, &mut rects, &mut texts);
    }
    (rects, texts)
}

fn assert_frames_match(
    frame1_rects: &[Rect],
    frame1_texts: &[(String, Rect)],
    frame4_rects: &[Rect],
    frame4_texts: &[(String, Rect)],
    tab_name: &str,
) {
    assert_eq!(
        frame1_rects.len(),
        frame4_rects.len(),
        "{} frame 1 rects count ({}) must match frame 4 ({})",
        tab_name,
        frame1_rects.len(),
        frame4_rects.len()
    );
    for (i, (r1, r4)) in frame1_rects.iter().zip(frame4_rects.iter()).enumerate() {
        let diff_min = r1.min.distance(r4.min);
        let diff_max = r1.max.distance(r4.max);
        assert!(
            diff_min <= 0.5 && diff_max <= 0.5,
            "{} rect [{}] mismatch: frame1={:?}, frame4={:?}, diff_min={}, diff_max={}",
            tab_name,
            i,
            r1,
            r4,
            diff_min,
            diff_max
        );
    }

    assert_eq!(
        frame1_texts.len(),
        frame4_texts.len(),
        "{} frame 1 text count ({}) must match frame 4 ({})",
        tab_name,
        frame1_texts.len(),
        frame4_texts.len()
    );
    for (i, ((t1, p1), (t4, p4))) in frame1_texts.iter().zip(frame4_texts.iter()).enumerate() {
        assert_eq!(t1, t4, "{} text [{}] content mismatch", tab_name, i);
        let diff_min = p1.min.distance(p4.min);
        let diff_max = p1.max.distance(p4.max);
        assert!(
            diff_min <= 0.5 && diff_max <= 0.5,
            "{} text [{}] '{}' rect mismatch: frame1={:?}, frame4={:?}",
            tab_name,
            i,
            t1,
            p1,
            p4
        );
    }
}

#[test]
fn test_b5_tab_switch_glitch_and_stability() {
    let mut project = Project::new("B5 Tab Switch Test");
    let ts_id = TilesetDefId(1);
    project.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "TestTileset".to_string(),
        path: String::new(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    });

    let mut app = ShellApp::new(project);
    app.active_mode = AppMode::Tilesets;
    app.tileset_view_state.select_tileset(ts_id);
    app.tileset_view_state.active_sub_tab = TILESET_DEF_SUB_TAB;

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));

    // 1. Render DEF for 4 frames
    for _ in 0..4 {
        render_frame(&ctx, &mut app, full_rect);
    }

    // 2. Switch to DOCUMENT tab
    app.tileset_view_state.set_sub_tab(DOCUMENT_SUB_TAB);

    // Frame 1 after switch to DOCUMENT
    let (doc_f1_rects, doc_f1_texts) = render_frame(&ctx, &mut app, full_rect);

    // Assert no leftover shapes from DEF tab
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "TILE ATLAS"),
        "DOCUMENT tab frame 1 must not contain TILE ATLAS label"
    );
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "TOOLS"),
        "DOCUMENT tab frame 1 must not contain TOOLS label"
    );
    assert!(
        !doc_f1_texts.iter().any(|(t, _)| t == "Source image"),
        "DOCUMENT tab frame 1 must not contain Source image label"
    );

    // Render frames 2, 3, 4 of DOCUMENT
    render_frame(&ctx, &mut app, full_rect);
    render_frame(&ctx, &mut app, full_rect);
    let (doc_f4_rects, doc_f4_texts) = render_frame(&ctx, &mut app, full_rect);

    // Frame 1 after switch must match frame 4 within 0.5 px
    assert_frames_match(
        &doc_f1_rects,
        &doc_f1_texts,
        &doc_f4_rects,
        &doc_f4_texts,
        "DOCUMENT",
    );

    // Simulate scrolling on DOCUMENT
    app.tileset_view_state.doc_scroll = 88.0;

    // 3. Switch back to DEF tab (must reset scroll)
    app.tileset_view_state.set_sub_tab(TILESET_DEF_SUB_TAB);
    assert_eq!(
        app.tileset_view_state.doc_scroll, 0.0,
        "Switching sub-tab must reset doc_scroll offset"
    );

    // Frame 1 after switch back to DEF
    let (def_f1_rects, def_f1_texts) = render_frame(&ctx, &mut app, full_rect);

    // Assert no leftover shapes from DOCUMENT tab (e.g. overview doc text)
    assert!(
        def_f1_texts.iter().any(|(t, _)| t == "TILE ATLAS"),
        "DEF tab frame 1 must contain TILE ATLAS label"
    );
    assert!(
        def_f1_texts.iter().any(|(t, _)| t == "TOOL ATTRIBUTES"),
        "DEF tab frame 1 must contain TOOL ATTRIBUTES label"
    );
    assert!(
        def_f1_texts.iter().any(|(t, _)| t.contains("(Tileset)")),
        "DEF tab frame 1 must immediately contain tileset header title"
    );

    // Render frames 2, 3, 4 of DEF
    render_frame(&ctx, &mut app, full_rect);
    render_frame(&ctx, &mut app, full_rect);
    let (def_f4_rects, def_f4_texts) = render_frame(&ctx, &mut app, full_rect);

    // Frame 1 after switch back must match frame 4 within 0.5 px
    assert_frames_match(
        &def_f1_rects,
        &def_f1_texts,
        &def_f4_rects,
        &def_f4_texts,
        "DEF",
    );
}

#[test]
fn test_tileset_five_sub_tabs_and_card_focus_scoping() {
    let mut p = Project::new("5 Tabs Test");
    let ts_id = TilesetDefId(1);
    p.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "WorldTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    });

    let mut app = ShellApp::new(p);
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    app.assets.set_project_dir(Some(repo_root.to_path_buf()));
    app.active_mode = AppMode::Tilesets;
    app.selected_tileset_def = Some(ts_id);
    app.tileset_view_state.select_tileset(ts_id);

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));

    // 1. TILESET DEF tab renders Attributes Card without "Marking enum:"
    app.tileset_view_state.set_sub_tab(TILESET_DEF_SUB_TAB);
    let (_, def_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(def_texts
        .iter()
        .any(|(t, _)| t == "TILESET IDENTIFIER & ATTRIBUTES"));
    assert!(!def_texts.iter().any(|(t, _)| t == "Marking enum:"));

    // 2. MARKING tab hides Attributes Card and shows "Marking enum:" selector even when source_enum is None
    app.tileset_view_state.set_sub_tab(MARKING_SUB_TAB);
    assert_eq!(app.tileset_view_state.active_mode, "Marking");
    let (_, marking_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(!marking_texts
        .iter()
        .any(|(t, _)| t == "TILESET IDENTIFIER & ATTRIBUTES"));
    assert!(marking_texts.iter().any(|(t, _)| t == "Marking enum:"));

    // 3. COLLISION tab renders TOOLS toolbar and hides Attributes Card
    app.tileset_view_state.set_sub_tab(COLLISION_SUB_TAB);
    assert_eq!(app.tileset_view_state.active_mode, "Collision");
    let (_, col_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(col_texts.iter().any(|(t, _)| t == "TOOLS"));
    assert!(!col_texts
        .iter()
        .any(|(t, _)| t == "TILESET IDENTIFIER & ATTRIBUTES"));

    // 4. ANIMATION tab renders TOOLS toolbar and hides Attributes Card
    app.tileset_view_state.set_sub_tab(ANIMATION_SUB_TAB);
    assert_eq!(app.tileset_view_state.active_mode, "Animation");
    let (_, anim_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(anim_texts.iter().any(|(t, _)| t == "TOOLS"));
    assert!(!anim_texts
        .iter()
        .any(|(t, _)| t == "TILESET IDENTIFIER & ATTRIBUTES"));

    // 5. Card focus scoping: when Attributes card is focused, Collision shortcut `B` does not change tool
    app.tileset_view_state.set_sub_tab(COLLISION_SUB_TAB);
    app.tileset_view_state.focused_card = Some(TilesetCard::Attributes);
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            events: vec![egui::Event::Key {
                key: egui::Key::B,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    out.textures_delta.clear();
    assert_eq!(
        app.tileset_view_state.collision_tool,
        lc_shell::features::tileset::state::CollisionTool::Select,
        "Collision shortcut B must NOT fire when Attributes card is focused"
    );

    // Focusing Atlas allows `B` shortcut to switch collision tool to Box
    app.tileset_view_state.focused_card = Some(TilesetCard::Atlas);
    let mut out2 = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(full_rect),
            events: vec![egui::Event::Key {
                key: egui::Key::B,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            render_tileset_view(&mut app, ui);
        },
    );
    out2.textures_delta.clear();
    assert_eq!(
        app.tileset_view_state.collision_tool,
        lc_shell::features::tileset::state::CollisionTool::Box,
        "Collision shortcut B must switch tool when Atlas card is focused"
    );
}

#[test]
fn test_tileset_def_inspector_marking_full_width_animation_shortcut_and_photocraft_theme() {
    let mut p = Project::new("Tileset Improvements Test");
    let ts_id = TilesetDefId(1);
    p.defs.tilesets.push(TilesetDef {
        id: ts_id,
        name: "WorldTiles".to_string(),
        path: "assets/Bg/tileset_picker_bg_netral.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
        collisions: Default::default(),
    });

    let mut app = ShellApp::new(p);
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    app.assets.set_project_dir(Some(repo_root.to_path_buf()));
    app.active_mode = AppMode::Tilesets;
    app.selected_tileset_def = Some(ts_id);
    app.tileset_view_state.select_tileset(ts_id);

    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::ProDark);
    assert_eq!(theme::Tokens::get(&ctx), theme::Tokens::PRO_DARK);
    theme::apply(&ctx, ThemeChoice::Dark);
    assert_eq!(theme::Tokens::get(&ctx), theme::Tokens::DARK);

    let full_rect = Rect::from_min_size(egui::Pos2::ZERO, vec2(1366.0, 768.0));

    // 1. TILESET DEF right panel shows TILESET OVERVIEW when no tile selected, and Tile Inspector when selected
    app.tileset_view_state.set_sub_tab(TILESET_DEF_SUB_TAB);
    let (_, overview_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(overview_texts.iter().any(|(t, _)| t == "TILESET OVERVIEW"));

    app.tileset_view_state.selected_tile = Some(0);
    let (_, inspector_texts) = render_frame(&ctx, &mut app, full_rect);
    assert!(inspector_texts.iter().any(|(t, _)| t == "CUSTOM DATA"));
    assert!(inspector_texts.iter().any(|(t, _)| t == "TILE SUMMARY"));

    // 2. MARKING tab dropdown spans full width of right panel (> 200 px inside 240 px card)
    app.tileset_view_state.set_sub_tab(MARKING_SUB_TAB);
    let (marking_rects, _) = render_frame(&ctx, &mut app, full_rect);
    let dropdown_rect = marking_rects
        .iter()
        .find(|r| r.min.x > 1114.0 && r.width() > 200.0 && (r.height() - 32.0).abs() < 0.5);
    assert!(
        dropdown_rect.is_some(),
        "Marking enum dropdown must span full width (> 200px) inside 240px right panel"
    );

    // 3. ANIMATION tab: plain click without anchor does NOT auto-create an anchor
    app.tileset_view_state.set_sub_tab(ANIMATION_SUB_TAB);
    app.tileset_view_state.selected_animation_anchor = None;
    let (anim_rects, _) = render_frame(&ctx, &mut app, full_rect);
    let atlas_rect = anim_rects
        .iter()
        .find(|r| {
            (r.min.x - 272.0).abs() < 0.5 && (r.max.x - 1102.0).abs() < 0.5 && r.height() > 200.0
        })
        .expect("Atlas rect in animation tab");
    let click_pos = atlas_rect.center();

    for pressed in [true, false] {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(full_rect),
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
                render_tileset_view(&mut app, ui);
            },
        );
        out.textures_delta.clear();
        app.process_commands_and_events();
    }
    let _ = render_frame(&ctx, &mut app, full_rect);
    assert!(
        app.project.defs.tilesets[0].animations.is_empty(),
        "Plain click without selected anchor must NOT auto-create an animation anchor"
    );

    // Ctrl+Click on the atlas tile DOES create an animation anchor
    for pressed in [true, false] {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(full_rect),
                events: vec![
                    egui::Event::ModifiersChanged(egui::Modifiers::CTRL),
                    egui::Event::PointerMoved(click_pos),
                    egui::Event::PointerButton {
                        pos: click_pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::CTRL,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                render_tileset_view(&mut app, ui);
            },
        );
        out.textures_delta.clear();
        app.process_commands_and_events();
    }
    let _ = render_frame(&ctx, &mut app, full_rect);
    assert!(
        !app.project.defs.tilesets[0].animations.is_empty(),
        "Ctrl+Click must create an animation anchor"
    );
    assert!(app.tileset_view_state.selected_animation_anchor.is_some());

    // 4. Frame drag-reorder helper + SetTileAnimationCommand roundtrip
    let anchor_id = app.tileset_view_state.selected_animation_anchor.unwrap();
    let mut anim = lc_core::defs::TileAnimationDef::new(anchor_id);
    anim.frames = vec![anchor_id, 1, 2];
    let mut frames = anim.frames.clone();
    assert!(lc_shell::shared::ui::drag_list::reorder_item(
        &mut frames,
        2,
        0
    ));
    assert_eq!(frames, vec![2, anchor_id, 1]);
}
