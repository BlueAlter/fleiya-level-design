//! Document view for editing and inspecting `TilesetDef` templates.

use egui::{
    vec2, Align2, Color32, CornerRadius, DragValue, FontFamily, FontId, Frame, Margin, Pos2, Rect,
    RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};
use glam::UVec2;
use lc_core::command::DocKind;
use lc_core::defs::TilesetDef;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::view::DocumentView;

use crate::commands::UpdateTilesetDefCommand;
use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::{hud, meta_row, page, page_header};

/// Document tab view for configuring an individual `TilesetDef`.
pub struct TilesetDocumentView {
    tileset_id: TilesetDefId,
    preview_zoom: f32,
    preview_pan: Vec2,
    hovered_tile: Option<u32>,

    // Cached edit buffers
    name_buf: String,
    path_buf: String,
    tile_w: u32,
    tile_h: u32,
    spacing: u32,
    padding: u32,
    buffers_synced: bool,
}

impl TilesetDocumentView {
    /// Creates a new document view for the specified tileset.
    pub fn new(tileset_id: TilesetDefId) -> Self {
        Self {
            tileset_id,
            preview_zoom: 2.0,
            preview_pan: Vec2::ZERO,
            hovered_tile: None,

            name_buf: String::new(),
            path_buf: String::new(),
            tile_w: 16,
            tile_h: 16,
            spacing: 0,
            padding: 0,
            buffers_synced: false,
        }
    }

    /// Returns the tileset ID associated with this document view.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Syncs local input buffers from project definition.
    fn sync_buffers(&mut self, def: &TilesetDef) {
        if !self.buffers_synced {
            self.name_buf = def.name.clone();
            self.path_buf = def.path.clone();
            self.tile_w = def.tile_size.x;
            self.tile_h = def.tile_size.y;
            self.spacing = def.spacing;
            self.padding = def.padding;
            self.buffers_synced = true;
        }
    }

    /// Commits local input buffers to a command if differences exist.
    fn commit_changes(&mut self, def: &TilesetDef, cx: &mut EditorCtx) {
        let updated = TilesetDef {
            id: self.tileset_id,
            name: self.name_buf.trim().to_string(),
            path: self.path_buf.trim().to_string(),
            tile_size: UVec2::new(self.tile_w.max(1), self.tile_h.max(1)),
            spacing: self.spacing,
            padding: self.padding,
        };

        if *def != updated {
            cx.exec(UpdateTilesetDefCommand::new(updated));
        }
    }
}

impl DocumentView for TilesetDocumentView {
    fn title(&self, p: &Project) -> String {
        p.defs
            .tileset(self.tileset_id)
            .map(|t| format!("Tileset: {}", t.name))
            .unwrap_or_else(|| "Tileset (deleted)".to_string())
    }

    fn is_valid(&self, p: &Project) -> bool {
        p.defs.tileset(self.tileset_id).is_some()
    }

