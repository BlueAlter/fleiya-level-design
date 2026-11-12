//! Console panel UI implementing `PanelView`.

use super::ring_buffer::{ConsoleFilter, SharedRingBuffer};
use egui::{FontId, RichText, ScrollArea, TextEdit, Ui};
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, hline, hud, hud_elided, tag, toggle_chip, ButtonKind};
use lc_ui_api::{EditorCtx, EditorEvent, PanelId, PanelView};

/// Godot-style console panel displaying real-time structured logs.
pub struct ConsolePanel {
    buffer: SharedRingBuffer,
    filter: ConsoleFilter,
    auto_scroll: bool,
}

impl ConsolePanel {
    /// Creates a new console panel attached to the given ring buffer.
    pub fn new(buffer: SharedRingBuffer) -> Self {
        Self {
            buffer,
            filter: ConsoleFilter::default(),
            auto_scroll: true,
        }
    }

    /// Access to the underlying shared ring buffer.
    pub fn buffer(&self) -> &SharedRingBuffer {
        &self.buffer
    }
}

impl PanelView for ConsolePanel {
    fn id(&self) -> PanelId {
        PanelId::new("console")
    }

    fn title(&self) -> &str {
        "Console"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());
        let (err_c, warn_c, info_c, debug_c, trace_c) = self.buffer.counts_by_level();

        // 1. Toolbar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            if toggle_chip(ui, &format!("ERR {err_c}"), self.filter.show_error, t.bad).clicked() {
                self.filter.show_error = !self.filter.show_error;
            }
            if toggle_chip(ui, &format!("WARN {warn_c}"), self.filter.show_warn, t.warn).clicked() {
                self.filter.show_warn = !self.filter.show_warn;
            }
            if toggle_chip(
                ui,
                &format!("INFO {info_c}"),
                self.filter.show_info,
                t.accent_ink,
            )
            .clicked()
            {
                self.filter.show_info = !self.filter.show_info;
            }
            if toggle_chip(
                ui,
                &format!("DBG {debug_c}"),
                self.filter.show_debug,
                t.muted,
            )
            .clicked()
            {
                self.filter.show_debug = !self.filter.show_debug;
            }
            if toggle_chip(
                ui,
                &format!("TRACE {trace_c}"),
                self.filter.show_trace,
                t.faint,
            )
            .clicked()
            {
                self.filter.show_trace = !self.filter.show_trace;
            }

            ui.add_space(2.0);

            // Search query filter
            ui.add(
                TextEdit::singleline(&mut self.filter.search_query)
                    .hint_text("SEARCH")
                    .desired_width(88.0)
                    .font(FontId::new(11.0, theme::mono_for(ui.ctx())))
                    .text_color(t.ink)
                    .margin(egui::Margin::symmetric(6, 4)),
            );

            // Collapse identical toggle
            if toggle_chip(ui, "DEDUPE", self.filter.collapse_duplicates, t.accent_ink).clicked() {
                self.filter.collapse_duplicates = !self.filter.collapse_duplicates;
            }

            // Auto-scroll toggle
            if toggle_chip(ui, "FOLLOW", self.auto_scroll, t.accent_ink).clicked() {
                self.auto_scroll = !self.auto_scroll;
            }

            ui.add_space(2.0);

            if button(ui, "CLEAR", ButtonKind::Ghost, true).clicked() {
                self.buffer.clear();
            }

            if button(ui, "COPY", ButtonKind::Ghost, true).clicked() {
                let items = self.buffer.filtered_entries(&self.filter);
                let mut text = String::new();
                for item in items {
                    let repeat_suffix = if item.count > 1 {
                        format!(" (x{})", item.count)
                    } else {
                        String::new()
                    };
                    text.push_str(&format!(
                        "[{}] [{}] [{}] {}{}\n",
                        item.entry.timestamp,
                        item.entry.level.as_str(),
                        item.entry.target,
                        item.entry.message,
                        repeat_suffix
                    ));
                }
                ui.ctx().copy_text(text);
                cx.status("Copied console logs to clipboard");
            }
        });
        hline(ui);

        // 2. Log message listing
        let entries = self.buffer.filtered_entries(&self.filter);

        ScrollArea::vertical()
            .stick_to_bottom(self.auto_scroll)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;

                if entries.is_empty() {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        hud(ui, "NO LOG MESSAGES MATCH ACTIVE FILTERS", t.faint);
                    });
                    return;
                }

                for item in entries {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        hud(ui, &item.entry.timestamp, t.faint);

                        let color = item.entry.level.color(&t);
                        tag(
                            ui,
                            item.entry.level.as_str(),
                            theme::tint(color, 0.14),
                            color,
                        );

                        hud_elided(ui, &format!("[{}]", item.entry.target), t.faint, 150.0);

                        ui.add(
                            egui::Label::new(
                                RichText::new(&item.entry.message)
                                    .font(FontId::new(11.5, theme::medium_for(ui.ctx())))
                                    .color(t.ink),
                            )
                            .truncate(),
                        );

                        if item.count > 1 {
                            tag(
                                ui,
                                &format!("x{}", item.count),
                                theme::tint(t.warn, 0.14),
                                t.warn,
                            );
                        }

                        // Click-to-focus DocKind link if present
                        if let Some(doc_kind) = item.entry.doc_target {
                            if button(ui, &format!("-> {doc_kind:?}"), ButtonKind::Ghost, true)
                                .clicked()
                            {
                                cx.emit(EditorEvent::OpenDocument(doc_kind));
                                cx.status(format!("Navigated to {doc_kind:?}"));
                            }
                        }
                    });
                }
            });
    }
}
