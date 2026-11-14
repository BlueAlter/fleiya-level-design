//! Shared Markdown document page for DOCUMENT tabs.
//!
//! Long-form documentation lives in `assets/localization/<lang>/docs/...` and is
//! registered as a catalog by its domain (see `lc_ui_api::i18n`). This module
//! owns the view state and the list layout shared by every document tab; the
//! domain only chooses the topic/page and the scroll id.
//!
//! `egui_commonmark` sizes list marker slots from the body text height while
//! the app theme reserves `spacing.interact_size.y` (28 px) per row, which puts
//! the marker ~6 px above its text line. Scoping the interact size to the body
//! height fixes the alignment; markers (and bold text) take their color from
//! `visuals.widgets.active`, set to the accent here. Both overrides live in
//! this scope, so the global theme is never touched.

use std::sync::{Arc, Mutex};

use egui::{FontId, ScrollArea, TextStyle, Ui};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use lc_canvas::theme::Tokens;

/// Markdown cache of a DOCUMENT sub-tab (view state, never in `Project`).
pub type DocumentCache = Arc<Mutex<CommonMarkCache>>;

/// Creates an empty cache for a new document tab.
pub fn new_document_cache() -> DocumentCache {
    Arc::new(Mutex::new(CommonMarkCache::default()))
}

/// Applies the DOCUMENT-only list layout before Markdown is rendered.
fn apply_document_list_layout(ui: &mut Ui, t: &Tokens) {
    let body = ui.text_style_height(&TextStyle::Body);
    ui.spacing_mut().interact_size.y = body;
    ui.visuals_mut().widgets.active.fg_stroke.color = t.accent;
}

/// Renders `markdown` with the DOCUMENT list layout (Body-height rows, accent
/// markers).
///
/// Exposed so the list-alignment regression test can render a controlled
/// document without duplicating the scope setup.
pub fn render_markdown_document(
    ui: &mut Ui,
    t: &Tokens,
    cache: &mut CommonMarkCache,
    markdown: &str,
) {
    ui.scope(|ui| {
        apply_document_list_layout(ui, t);
        CommonMarkViewer::new().show(ui, cache, markdown);
    });
}

/// Renders one documentation page in the shared DOCUMENT layout.
///
/// The page gets theme text styles, its own vertical [`ScrollArea`] and the
/// list-marker fix. `id_salt` distinguishes the scroll state of several document
/// tabs, and `scroll` stores the offset between frames (view state, never
/// project data).
pub fn render_document_page(
    ui: &mut Ui,
    t: &Tokens,
    cache: &DocumentCache,
    scroll: &mut f32,
    id_salt: &str,
    markdown: &str,
) {
    let mut style = ui.style().as_ref().clone();
    style.visuals.override_text_color = Some(t.ink);
    style.visuals.hyperlink_color = t.accent;
    style.visuals.widgets.noninteractive.fg_stroke.color = t.ink_strong;
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(12.5));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));

    ui.scope(|ui| {
        ui.set_style(style);
        let output = ScrollArea::vertical()
            .id_salt(id_salt)
            .auto_shrink([false, false])
            .vertical_scroll_offset(*scroll)
            .show(ui, |ui| {
                let mut cache = match cache.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                render_markdown_document(ui, t, &mut cache, markdown);
            });
        *scroll = output.state.offset.y;
    });
}