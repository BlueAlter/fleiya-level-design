use egui::epaint::{ClippedShape, RectShape, TextShape};
use egui::{vec2, Context, FullOutput, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_canvas::icons::Icon;
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::editor::shell_app::TagDef;
use lc_shell::features::enums::view::{render_enum_view, ENUM_CARD_HEIGHT};
use lc_shell::ShellApp;

const P: f32 = 12.0;
const TOLERANCE: f32 = 0.5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOLERANCE
}

#[derive(Default)]
struct CollectedShapes {
    rects: Vec<RectShape>,
    texts: Vec<TextShape>,
}

fn collect_shapes(shape: &Shape, out: &mut CollectedShapes) {
    match shape {
        Shape::Rect(r) => out.rects.push(r.clone()),
        Shape::Text(t) => out.texts.push(t.clone()),
        Shape::Vec(vec) => {
            for s in vec {
                collect_shapes(s, out);
            }
        }
        _ => {}
    }
}

fn collect_from_output(shapes: &[ClippedShape]) -> CollectedShapes {
    let mut collected = CollectedShapes::default();
    for clipped in shapes {
        collect_shapes(&clipped.shape, &mut collected);
    }
    collected
}

fn setup_app(enum_count: usize) -> ShellApp {
    let mut app = ShellApp::new(Project::new("Rect Test"));
    app.active_mode = AppMode::Enums;

    app.project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Dungeon_16".to_string(),
        path: "assets/tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
    });

    app.tag_registry.push(TagDef {
        id: lc_shell::features::editor::shell_app::TagId(1),
        name: "Gameplay".to_string(),
    });
    app.tag_registry.push(TagDef {
        id: lc_shell::features::editor::shell_app::TagId(2),
        name: "Audio".to_string(),
    });

    for i in 1..=enum_count {
        let enum_id = EnumDefId(i as u64);
        let mut values = Vec::new();
        if i == 1 {
            values.push(EnumValueDef {
                id: "Common".to_string(),
                name: "Common Item".to_string(),
                color: [156, 163, 175, 255],
                tile: None,
            });
            values.push(EnumValueDef {
                id: "Rare".to_string(),
                name: "Rare Item".to_string(),
                color: [61, 214, 184, 255],
                tile: None,
            });
        }
        app.enum_draft.push(EnumDef {
            id: enum_id,
            name: format!("Enum_{i}"),
            tileset_id: Some(TilesetDefId(1)),
            tags: Vec::new(),
            values,
        });
    }

    app.selected_enum_def = Some(EnumDefId(1));
    app.enum_draft_revision = 1;
    app.enum_saved_revision = 0;

    app
}

fn render_view(app: &mut ShellApp, width: f32, height: f32) -> (FullOutput, CollectedShapes) {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(width, height))),
        ..Default::default()
    };

    let mut output = ctx.run_ui(raw_input, |ui| {
        render_enum_view(app, ui);
    });
    output.textures_delta.clear();

    let collected = collect_from_output(&output.shapes);
    (output, collected)
}

struct MeasuredWorkspace {
    left_panel_rect: Rect,
    new_enum_button_rect: Rect,
    left_kebab_button_rect: Rect,
    search_rect: Rect,
    header_text_pos: Pos2,
    card_row_rects: Vec<Rect>,
    sub_tabs_rect: Rect,
    detail_card_rect: Rect,
    identifier_control_rect: Rect,
    export_button_rect: Rect,
    tileset_control_rect: Rect,
    tags_first_control_rect: Rect,
    tag_plus_button_rect: Rect,
    tag_recall_button_rect: Rect,
    add_value_button_rect: Rect,
    value_trash_button_rects: Vec<Rect>,
    footer_text_pos: Pos2,
    save_button_rect: Rect,
    revert_button_rect: Rect,
}

