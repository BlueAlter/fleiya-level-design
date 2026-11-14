//! Headless tests for the enum DOCUMENT sub-tab.
//!
//! The tab renders the embedded Markdown overview instead of the definition
//! form. The title row and the pinned footer describe the definition being
//! edited, so they belong to ENUM DEF only and must not appear here.

use egui::{Context, Id, RawInput};
use lc_core::defs::EnumDef;
use lc_core::id::EnumDefId;
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::enums::view::EnumViewState;
use lc_shell::ShellApp;

const STATE_ID: &str = "enum_mode_view_transient_state";

/// What one rendered frame produced.
struct Rendered {
    text: String,
    app: ShellApp,
    dirty: bool,
}

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

/// Renders the enum mode with `active_sub_tab` preselected.
fn render_tab(active_sub_tab: usize) -> Rendered {
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
    let dirty = app.enum_is_dirty();

    let state = EnumViewState {
        active_sub_tab,
        ..Default::default()
    };
    ctx.data_mut(|data| data.insert_temp(Id::new(STATE_ID), state));

    let mut output = ctx.run_ui(RawInput::default(), |ui| {
        app.ui(ui);
    });
    output.textures_delta.clear();

    Rendered {
        text: rendered_text(&output.shapes),
        app,
        dirty,
    }
}

#[test]
fn document_tab_renders_only_the_markdown_overview() {
    let rendered = render_tab(1);
    let text = &rendered.text;

    assert!(
        text.contains("Create an enum"),
        "the overview heading must be rendered, got:\n{text}"
    );
    assert!(
        text.contains("Tags are labels that only"),
        "body text must be rendered"
    );

    // Definition chrome: title row, usage footer and Save/Revert actions.
    assert!(
        !text.contains("ItemRarity (Enum)"),
        "the definition title must not render on the DOCUMENT tab"
    );
    assert!(
        !text.contains("ENUM IDENTIFIER & ATTRIBUTES"),
        "the definition form must be hidden on the DOCUMENT tab"
    );
    assert!(
        !text.contains("Save Enum Def"),
        "the Save action belongs to ENUM DEF only"
    );
    assert!(
        !text.contains("Revert"),
        "the Revert action belongs to ENUM DEF only"
    );
    assert!(
        !text.contains("Used by"),
        "the usage footer belongs to ENUM DEF only"
    );

    // Switching the sub-tab must not mutate the draft or its dirty state.
    assert_eq!(rendered.app.enum_draft[0].name, "ItemRarity");
    assert_eq!(rendered.app.enum_is_dirty(), rendered.dirty);
}

#[test]
fn definition_tab_keeps_title_form_and_footer() {
    let rendered = render_tab(0);
    let text = &rendered.text;

    assert!(
        text.contains("ItemRarity (Enum)"),
        "the definition title is shown on the ENUM DEF tab"
    );
    assert!(
        text.contains("ENUM IDENTIFIER & ATTRIBUTES"),
        "the definition form is shown on the ENUM DEF tab"
    );
    assert!(
        text.contains("Save Enum Def"),
        "the footer Save action is shown on the ENUM DEF tab"
    );
    assert!(
        text.contains("Revert"),
        "the footer Revert action is shown on the ENUM DEF tab"
    );
    assert!(
        !text.contains("Create an enum"),
        "the document must not render on the ENUM DEF tab"
    );

    assert_eq!(rendered.app.enum_is_dirty(), rendered.dirty);
}
