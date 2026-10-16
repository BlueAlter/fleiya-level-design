//! Console panel UI implementing `PanelView`.

use super::ring_buffer::{ConsoleFilter, LogLevel, SharedRingBuffer};
use egui::{Button, RichText, ScrollArea, TextEdit, Ui};
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
        let (err_c, warn_c, info_c, debug_c, trace_c) = self.buffer.counts_by_level();

        // 1. Toolbar
        ui.horizontal_wrapped(|ui| {
            // Level toggles
            let err_btn = ui.selectable_label(
                self.filter.show_error,
                RichText::new(format!("Error ({err_c})")).color(LogLevel::Error.color()),
            );
            if err_btn.clicked() {
                self.filter.show_error = !self.filter.show_error;
            }

            let warn_btn = ui.selectable_label(
                self.filter.show_warn,
                RichText::new(format!("Warn ({warn_c})")).color(LogLevel::Warn.color()),
            );
            if warn_btn.clicked() {
                self.filter.show_warn = !self.filter.show_warn;
            }

            let info_btn = ui.selectable_label(
                self.filter.show_info,
                RichText::new(format!("Info ({info_c})")).color(LogLevel::Info.color()),
            );
            if info_btn.clicked() {
                self.filter.show_info = !self.filter.show_info;
            }

            let debug_btn = ui.selectable_label(
                self.filter.show_debug,
                RichText::new(format!("Debug ({debug_c})")).color(LogLevel::Debug.color()),
            );
            if debug_btn.clicked() {
                self.filter.show_debug = !self.filter.show_debug;
            }

            let trace_btn = ui.selectable_label(
                self.filter.show_trace,
                RichText::new(format!("Trace ({trace_c})")).color(LogLevel::Trace.color()),
            );
            if trace_btn.clicked() {
                self.filter.show_trace = !self.filter.show_trace;
            }

            ui.separator();

            // Search query filter
            ui.add(
                TextEdit::singleline(&mut self.filter.search_query)
                    .hint_text("Search logs...")
                    .desired_width(140.0),
            );

            // Collapse identical toggle
            ui.checkbox(&mut self.filter.collapse_duplicates, "Collapse");

            // Auto-scroll toggle
            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");

            ui.separator();

            // Clear button
            if ui.button("Clear").clicked() {
                self.buffer.clear();
            }

            // Copy button
            if ui.button("Copy").clicked() {
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

        ui.separator();

        // 2. Log message listing
        let entries = self.buffer.filtered_entries(&self.filter);

        ScrollArea::vertical()
            .stick_to_bottom(self.auto_scroll)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if entries.is_empty() {
                    ui.label(RichText::new("No log messages match active filters.").weak());
                    return;
                }

                for item in entries {
                    ui.horizontal(|ui| {
                        // Timestamp
                        ui.label(RichText::new(&item.entry.timestamp).weak().monospace());

                        // Level badge
                        ui.colored_label(
                            item.entry.level.color(),
                            format!("[{}]", item.entry.level.as_str()),
                        );

                        // Target
                        ui.label(RichText::new(format!("[{}]", item.entry.target)).weak());

                        // Message
                        ui.label(&item.entry.message);

                        // Duplicate badge
                        if item.count > 1 {
                            ui.label(
                                RichText::new(format!("(x{})", item.count))
                                    .color(egui::Color32::from_rgb(255, 204, 0))
                                    .strong(),
                            );
                        }

                        // Click-to-focus DocKind link if present
                        if let Some(doc_kind) = item.entry.doc_target {
                            let btn = ui.add(
                                Button::new(RichText::new(format!("-> {doc_kind:?}")).small())
                                    .small(),
                            );
                            if btn.clicked() {
                                cx.emit(EditorEvent::OpenDocument(doc_kind));
                                cx.status(format!("Navigated to {doc_kind:?}"));
                            }
                        }
                    });
                }
            });
    }
}