fn measure_workspace(collected: &CollectedShapes, height: f32) -> MeasuredWorkspace {
    // 1. Left panel background (starts at 0, 0, max.y == height)
    let left_panel_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| approx_eq(r.min.x, 0.0) && approx_eq(r.min.y, 0.0) && approx_eq(r.max.y, height))
        .expect("left panel background rect not found");

    // 2. New Enum button (starts at 12, 12, height 28)
    let new_enum_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| {
            approx_eq(r.min.x, 12.0) && approx_eq(r.min.y, 12.0) && approx_eq(r.height(), 28.0)
        })
        .expect("New Enum button rect not found");

    // 3. Left toolbar kebab ("⋮") button (next to New Enum button, same y row)
    let left_kebab_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| {
            r.min.x > 200.0
                && r.min.x < 260.0
                && approx_eq(r.min.y, 12.0)
                && approx_eq(r.height(), 28.0)
        })
        .expect("left kebab button rect not found");

    // 4. Search box (y between 45 and 90, starts at 12)
    let search_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| approx_eq(r.min.x, 12.0) && r.min.y > 45.0 && r.min.y < 90.0)
        .expect("search box rect not found");

    // 5. Header text "ENUM DEFINITIONS"
    let header_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text().contains("ENUM DEFINITIONS"))
        .expect("header text not found");
    let header_text_pos = header_text.pos;

    // 6. Card row rects in the left list
    let card_row_rects: Vec<Rect> = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .filter(|r| {
            approx_eq(r.min.x, 12.0) && approx_eq(r.height(), ENUM_CARD_HEIGHT) && r.min.y >= 100.0
        })
        .collect();
    assert!(!card_row_rects.is_empty(), "card row rects not found");

    // 7. Sub-tabs rect ("ENUM DEF")
    let sub_tabs_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "ENUM DEF")
        .expect("ENUM DEF tab text not found");
    let sub_tabs_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(sub_tabs_text.pos) && r.height() <= 40.0 && r.width() < 200.0)
        .expect("sub_tabs tab rect not found");

    // 8. Detail card frame "ENUM IDENTIFIER & ATTRIBUTES"
    let detail_card_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.min.x > 260.0 && r.min.y > 100.0 && r.min.y < 200.0 && r.width() > 800.0)
        .expect("detail card frame rect not found");

    // 9. Identifier control text edit rect
    let id_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "Enum_1" && t.pos.x > 260.0)
        .expect("Enum_1 text in detail panel not found");
    let identifier_control_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| {
            r.contains(id_text.pos)
                && r.height() >= 28.0
                && r.height() <= 40.0
                && r.width() < 1000.0
        })
        .expect("identifier text edit rect not found");

    // 10. Export button enclosing Icon::Export glyph (\u{eaf0})
    let export_glyph = Icon::Export.glyph();
    let export_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == export_glyph)
        .expect("export icon text not found");
    let export_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(export_text.pos) && r.width() < 60.0)
        .expect("export button rect not found");

    // 11. Tileset combo control
    let tileset_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "Dungeon_16")
        .expect("Dungeon_16 text not found");
    let tileset_control_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| {
            r.contains(tileset_text.pos)
                && r.height() >= 28.0
                && r.height() <= 40.0
                && r.width() < 300.0
        })
        .expect("tileset combo rect not found");

    // 12. Tags row first control: "(No tag)" rect
    let no_tag_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "(No tag)")
        .expect("(No tag) text not found");
    let tags_first_control_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| {
            r.contains(no_tag_text.pos)
                && r.height() >= 28.0
                && r.height() <= 40.0
                && r.width() < 200.0
        })
        .expect("tags first control rect not found");

    // 13. Tag "+" button and "⌄" button enclosing Icon::Add and Icon::ChevronDown glyphs
    let add_glyph = Icon::Add.glyph();
    let tag_plus_text = collected
        .texts
        .iter()
        .find(|t| {
            t.galley.text() == add_glyph && t.pos.x > 300.0 && t.pos.y > 200.0 && t.pos.y < 280.0
        })
        .expect("tag add glyph not found");
    let tag_plus_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(tag_plus_text.pos) && r.width() < 60.0)
        .expect("tag plus button rect not found");

    let chevron_glyph = Icon::ChevronDown.glyph();
    let tag_recall_text = collected
        .texts
        .iter()
        .find(|t| {
            t.galley.text() == chevron_glyph
                && t.pos.x > tag_plus_button_rect.max.x
                && t.pos.y > 200.0
                && t.pos.y < 280.0
        })
        .expect("tag recall chevron glyph not found");
    let tag_recall_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(tag_recall_text.pos) && r.width() < 60.0)
        .expect("tag recall button rect not found");

    // 14. "+ Add Value" button
    let add_val_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "+ Add Value")
        .expect("+ Add Value text not found");
    let add_value_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(add_val_text.pos) && r.height() >= 28.0 && r.width() < 200.0)
        .expect("+ Add Value button rect not found");

    // 15. Value row trash buttons enclosing Icon::Delete glyph
    let del_glyph = Icon::Delete.glyph();
    let mut value_trash_button_rects: Vec<Rect> = Vec::new();
    for t in collected
        .texts
        .iter()
        .filter(|t| t.galley.text() == del_glyph)
    {
        if let Some(r) = collected
            .rects
            .iter()
            .map(|r| r.rect)
            .find(|r| r.contains(t.pos) && r.width() < 60.0)
        {
            value_trash_button_rects.push(r);
        }
    }

    // 16. Footer text
    let footer_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text().starts_with("ID: EnumDefId"))
        .expect("footer text not found");
    let footer_text_pos = footer_text.pos;

    // 17. Save and Revert buttons
    let save_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "Save Enum Def")
        .expect("Save button text not found");
    let save_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(save_text.pos) && r.width() < 200.0)
        .expect("save button rect not found");

    let revert_text = collected
        .texts
        .iter()
        .find(|t| t.galley.text() == "Revert")
        .expect("Revert button text not found");
    let revert_button_rect = collected
        .rects
        .iter()
        .map(|r| r.rect)
        .find(|r| r.contains(revert_text.pos) && r.width() < 200.0)
        .expect("revert button rect not found");

    MeasuredWorkspace {
        left_panel_rect,
        new_enum_button_rect,
        left_kebab_button_rect,
        search_rect,
        header_text_pos,
        card_row_rects,
        sub_tabs_rect,
        detail_card_rect,
        identifier_control_rect,
        export_button_rect,
        tileset_control_rect,
        tags_first_control_rect,
        tag_plus_button_rect,
        tag_recall_button_rect,
        add_value_button_rect,
        value_trash_button_rects,
        footer_text_pos,
        save_button_rect,
        revert_button_rect,
    }
}

