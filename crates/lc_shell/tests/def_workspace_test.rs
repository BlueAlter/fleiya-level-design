use egui::Context;
use lc_canvas::theme::Tokens;
use lc_shell::shared::ui::def_workspace::{
    def_field_row, def_footer, def_left_panel, def_panel_width, def_right_header, def_row_layout,
    def_section, format_footer_left, DefItem, DefLeftPanelConfig, DEF_CARD_HEIGHT,
    DEF_HANDLE_WIDTH, DEF_KEBAB_WIDTH,
};
use lc_shell::shared::ui::DragListState;

#[test]
fn test_def_panel_width_constant() {
    assert_eq!(def_panel_width(), 260.0);
}

#[test]
fn test_def_row_layout_geometry() {
    let r = egui::Rect::from_min_size(egui::pos2(10.0, 50.0), egui::vec2(240.0, DEF_CARD_HEIGHT));
    let layout = def_row_layout(r);

    assert!(layout.handle.min.x >= r.min.x);
    assert!(layout.handle.max.x < layout.name.min.x);
    assert!(layout.name.max.x < layout.kebab.min.x);
    assert!(layout.kebab.max.x <= r.max.x);
    assert_eq!(layout.handle.width(), DEF_HANDLE_WIDTH);
    assert_eq!(layout.kebab.width(), DEF_KEBAB_WIDTH);
}

#[test]
fn test_format_footer_left() {
    assert_eq!(
        format_footer_left("EnumDefId(1)", "2 Entity Def(s)", false),
        "ID: EnumDefId(1) • Used by 2 Entity Def(s)"
    );
    assert_eq!(
        format_footer_left("TilesetDefId(2)", "4 Level(s)", true),
        "ID: TilesetDefId(2) • Used by 4 Level(s) • Unsaved changes"
    );
}

#[test]
fn test_headless_render_def_left_panel() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);
    let mut search = String::new();
    let mut drag = DragListState::new();

    let items = vec![
        DefItem::new(1u32, "Item One"),
        DefItem::new(2u32, "Item Two"),
        DefItem::new(3u32, "Item Three"),
    ];

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let config = DefLeftPanelConfig::new("+ New Item", "TEST DEFINITIONS").with_menu(true);
        let actions = def_left_panel(
            ui,
            &tokens,
            &config,
            &mut search,
            &items,
            Some(2),
            &mut drag,
        );
        assert!(actions.is_empty());
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_right_header() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);
    let mut active_tab = 0;

    let mut output = ctx.run_ui(Default::default(), |ui| {
        let changed = def_right_header(
            ui,
            &tokens,
            &mut active_tab,
            &["DEF", "DOCUMENT"],
            "BiomeType",
            "Enum",
            "A test enum definition",
        );
        assert!(!changed);
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_section() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_section(ui, &tokens, "TEST SECTION", |ui| {
            ui.label("Inside section card");
        });
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_field_row() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_field_row(ui, &tokens, "Identifier", |ui| {
            let mut s = "value".to_string();
            ui.text_edit_singleline(&mut s);
        });
    });
    output.textures_delta.clear();
}

#[test]
fn test_headless_render_def_footer() {
    let ctx = Context::default();
    let tokens = Tokens::get(&ctx);

    let mut output = ctx.run_ui(Default::default(), |ui| {
        def_footer(ui, &tokens, "ID: DefId(1) • Used by 0", |ui| {
            let _ = ui.button("Save");
        });
    });
    output.textures_delta.clear();
}