    fn doc_kind(&self) -> DocKind {
        DocKind::TilesetDef(self.tileset_id)
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let Some(def) = cx.project().defs.tileset(self.tileset_id).cloned() else {
            ui.label("Tileset definition does not exist.");
            return;
        };

        self.sync_buffers(&def);

        let t = Tokens::get(ui.ctx());

        page(ui, |ui| {
            page_header(
                ui,
                "TILESET",
                &def.name,
                "Configure sprite sheet texture, tile slice dimensions, spacing, and border padding.",
            );

            ui.add_space(16.0);

            // 1. General Properties Section
            hud(ui, "PROPERTIES", t.accent_ink);
            ui.add_space(8.0);

            let mut commit_needed = false;

            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::ZERO)
                .show(ui, |ui| {
                    egui::Grid::new("tileset_props_grid")
                        .num_columns(2)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Name").color(t.ink_strong));
                            let name_resp = ui
                                .add(TextEdit::singleline(&mut self.name_buf).desired_width(260.0));
                            if name_resp.lost_focus() {
                                commit_needed = true;
                            }
                            ui.end_row();

                            ui.label(RichText::new("Asset Path").color(t.ink_strong));
                            let path_resp = ui.add(
                                TextEdit::singleline(&mut self.path_buf)
                                    .desired_width(260.0)
                                    .hint_text("e.g. tiles/dungeon.png"),
                            );
                            if path_resp.lost_focus() {
                                commit_needed = true;
                            }
                            ui.end_row();

                            ui.label(RichText::new("Tile Size").color(t.ink_strong));
                            ui.horizontal(|ui| {
                                let w_resp = ui.add(
                                    DragValue::new(&mut self.tile_w)
                                        .range(1..=1024)
                                        .prefix("W: ")
                                        .suffix(" px"),
                                );
                                let h_resp = ui.add(
                                    DragValue::new(&mut self.tile_h)
                                        .range(1..=1024)
                                        .prefix("H: ")
                                        .suffix(" px"),
                                );
                                if w_resp.changed() || h_resp.changed() {
                                    commit_needed = true;
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Spacing").color(t.ink_strong));
                            let sp_resp = ui.add(
                                DragValue::new(&mut self.spacing)
                                    .range(0..=256)
                                    .suffix(" px"),
                            );
                            if sp_resp.changed() {
                                commit_needed = true;
                            }
                            ui.end_row();

                            ui.label(RichText::new("Padding").color(t.ink_strong));
                            let pad_resp = ui.add(
                                DragValue::new(&mut self.padding)
                                    .range(0..=256)
                                    .suffix(" px"),
                            );
                            if pad_resp.changed() {
                                commit_needed = true;
                            }
                            ui.end_row();
                        });
                });

            if commit_needed {
                self.commit_changes(&def, cx);
            }

            ui.add_space(16.0);

            // Metadata summary rows
            meta_row(ui, "TILESET ID", &format!("#{}", self.tileset_id.0));
            meta_row(
                ui,
                "SLICE SIZE",
                &format!("{} x {} px", def.tile_size.x, def.tile_size.y),
            );

            ui.add_space(20.0);

            // 2. Texture & Grid Slicing Preview
            ui.horizontal(|ui| {
                hud(ui, "TEXTURE & SLICING PREVIEW", t.accent_ink);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("4x").clicked() {
                        self.preview_zoom = 4.0;
                    }
                    if ui.button("2x").clicked() {
                        self.preview_zoom = 2.0;
                    }
                    if ui.button("1x").clicked() {
                        self.preview_zoom = 1.0;
                    }
                    if ui.button("Reset Pan").clicked() {
                        self.preview_pan = Vec2::ZERO;
                    }
                    ui.label(format!("{:.0}%", self.preview_zoom * 100.0));
                });
            });

            ui.add_space(8.0);

            let preview_height = 360.0;
            let (preview_rect, response) = ui.allocate_exact_size(
                vec2(ui.available_width(), preview_height),
                Sense::click_and_drag(),
            );

            // Pan handling in preview
            if response.dragged_by(egui::PointerButton::Middle)
                || (response.dragged_by(egui::PointerButton::Primary)
                    && ui.input(|i| i.key_down(egui::Key::Space)))
            {
                self.preview_pan += response.drag_delta();
            }

            // Zoom handling on hover
            if response.hovered() {
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    let factor = if scroll > 0.0 { 1.15 } else { 1.0 / 1.15 };
                    self.preview_zoom = (self.preview_zoom * factor).clamp(0.25, 16.0);
                }
            }

            let painter = ui.painter().with_clip_rect(preview_rect);
            painter.rect_filled(preview_rect, CornerRadius::ZERO, t.canvas);
            draw_checkerboard(&painter, preview_rect, 16.0, &t);