fn assert_workspace_rects(m: &MeasuredWorkspace, screen_width: f32) {
    // 1. Left panel fill x 0..260
    assert!(
        approx_eq(m.left_panel_rect.min.x, 0.0) && approx_eq(m.left_panel_rect.max.x, 260.0),
        "left panel fill must be 0..260, was {:?}",
        m.left_panel_rect
    );

    // 2. Every left-column element spans 12..248
    // - New button row spans 12..248 (new_btn min.x=12, kebab max.x=248)
    assert!(
        approx_eq(m.new_enum_button_rect.min.x, 12.0),
        "new button row must start at 12, was {}",
        m.new_enum_button_rect.min.x
    );
    assert!(
        approx_eq(m.left_kebab_button_rect.max.x, 248.0),
        "new button row must end at 248, was {}",
        m.left_kebab_button_rect.max.x
    );
    assert!(
        approx_eq(m.left_kebab_button_rect.width(), 28.0)
            && approx_eq(m.left_kebab_button_rect.height(), 28.0),
        "left kebab button must be 28x28, was {:?}",
        m.left_kebab_button_rect
    );

    // - Search box spans 12..248
    assert!(
        approx_eq(m.search_rect.min.x, 12.0) && approx_eq(m.search_rect.max.x, 248.0),
        "search box must span 12..248, was {:?}",
        m.search_rect
    );

    // - Header text starts at 12
    assert!(
        approx_eq(m.header_text_pos.x, 12.0),
        "header text must start at 12, was {}",
        m.header_text_pos.x
    );

    // - Each row in cards list spans 12..248
    for (i, row) in m.card_row_rects.iter().enumerate() {
        assert!(
            approx_eq(row.min.x, 12.0) && approx_eq(row.max.x, 248.0),
            "card row {} must span 12..248, was {:?}",
            i,
            row
        );
    }

    // 3. Detail content: left = 260 + P, right = 1366 - P, top = P; card edges equal content edges
    let expected_left = 260.0 + P;
    let expected_right = screen_width - P;
    assert!(
        approx_eq(m.detail_card_rect.min.x, expected_left)
            && approx_eq(m.detail_card_rect.max.x, expected_right),
        "card edges must equal content edges {}..{}, was {:?}",
        expected_left,
        expected_right,
        m.detail_card_rect
    );

    // 4. sub_tabs row and the "+ New Enum" button share the same top and the same height (28)
    assert!(
        approx_eq(m.sub_tabs_rect.min.y, m.new_enum_button_rect.min.y),
        "sub_tabs top ({}) must equal new enum button top ({})",
        m.sub_tabs_rect.min.y,
        m.new_enum_button_rect.min.y
    );
    assert!(
        approx_eq(m.sub_tabs_rect.height(), 28.0),
        "sub_tabs height must be 28, was {}",
        m.sub_tabs_rect.height()
    );

    // 5. First control of Identifier, Tileset and Tags rows start at the same x
    assert!(
        approx_eq(
            m.identifier_control_rect.min.x,
            m.tileset_control_rect.min.x
        ),
        "identifier min.x ({}) must equal tileset min.x ({})",
        m.identifier_control_rect.min.x,
        m.tileset_control_rect.min.x
    );
    assert!(
        approx_eq(
            m.tileset_control_rect.min.x,
            m.tags_first_control_rect.min.x
        ),
        "tileset min.x ({}) must equal tags first control min.x ({})",
        m.tileset_control_rect.min.x,
        m.tags_first_control_rect.min.x
    );

    // 6. Icon buttons are square at their tier:
    // kebab 28x28, export 32x32, Tags "+" 32x32, "⌄" 32x32, trash 32x32
    assert!(
        approx_eq(m.export_button_rect.width(), 32.0)
            && approx_eq(m.export_button_rect.height(), 32.0),
        "export button must be 32x32, was {:?}",
        m.export_button_rect
    );
    assert!(
        approx_eq(m.tag_plus_button_rect.width(), 32.0)
            && approx_eq(m.tag_plus_button_rect.height(), 32.0),
        "tag plus button must be 32x32, was {:?}",
        m.tag_plus_button_rect
    );
    assert!(
        approx_eq(m.tag_recall_button_rect.width(), 32.0)
            && approx_eq(m.tag_recall_button_rect.height(), 32.0),
        "tag recall button must be 32x32, was {:?}",
        m.tag_recall_button_rect
    );
    for (i, trash_rect) in m.value_trash_button_rects.iter().enumerate() {
        assert!(
            approx_eq(trash_rect.width(), 32.0) && approx_eq(trash_rect.height(), 32.0),
            "trash button {} must be 32x32, was {:?}",
            i,
            trash_rect
        );
    }

    // 7. "+ Add Value", Revert and Save right edge = 1366 - P; footer text left = 260 + P
    assert!(
        approx_eq(m.add_value_button_rect.max.x, expected_right),
        "+ Add Value button right edge must be {}, was {}",
        expected_right,
        m.add_value_button_rect.max.x
    );
    assert!(
        approx_eq(m.save_button_rect.max.x, expected_right),
        "Save button right edge must be {}, was {}",
        expected_right,
        m.save_button_rect.max.x
    );
    assert!(
        m.revert_button_rect.max.x < m.save_button_rect.min.x,
        "Revert button must be to the left of Save button"
    );
    assert!(
        approx_eq(m.footer_text_pos.x, expected_left),
        "Footer text left must be {}, was {}",
        expected_left,
        m.footer_text_pos.x
    );
}

