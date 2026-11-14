//! Regression test for Markdown list markers in shared document tabs.
//!
//! The app theme reserves `spacing.interact_size.y = 28 px` per row, while
//! `egui_commonmark` paints list markers in a Body-height (16 px) slot. That
//! mismatch used to push every bullet and number ~6 px above its text line and
//! left the markers white. The shared document view scopes the interact size to
//! the Body height and points the marker color at the accent, so this test
//! measures the rendered shapes instead of trusting the code path.

use egui::{pos2, vec2, Color32, Context, Pos2, RawInput, Rect};
use lc_canvas::theme::{self, ThemeChoice, Tokens};
use lc_shell::shared::ui::markdown_document::render_markdown_document;

const MARKDOWN: &str = "\
- Bullet item
- A wrapped bullet item that is long enough to need a second line at the narrow test width
- Third bullet item

10. Tenth item
11. Eleventh item
12. Twelfth item
";

/// One painted text run, with the geometry the assertions need.
#[derive(Debug)]
struct RenderedText {
    text: String,
    /// Center of the whole galley (all rows).
    center: Pos2,
    /// Center of the first text row, where a list marker must sit.
    first_row_center: Pos2,
    rows: usize,
    color: Color32,
}

/// Painted text and bullet circles collected from one frame.
#[derive(Default)]
struct Collected {
    texts: Vec<RenderedText>,
    bullets: Vec<egui::epaint::CircleShape>,
}

fn collect(shape: &egui::Shape, out: &mut Collected) {
    match shape {
        egui::Shape::Text(text) => {
            let center = text.pos + text.galley.size() / 2.0;
            let first_row_center = text
                .galley
                .rows
                .first()
                .map(|row| text.pos + row.rect().center().to_vec2())
                .unwrap_or(center);
            out.texts.push(RenderedText {
                text: text.galley.text().to_owned(),
                center,
                first_row_center,
                rows: text.galley.rows.len(),
                color: text.fallback_color,
            });
        }
        egui::Shape::Circle(circle) => out.bullets.push(*circle),
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                collect(shape, out);
            }
        }
        _ => {}
    }
}

/// Renders the controlled Markdown through the production document renderer.
fn render() -> (Collected, Tokens) {
    let ctx = Context::default();
    theme::install_fonts(&ctx);
    theme::apply(&ctx, ThemeChoice::Dark);
    let tokens = Tokens::get(&ctx);

    let mut collected = Collected::default();
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(360.0, 800.0))),
            ..Default::default()
        },
        |ui| {
            ui.set_max_width(320.0);
            let mut cache = egui_commonmark::CommonMarkCache::default();
            render_markdown_document(ui, &tokens, &mut cache, MARKDOWN);
        },
    );
    // Dropping a `FullOutput` with unapplied textures panics.
    output.textures_delta.clear();
    for clipped in &output.shapes {
        collect(&clipped.shape, &mut collected);
    }
    (collected, tokens)
}

fn text<'a>(collected: &'a Collected, needle: &str) -> &'a RenderedText {
    collected
        .texts
        .iter()
        .find(|text| text.text == needle)
        .unwrap_or_else(|| {
            let seen: Vec<&str> = collected.texts.iter().map(|t| t.text.as_str()).collect();
            panic!("missing text `{needle}`; rendered: {seen:?}")
        })
}

#[test]
fn bullet_markers_sit_on_their_text_line_in_the_accent_color() {
    let (collected, tokens) = render();

    assert_eq!(
        collected.bullets.len(),
        3,
        "one filled bullet per top-level item, got {:?}",
        collected.bullets
    );
    for bullet in &collected.bullets {
        assert_eq!(
            bullet.fill, tokens.accent,
            "bullet markers must use the accent color"
        );
    }

    let item = text(&collected, "Bullet item");
    let offset = (collected.bullets[0].center.y - item.first_row_center.y).abs();
    assert!(
        offset < 1.0,
        "bullet marker is {offset:.2} px off its text center (marker {}, text {})",
        collected.bullets[0].center.y,
        item.first_row_center.y
    );
}

#[test]
fn numbered_markers_sit_on_their_text_line_in_the_accent_color() {
    let (collected, tokens) = render();

    for (marker, item) in [
        ("10.", "Tenth item"),
        ("11.", "Eleventh item"),
        ("12.", "Twelfth item"),
    ] {
        let marker = text(&collected, marker);
        assert_eq!(
            marker.color, tokens.accent,
            "number markers must use the accent color"
        );

        let item = text(&collected, item);
        let offset = (marker.center.y - item.first_row_center.y).abs();
        assert!(
            offset < 1.0,
            "`{}` marker is {offset:.2} px off `{}` (marker {}, text {})",
            marker.text,
            item.text,
            marker.center.y,
            item.first_row_center.y
        );
    }
}

#[test]
fn wrapped_items_keep_the_marker_on_the_first_text_line() {
    let (collected, _tokens) = render();

    let wrapped = text(
        &collected,
        "A wrapped bullet item that is long enough to need a second line at the narrow test width",
    );
    assert!(
        wrapped.rows >= 2,
        "the long item must wrap, but it has {} row(s)",
        wrapped.rows
    );

    let offset = (collected.bullets[1].center.y - wrapped.first_row_center.y).abs();
    assert!(
        offset < 1.0,
        "wrapped item marker is {offset:.2} px off its first text line (marker {}, text {})",
        collected.bullets[1].center.y,
        wrapped.first_row_center.y
    );
}
