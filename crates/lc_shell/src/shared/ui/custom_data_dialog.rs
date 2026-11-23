//! Universal tile custom data modal dialog component.
//!
//! Renders a modal dialog allowing users to view, associate, edit and delete
//! arbitrary string data (such as JSON, XML, numbers or text) for a specific tile.

use egui::{
    vec2, Area, Color32, Context, CornerRadius, FontId, Frame, Id, Key, Margin, Order, RichText,
    Sense, Stroke, Ui,
};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::id::TilesetDefId;

use super::icon_button::{icon_button, IconButtonStyle};
use super::info_banner::info_banner;

/// State for managing the custom data dialog.
#[derive(Debug, Clone, Default)]
pub struct CustomDataDialogState {
    pub is_open: bool,
    pub tileset_id: Option<TilesetDefId>,
    pub tile_id: u32,
    pub text_buffer: String,
    pub original_text: String,
}

impl CustomDataDialogState {
    /// Opens the dialog for a specific tile and populates it with the existing data.
    pub fn open(&mut self, tileset_id: TilesetDefId, tile_id: u32, current_data: Option<&str>) {
        self.is_open = true;
        self.tileset_id = Some(tileset_id);
        self.tile_id = tile_id;
        let text = current_data.unwrap_or("").to_string();
        self.text_buffer = text.clone();
        self.original_text = text;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Action resulting from interacting with the custom data modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomDataOutcome {
    /// Save custom data and close dialog.
    Save(String),
    /// Delete/clear custom data and close dialog.
    Delete,
    /// Cancel and close dialog without saving.
    Cancel,
}

/// Renders the custom data modal dialog if open.
pub fn render_custom_data_dialog<F>(
    ctx: &Context,
    t: &Tokens,
    state: &mut CustomDataDialogState,
    preview: Option<F>,
) -> Option<CustomDataOutcome>
where
    F: FnOnce(&mut Ui),
{
    if !state.is_open {
        return None;
    }

    let mut outcome = None;

    // 1. Semi-transparent dark background scrim
    let screen_rect = ctx.viewport_rect();
    Area::new(Id::new("custom_data_modal_scrim"))
        .order(Order::Middle)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let (scrim_rect, response) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
            ui.painter().rect_filled(
                scrim_rect,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
            if response.clicked() {
                outcome = Some(CustomDataOutcome::Save(state.text_buffer.clone()));
            }
        });

    // Handle Escape key
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.close();
        return Some(CustomDataOutcome::Cancel);
    }

    // 2. Centered modal dialog window
    let modal_width = 680.0_f32.min(screen_rect.width() - 32.0);
    let modal_height = 460.0_f32.min(screen_rect.height() - 32.0);

    Area::new(Id::new("custom_data_modal_window"))
        .order(Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(modal_width);
            ui.set_height(modal_height);

            let window_frame = Frame::new()
                .fill(Color32::from_rgb(26, 36, 48))
                .stroke(Stroke::new(1.0, Color32::from_rgb(55, 75, 95)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(0));

            window_frame.show(ui, |ui| {
                ui.vertical(|ui| {
                    // Header bar
                    let header_frame = Frame::new()
                        .fill(Color32::from_rgb(32, 45, 60))
                        .inner_margin(Margin::symmetric(16, 12));
                    header_frame.show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("TILE {} CUSTOM DATA", state.tile_id))
                                .size(13.5)
                                .strong()
                                .color(Color32::WHITE),
                        );
                    });

                    // Body contents
                    Frame::new()
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            // Top info callout banner
                            info_banner(
                                ui,
                                t,
                                "You can enter any kind of data here, which will be associated to this tile and stored in the project JSON. This data could either be numbers, text, JSON, XML etc. Basically, any tile related info you would like to pass to your game engine.",
                                preview,
                            );

                            ui.add_space(12.0);

                            // Multi-line editor with line numbers
                            let editor_frame = Frame::new()
                                .fill(Color32::from_rgb(14, 22, 30))
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::same(10));

                            editor_frame.show(ui, |ui| {
                                let editor_height = (modal_height - 210.0).max(140.0);
                                ui.set_height(editor_height);

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                            // Line numbers gutter
                                            let count = state.text_buffer.lines().count().max(1);
                                            let mut gutter_str = String::with_capacity(count * 4);
                                            for idx in 1..=count {
                                                gutter_str.push_str(&format!("{}\n", idx));
                                            }
                                            ui.label(
                                                RichText::new(gutter_str.trim_end())
                                                    .font(FontId::monospace(12.5))
                                                    .color(Color32::from_rgb(70, 95, 120)),
                                            );

                                            ui.add_space(6.0);
                                            ui.painter().vline(
                                                ui.cursor().min.x,
                                                ui.cursor().min.y
                                                    ..=(ui.cursor().min.y + editor_height),
                                                Stroke::new(1.0, Color32::from_rgb(35, 50, 65)),
                                            );
                                            ui.add_space(8.0);

                                            // Actual editable text buffer
                                            let text_edit =
                                                egui::TextEdit::multiline(&mut state.text_buffer)
                                                    .font(FontId::monospace(12.5))
                                                    .text_color(Color32::from_rgb(220, 235, 245))
                                                    .desired_width(f32::INFINITY)
                                                    .frame(Frame::new());
                                            ui.add(text_edit);
                                        });
                                    });
                            });

                            ui.add_space(14.0);

                            // Bottom action buttons
                            ui.horizontal(|ui| {
                                let close_btn_width = ui.available_width() - 44.0;
                                let close_btn = egui::Button::new(
                                    RichText::new("CLOSE")
                                        .size(12.5)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(220, 75, 12))
                                .corner_radius(CornerRadius::same(3))
                                .min_size(vec2(close_btn_width.max(100.0), 34.0));

                                if ui.add(close_btn).clicked() {
                                    outcome =
                                        Some(CustomDataOutcome::Save(state.text_buffer.clone()));
                                }

                                ui.add_space(10.0);

                                // Delete / Trash button
                                let del_btn =
                                    icon_button(ui, Icon::Delete, 32.0, IconButtonStyle::danger())
                                        .on_hover_text("Delete custom data for this tile");
                                if del_btn.clicked() {
                                    state.text_buffer.clear();
                                    outcome = Some(CustomDataOutcome::Delete);
                                }
                            });
                        });
                });
            });
        });

    if outcome.is_some() {
        state.close();
    }

    outcome
}