#[test]
fn test_enum_workspace_rects_1366x768() {
    let mut app = setup_app(3);
    let (_, collected) = render_view(&mut app, 1366.0, 768.0);
    let m = measure_workspace(&collected, 768.0);
    assert_workspace_rects(&m, 1366.0);
}

#[test]
fn test_enum_workspace_rects_scroll_height_500() {
    let mut app = setup_app(12);
    let (_, collected) = render_view(&mut app, 1366.0, 500.0);
    let m = measure_workspace(&collected, 500.0);
    assert!(
        approx_eq(m.left_panel_rect.min.x, 0.0) && approx_eq(m.left_panel_rect.max.x, 260.0),
        "left panel fill must be 0..260 at 500h, was {:?}",
        m.left_panel_rect
    );
    assert!(
        approx_eq(m.new_enum_button_rect.min.x, 12.0)
            && approx_eq(m.left_kebab_button_rect.max.x, 248.0),
        "new button row must span 12..248 at 500h"
    );
    assert!(
        approx_eq(m.left_kebab_button_rect.width(), 28.0)
            && approx_eq(m.left_kebab_button_rect.height(), 28.0),
        "left kebab button must be 28x28 at 500h"
    );
    assert!(
        approx_eq(m.search_rect.min.x, 12.0) && approx_eq(m.search_rect.max.x, 248.0),
        "search box must span 12..248 at 500h"
    );
    assert!(
        approx_eq(m.header_text_pos.x, 12.0),
        "header text must start at 12 at 500h"
    );
    for (i, row) in m.card_row_rects.iter().enumerate() {
        assert!(
            approx_eq(row.min.x, 12.0) && approx_eq(row.max.x, 248.0),
            "card row {} must span 12..248 at 500h, was {:?}",
            i,
            row
        );
    }
}
