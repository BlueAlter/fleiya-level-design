//! Headless tests for the enum DOCUMENT sub-tab.
//!
//! The tab renders the embedded Markdown overview instead of the definition
//! form, while the title row stays visible.

use egui::{Context, Id, RawInput};
use lc_core::defs::EnumDef;
use lc_core::id::EnumDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::enums::view::EnumViewState;
use lc_shell::ShellApp;

const STATE_ID: &str = "enum_mode_view_transient_state";

/// Collects every text fragment egui produced this frame.
fn rendered_text(shapes: &[egui::epaint::ClippedShape]) -> String {
    fn walk(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(text) => {
                out.push_str(text.galley.text());
                out.push('\n');
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    walk(shape, out);
                }
            }
            _ => {}
        }
    }

    let mut out = String::new();
    for clipped in shapes {
        walk(&clipped.shape, &mut out);
    }
    out
}

/// Renders the enum mode with `active_sub_tab` preselected and returns its text.
fn render_tab(active_sub_tab: usize) -> String {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let mut app = ShellApp::new(Project::new("Document Tab Test"));
    app.active_mode = AppMode::Enums;
    app.enum_draft = vec![EnumDef {
        id: EnumDefId(1),
        name: "ItemRarity".to_string(),
        tileset_id: None,
        tags: Vec::new(),
        values: Vec::new(),
    }];
    app.selected_enum_def = Some(EnumDefId(1));

    let state = EnumViewState {
        active_sub_tab,
        ..Default::default()
    };
    ctx.data_mut(|data| data.insert_temp(Id::new(STATE_ID), state));

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    rendered_text(&output.shapes)
}

#[test]
fn document_tab_renders_the_markdown_overview() {
    let text = render_tab(1);
    assert!(
        text.contains("Create an enum"),
        "the overview heading must be rendered, got:\n{text}"
    );
    assert!(
        text.contains("Tags are labels that only"),
        "body text must be rendered"
    );
    assert!(
        text.contains("ItemRarity (Enum)"),
        "the title row must stay visible on the DOCUMENT tab"
    );
    assert!(
        !text.contains("ENUM IDENTIFIER & ATTRIBUTES"),
        "the definition form must be hidden on the DOCUMENT tab"
    );
}

#[test]
fn definition_tab_keeps_showing_the_form() {
    let text = render_tab(0);
    assert!(
        text.contains("ENUM IDENTIFIER & ATTRIBUTES"),
        "the definition form is shown on the ENUM DEF tab"
    );
    assert!(
        !text.contains("Create an enum"),
        "the document must not render on the ENUM DEF tab"
    );
}