            // Load texture via AssetService
            match cx.assets().load_texture(ui.ctx(), &def.path) {
                Ok(texture) => {
                    let img_size = texture.size_vec2();
                    let scaled_size = img_size * self.preview_zoom;
                    let tex_origin = preview_rect.min + self.preview_pan + vec2(16.0, 16.0);
                    let tex_rect = Rect::from_min_size(tex_origin, scaled_size);

                    // Draw image
                    painter.image(
                        texture.id(),
                        tex_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    // Compute slicing columns & rows
                    let tile_w = def.tile_size.x.max(1);
                    let tile_h = def.tile_size.y.max(1);
                    let padding = def.padding;
                    let spacing = def.spacing;

                    let img_w = img_size.x as u32;
                    let img_h = img_size.y as u32;
                    let eff_w = img_w.saturating_sub(padding * 2);
                    let eff_h = img_h.saturating_sub(padding * 2);
                    let stride_x = tile_w + spacing;
                    let stride_y = tile_h + spacing;

                    let cols = (eff_w + spacing).checked_div(stride_x).unwrap_or(0);
                    let rows = (eff_h + spacing).checked_div(stride_y).unwrap_or(0);

                    // Draw grid overlay lines for tile cells
                    let grid_stroke = Stroke::new(1.0, t.grid_major);
                    let zoom = self.preview_zoom;

                    for r in 0..rows {
                        for c in 0..cols {
                            let cell_x =
                                tex_origin.x + (padding as f32 + (c * stride_x) as f32) * zoom;
                            let cell_y =
                                tex_origin.y + (padding as f32 + (r * stride_y) as f32) * zoom;
                            let cell_rect = Rect::from_min_size(
                                Pos2::new(cell_x, cell_y),
                                vec2(tile_w as f32 * zoom, tile_h as f32 * zoom),
                            );

                            painter.rect_stroke(
                                cell_rect,
                                CornerRadius::ZERO,
                                grid_stroke,
                                StrokeKind::Inside,
                            );
                        }
                    }

                    // Hover tile calculation
                    self.hovered_tile = None;
                    if let Some(mouse_pos) = response.hover_pos() {
                        if tex_rect.contains(mouse_pos) && cols > 0 && rows > 0 {
                            let rel_x = (mouse_pos.x - tex_origin.x) / zoom;
                            let rel_y = (mouse_pos.y - tex_origin.y) / zoom;

                            if rel_x >= padding as f32 && rel_y >= padding as f32 {
                                let c = ((rel_x - padding as f32) / stride_x as f32).floor() as u32;
                                let r = ((rel_y - padding as f32) / stride_y as f32).floor() as u32;

                                if c < cols && r < rows {
                                    let tile_id = r * cols + c;
                                    self.hovered_tile = Some(tile_id);

                                    // Highlight hovered cell
                                    let cell_x = tex_origin.x
                                        + (padding as f32 + (c * stride_x) as f32) * zoom;
                                    let cell_y = tex_origin.y
                                        + (padding as f32 + (r * stride_y) as f32) * zoom;
                                    let cell_rect = Rect::from_min_size(
                                        Pos2::new(cell_x, cell_y),
                                        vec2(tile_w as f32 * zoom, tile_h as f32 * zoom),
                                    );

                                    painter.rect_stroke(
                                        cell_rect,
                                        CornerRadius::ZERO,
                                        Stroke::new(2.0, t.accent),
                                        StrokeKind::Inside,
                                    );
                                }
                            }
                        }
                    }

                    // Border around texture
                    painter.rect_stroke(
                        tex_rect,
                        CornerRadius::ZERO,
                        Stroke::new(1.0, t.accent_ink),
                        StrokeKind::Inside,
                    );

                    // Summary footer below preview
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let total_tiles = cols * rows;
                        hud(
                            ui,
                            &format!(
                                "IMAGE: {:.0}x{:.0} px  |  GRID: {} cols x {} rows ({} tiles)",
                                img_size.x, img_size.y, cols, rows, total_tiles
                            ),
                            t.faint,
                        );

                        if let Some(hovered) = self.hovered_tile {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    hud(ui, &format!("HOVERED: Tile #{}", hovered), t.accent_ink);
                                },
                            );
                        }
                    });
                }
                Err(err) => {
                    // Friendly placeholder when texture is missing or unloaded
                    let text = format!("Texture asset not loaded: {}\nPath: \"{}\"", err, def.path);
                    painter.text(
                        preview_rect.center(),
                        Align2::CENTER_CENTER,
                        text,
                        FontId::new(14.0, FontFamily::Proportional),
                        t.muted,
                    );
                }
            }

            // Outline around the preview rect
            painter.rect_stroke(
                preview_rect,
                CornerRadius::ZERO,
                t.hairline(),
                StrokeKind::Inside,
            );
        });
    }
}
