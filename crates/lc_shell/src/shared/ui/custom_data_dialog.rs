//! Universal tile custom data modal dialog component.
//!
//! Renders a modal dialog allowing users to view, associate, edit and delete
//! arbitrary string data (such as JSON, XML, numbers or text) for a specific tile.

use egui::{
    vec2, Area, Color32, Context, CornerRadius, FontId, Frame, Id, Key, Margin, Order, Pos2,
    RichText, Sense, Stroke, Ui,
};
use lc_canvas::theme::Tokens;
use lc_core::id::TilesetDefId;

use super::info_banner::info_banner;

/// State for managing the custom data dialog.
#[derive(Debug, Clone, Default)]
pub struct CustomDataDialogState {
    pub is_open: bool,
    pub tileset_id: Option<TilesetDefId>,
    pub tile_id: u32,
    pub text_buffer: String,
    pub original_text: String,
    pub should_focus: bool,
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
        self.should_focus = true;
    }

    /// Closes the dialog.
    pub fn close(&mut self) {
        self.is_open = false;
        self.should_focus = false;
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

    // 1. Semi-transparent dark background scrim in Foreground order (blocks all clicks and drags underneath)
    let screen_rect = ctx.viewport_rect();
    Area::new(Id::new("custom_data_modal_scrim"))
        .order(Order::Foreground)
        .fixed_pos(screen_rect.min)
        .show(ctx, |ui| {
            let (scrim_rect, response) =
                ui.allocate_exact_size(screen_rect.size(), Sense::click_and_drag());
            ui.painter().rect_filled(
                scrim_rect,
                CornerRadius::ZERO,
                Color32::from_black_alpha(140),
            );
            if response.clicked() {
                outcome = Some(CustomDataOutcome::Cancel);
            }
        });

    // Consume background atlas keys while dialog is active
    ctx.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, Key::F);
        i.consume_key(egui::Modifiers::NONE, Key::Home);
        i.consume_key(egui::Modifiers::NONE, Key::Space);
    });

    // Handle Escape key
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.close();
        return Some(CustomDataOutcome::Cancel);
    }

    // 2. Centered modal dialog window with sharp rectangular styling
    let modal_width = 680.0_f32.min(screen_rect.width() - 32.0);
    let modal_height = 460.0_f32.min(screen_rect.height() - 32.0);

    Area::new(Id::new("custom_data_modal_window"))
        .order(Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(modal_width);
            ui.set_height(modal_height);

            let window_frame = Frame::new()
                .fill(t.panel)
                .stroke(Stroke::new(1.0, t.line_strong))
                .corner_radius(CornerRadius::ZERO)
                .inner_margin(Margin::same(0));

            window_frame.show(ui, |ui| {
                ui.set_width(modal_width);
                ui.vertical(|ui| {
                    // Header bar spanning the exact full width of the window
                    let header_h = 42.0;
                    let (header_rect, _) =
                        ui.allocate_exact_size(vec2(modal_width, header_h), Sense::hover());
                    ui.painter()
                        .rect_filled(header_rect, CornerRadius::ZERO, t.sunken);
                    ui.painter().hline(
                        header_rect.x_range(),
                        header_rect.max.y,
                        Stroke::new(1.0, t.line),
                    );

                    let title_pos = Pos2::new(header_rect.min.x + 16.0, header_rect.center().y);
                    ui.painter().text(
                        title_pos,
                        egui::Align2::LEFT_CENTER,
                        format!("TILE {} CUSTOM DATA", state.tile_id),
                        FontId::proportional(13.0),
                        t.ink_strong,
                    );

                    // Body contents
                    Frame::new()
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            // Top info callout banner (rectangular)
                            info_banner(
                                ui,
                                t,
                                "You can enter any kind of data here, which will be associated to this tile and stored in the project JSON. This data could either be numbers, text, JSON, XML etc. Basically, any tile related info you would like to pass to your game engine.",
                                preview,
                            );

                            ui.add_space(12.0);

                            // Multi-line editor with rectangular styling
                            let editor_frame = Frame::new()
                                .fill(t.canvas)
                                .stroke(t.hairline())
                                .corner_radius(CornerRadius::ZERO)
                                .inner_margin(Margin::same(8));

                            let editor_height = (modal_height - 210.0).max(140.0);

                            editor_frame.show(ui, |ui| {
                                ui.set_height(editor_height);
                                let editor_rect = ui.available_rect_before_wrap();

                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.horizontal_top(|ui| {
                                            // Line numbers gutter
                                            let count = state.text_buffer.split('\n').count().max(1);
                                            let digits = count.max(9).to_string().len().max(2);
                                            let mut gutter_str = String::with_capacity(count * (digits + 2));
                                            for idx in 1..=count {
                                                use std::fmt::Write;
                                                let _ = writeln!(gutter_str, "{:>width$}", idx, width = digits);
                                            }

                                            let gutter_resp = ui.label(
                                                RichText::new(gutter_str.trim_end())
                                                    .font(FontId::monospace(12.0))
                                                    .color(t.faint),
                                            );

                                            let divider_x = gutter_resp.rect.max.x + 6.0;

                                            // Draw vertical divider spanning the entire content height
                                            let line_top = gutter_resp.rect.min.y;
                                            let line_bottom = gutter_resp.rect.max.y.max(line_top + editor_height);
                                            ui.painter().vline(
                                                divider_x,
                                                line_top..=line_bottom,
                                                t.hairline(),
                                            );

                                            // Also draw vertical divider across the full visible editor frame
                                            // so it is permanently visible regardless of scrolling
                                            ui.painter().vline(
                                                divider_x,
                                                editor_rect.min.y..=editor_rect.max.y,
                                                t.hairline(),
                                            );

                                            ui.add_space(14.0);

                                            // Actual editable text buffer
                                            let text_edit =
                                                egui::TextEdit::multiline(&mut state.text_buffer)
                                                    .id_salt("custom_data_multiline_edit")
                                                    .font(FontId::monospace(12.0))
                                                    .text_color(t.ink)
                                                    .desired_width(f32::INFINITY)
                                                    .frame(Frame::new());
                                            let edit_resp = ui.add(text_edit);
                                            if state.should_focus {
                                                edit_resp.request_focus();
                                                state.should_focus = false;
                                            }
                                            if !edit_resp.has_focus() {
                                                ui.ctx().input_mut(|i| {
                                                    i.consume_key(egui::Modifiers::NONE, Key::Enter);
                                                });
                                            }
                                        });
                                    });
                            });

                            ui.add_space(14.0);

                            // Bottom action buttons matching standard Fleiya desktop dialogs
                            ui.horizontal(|ui| {
                                let has_data = !state.original_text.is_empty()
                                    || !state.text_buffer.is_empty();
                                if has_data {
                                    let del_btn = egui::Button::new(
                                        RichText::new("Delete")
                                            .size(11.5)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(220, 38, 38))
                                    .corner_radius(CornerRadius::ZERO)
                                    .min_size(vec2(64.0, 26.0));

                                    if ui
                                        .add(del_btn)
                                        .on_hover_text("Delete custom data for this tile")
                                        .clicked()
                                    {
                                        state.text_buffer.clear();
                                        outcome = Some(CustomDataOutcome::Delete);
                                    }
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let save_btn = egui::Button::new(
                                            RichText::new("Save")
                                                .size(11.5)
                                                .strong()
                                                .color(t.invert_fg),
                                        )
                                        .fill(t.accent)
                                        .corner_radius(CornerRadius::ZERO)
                                        .min_size(vec2(76.0, 26.0));

                                        if ui.add(save_btn).clicked() {
                                            outcome = Some(CustomDataOutcome::Save(
                                                state.text_buffer.clone(),
                                            ));
                                        }

                                        ui.add_space(8.0);

                                        let cancel_btn = egui::Button::new(
                                            RichText::new("Cancel")
                                                .size(11.5)
                                                .color(t.ink),
                                        )
                                        .fill(t.sunken)
                                        .stroke(t.hairline())
                                        .corner_radius(CornerRadius::ZERO)
                                        .min_size(vec2(68.0, 26.0));

                                        if ui.add(cancel_btn).clicked() {
                                            outcome = Some(CustomDataOutcome::Cancel);
                                        }
                                    },
                                );
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
