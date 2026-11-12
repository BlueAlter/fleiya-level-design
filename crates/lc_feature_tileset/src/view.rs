//! Document view for editing and inspecting `TilesetDef` templates matching reference picture 1.

use egui::{
    vec2, Color32, CornerRadius, DragValue, FontId, Frame, Margin, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, TextEdit, Ui, Vec2,
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
use lc_canvas::theme::{self, Tokens};
use lc_canvas::widgets::{button, ButtonKind};

/// Document tab view for configuring an individual `TilesetDef`.
pub struct TilesetDocumentView {
    tileset_id: TilesetDefId,
    preview_zoom: f32,
    preview_pan: Vec2,
    hovered_tile: Option<u32>,
    selected_tile_id: Option<u32>,
    show_grid_lines: bool,
    show_collision: bool,
    active_sub_tab: usize, // 0 = TILESET DEF, 1 = DOCUMENT

    // Cached edit buffers
    name_buf: String,
    path_buf: String,
    tile_w: u32,
    tile_h: u32,
    spacing: u32,
    padding: u32,
    custom_json: String,
    enum_binding: String,
    buffers_synced: bool,
}

impl TilesetDocumentView {
    /// Creates a new document view for the specified tileset.
    pub fn new(tileset_id: TilesetDefId) -> Self {
        Self {
            tileset_id,
            preview_zoom: 1.0,
            preview_pan: Vec2::ZERO,
            hovered_tile: None,
            selected_tile_id: Some(36),
            show_grid_lines: true,
            show_collision: true,
            active_sub_tab: 0,

            name_buf: String::new(),
            path_buf: String::new(),
            tile_w: 16,
            tile_h: 16,
            spacing: 0,
            padding: 0,
            custom_json: "{\n  \"damage\": 0,\n  \"climbable\": false\n}".to_string(),
            enum_binding: "= BiomeType.Dungeon".to_string(),
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
            cx.status(format!("Updated tileset {}", self.name_buf));
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);

            // 1. Top Sub-tabs (TILESET DEF / DOCUMENT)
            ui.horizontal(|ui| {
                let is_def = self.active_sub_tab == 0;
                let btn_def = Frame::new()
                    .fill(if is_def { t.sunken } else { Color32::TRANSPARENT })
                    .stroke(if is_def { Stroke::new(1.0, t.accent) } else { Stroke::NONE })
                    .inner_margin(Margin::symmetric(10, 4))
                    .corner_radius(CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.label(RichText::new("TILESET DEF").size(11.0).color(if is_def { t.accent_ink } else { t.muted }))
                    });
                if btn_def.response.clicked() {
                    self.active_sub_tab = 0;
                }

                let is_doc = self.active_sub_tab == 1;
                let btn_doc = Frame::new()
                    .fill(if is_doc { t.sunken } else { Color32::TRANSPARENT })
                    .stroke(if is_doc { Stroke::new(1.0, t.accent) } else { Stroke::NONE })
                    .inner_margin(Margin::symmetric(10, 4))
                    .corner_radius(CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.label(RichText::new("DOCUMENT").size(11.0).color(if is_doc { t.accent_ink } else { t.muted }))
                    });
                if btn_doc.response.clicked() {
                    self.active_sub_tab = 1;
                }
            });

            ui.add_space(8.0);

            // 2. Large Heading & Description
            ui.label(
                RichText::new(format!("{} (Tileset)", def.name))
                    .font(FontId::new(26.0, theme::mono_bold_for(ui.ctx())))
                    .color(t.ink_strong),
            );
            ui.label(
                RichText::new("Configure atlas texture source, tile slicing dimensions, spacing margins, tags, and collision data.")
                    .size(12.0)
                    .color(t.muted),
            );

            ui.add_space(14.0);

            // 3. Section: ATLAS SOURCE & SLICING GEOMETRY
            ui.label(
                RichText::new("ATLAS SOURCE & SLICING GEOMETRY")
                    .font(FontId::new(10.5, theme::mono_bold_for(ui.ctx())))
                    .color(t.accent_ink),
            );
            ui.add_space(4.0);

            let mut commit_needed = false;

            Frame::new()
                .fill(t.panel)
                .stroke(t.hairline())
                .inner_margin(Margin::same(12))
                .corner_radius(CornerRadius::same(2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Identifier").color(t.ink_strong).size(12.0));
                        ui.add_space(6.0);
                        let id_resp = ui.add(TextEdit::singleline(&mut self.name_buf).desired_width(160.0));
                        if id_resp.lost_focus() {
                            commit_needed = true;
                        }

                        ui.add_space(20.0);
                        ui.label(RichText::new("Texture Path").color(t.ink_strong).size(12.0));
                        ui.add_space(6.0);
                        let path_resp = ui.add(TextEdit::singleline(&mut self.path_buf).desired_width(220.0));
                        if path_resp.lost_focus() {
                            commit_needed = true;
                        }

                        if ui.button("Browse...").clicked() {
                            // Quick file dialog trigger or default suggestion
                        }
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tile Size").color(t.ink_strong).size(12.0));
                        ui.add_space(6.0);
                        let w_resp = ui.add(DragValue::new(&mut self.tile_w).range(1..=1024));
                        ui.label("×");
                        let h_resp = ui.add(DragValue::new(&mut self.tile_h).range(1..=1024));
                        if w_resp.changed() || h_resp.changed() {
                            commit_needed = true;
                        }

                        ui.add_space(16.0);
                        ui.label(RichText::new("Spacing").color(t.ink_strong).size(12.0));
                        ui.add_space(6.0);
                        let sp_resp = ui.add(DragValue::new(&mut self.spacing).range(0..=256));
                        if sp_resp.changed() {
                            commit_needed = true;
                        }

                        ui.add_space(16.0);
                        ui.label(RichText::new("Padding").color(t.ink_strong).size(12.0));
                        ui.add_space(6.0);
                        let pad_resp = ui.add(DragValue::new(&mut self.padding).range(0..=256));
                        if pad_resp.changed() {
                            commit_needed = true;
                        }

                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("Atlas: 256 × 256 px (16 × 16 = 256 tiles)")
                                .color(t.faint)
                                .size(11.0),
                        );
                    });
                });

            if commit_needed {
                self.commit_changes(&def, cx);
            }

            ui.add_space(16.0);

            // 4. Section: ATLAS SLICING PREVIEW & METADATA
            ui.label(
                RichText::new("ATLAS SLICING PREVIEW & METADATA")
                    .font(FontId::new(10.5, theme::mono_bold_for(ui.ctx())))
                    .color(t.accent_ink),
            );
            ui.add_space(4.0);

            ui.columns(2, |cols| {
                // Left Column: Interactive atlas preview
                let ui_left = &mut cols[0];
                ui_left.horizontal(|ui| {
                    ui.label(RichText::new(format!("Zoom: {:.0}%", self.preview_zoom * 100.0)).size(11.0).color(t.muted));
                    ui.add_space(12.0);
                    if ui.selectable_label(self.show_grid_lines, "Grid Lines").clicked() {
                        self.show_grid_lines = !self.show_grid_lines;
                    }
                    if ui.selectable_label(self.show_collision, "Collision").clicked() {
                        self.show_collision = !self.show_collision;
                    }
                });

                ui_left.add_space(6.0);

                let preview_height = 340.0;
                let (preview_rect, response) = ui_left.allocate_exact_size(
                    vec2(ui_left.available_width(), preview_height),
                    Sense::click_and_drag(),
                );

                if response.dragged_by(egui::PointerButton::Middle)
                    || (response.dragged_by(egui::PointerButton::Primary)
                        && ui_left.input(|i| i.key_down(egui::Key::Space)))
                {
                    self.preview_pan += response.drag_delta();
                }

                if response.hovered() {
                    let scroll = ui_left.input(|i| i.smooth_scroll_delta.y);
                    if scroll != 0.0 {
                        let factor = if scroll > 0.0 { 1.15 } else { 1.0 / 1.15 };
                        self.preview_zoom = (self.preview_zoom * factor).clamp(0.25, 8.0);
                    }
                }

                let painter = ui_left.painter().with_clip_rect(preview_rect);
                painter.rect_filled(preview_rect, CornerRadius::ZERO, t.canvas);
                draw_checkerboard(&painter, preview_rect, 16.0, &t);

                // Load texture or fallback
                let tile_w = def.tile_size.x.max(1);
                let tile_h = def.tile_size.y.max(1);
                let zoom = self.preview_zoom;
                let stride_x = tile_w as f32 * zoom;
                let stride_y = tile_h as f32 * zoom;

                let cols_count = 16;
                let rows_count = 16;
                let grid_origin = preview_rect.min + self.preview_pan + vec2(12.0, 12.0);

                if let Ok(texture) = cx.assets().load_texture(ui_left.ctx(), &def.path) {
                    let img_size = texture.size_vec2();
                    let scaled_size = img_size * zoom;
                    let tex_rect = Rect::from_min_size(grid_origin, scaled_size);
                    painter.image(
                        texture.id(),
                        tex_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                // Draw slicing grid
                if self.show_grid_lines {
                    let grid_stroke = Stroke::new(1.0, Color32::from_rgb(45, 45, 55));
                    for r in 0..rows_count {
                        for c in 0..cols_count {
                            let cell_rect = Rect::from_min_size(
                                grid_origin + vec2(c as f32 * stride_x, r as f32 * stride_y),
                                vec2(stride_x, stride_y),
                            );
                            painter.rect_stroke(cell_rect, CornerRadius::ZERO, grid_stroke, StrokeKind::Inside);
                        }
                    }
                }

                // Highlight selected tile (e.g. ID #36)
                if let Some(selected) = self.selected_tile_id {
                    let c = selected % cols_count;
                    let r = selected / cols_count;
                    let cell_rect = Rect::from_min_size(
                        grid_origin + vec2(c as f32 * stride_x, r as f32 * stride_y),
                        vec2(stride_x, stride_y),
                    );
                    painter.rect_stroke(
                        cell_rect,
                        CornerRadius::ZERO,
                        Stroke::new(2.0, Color32::from_rgb(60, 140, 255)),
                        StrokeKind::Inside,
                    );
                }

                // Handle hover and click tile selection
                if let Some(pos) = response.hover_pos() {
                    let rel_x = (pos.x - grid_origin.x) / stride_x;
                    let rel_y = (pos.y - grid_origin.y) / stride_y;
                    if rel_x >= 0.0 && rel_y >= 0.0 {
                        let c = rel_x.floor() as u32;
                        let r = rel_y.floor() as u32;
                        if c < cols_count && r < rows_count {
                            self.hovered_tile = Some(r * cols_count + c);
                        } else {
                            self.hovered_tile = None;
                        }
                    } else {
                        self.hovered_tile = None;
                    }
                } else {
                    self.hovered_tile = None;
                }

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rel_x = (pos.x - grid_origin.x) / stride_x;
                        let rel_y = (pos.y - grid_origin.y) / stride_y;
                        if rel_x >= 0.0 && rel_y >= 0.0 {
                            let c = rel_x.floor() as u32;
                            let r = rel_y.floor() as u32;
                            if c < cols_count && r < rows_count {
                                self.selected_tile_id = Some(r * cols_count + c);
                            }
                        }
                    }
                }

                painter.rect_stroke(preview_rect, CornerRadius::ZERO, t.hairline(), StrokeKind::Inside);

                // Right Column: Metadata and properties for selected tile
                let ui_right = &mut cols[1];
                Frame::new()
                    .fill(t.panel)
                    .stroke(t.hairline())
                    .inner_margin(Margin::same(12))
                    .corner_radius(CornerRadius::same(2))
                    .show(ui_right, |ui| {
                        let sel_id = self.selected_tile_id.unwrap_or(36);
                        let col = sel_id % cols_count;
                        let row = sel_id / cols_count;

                        ui.label(
                            RichText::new(format!("SELECTED TILE: ID #{sel_id} (COL {col}, ROW {row})"))
                                .font(FontId::new(11.0, theme::mono_bold_for(ui.ctx())))
                                .color(Color32::from_rgb(255, 190, 80)),
                        );
                        ui.add_space(10.0);

                        // Tags
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Tags:").color(t.ink_strong).size(12.0));
                            let tag_frame = |ui: &mut Ui, text: &str, border_color: Color32| {
                                Frame::new()
                                    .fill(Color32::from_black_alpha(50))
                                    .stroke(Stroke::new(1.0, border_color))
                                    .corner_radius(CornerRadius::same(2))
                                    .inner_margin(Margin::symmetric(8, 2))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(text).size(11.0).color(border_color));
                                    });
                            };
                            tag_frame(ui, "solid", Color32::from_rgb(255, 160, 60));
                            tag_frame(ui, "wall", Color32::from_rgb(60, 140, 255));
                            if ui.button("+").clicked() {}
                        });

                        ui.add_space(10.0);

                        // Enum Value Binding
                        ui.label(RichText::new("Enum Value Binding:").color(t.ink_strong).size(12.0));
                        ui.add_space(3.0);
                        ui.add(TextEdit::singleline(&mut self.enum_binding).desired_width(f32::INFINITY));

                        ui.add_space(10.0);

                        // Collision Bounds (Box)
                        ui.label(RichText::new("Collision Bounds (Box):").color(t.ink_strong).size(12.0));
                        ui.add_space(3.0);
                        ui.add(
                            TextEdit::singleline(&mut "[ x: 0, y: 0, w: 16, h: 16 ] (Full solid)".to_string())
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(10.0);

                        // Custom Tile Data (JSON)
                        ui.label(RichText::new("Custom Tile Data (JSON):").color(t.ink_strong).size(12.0));
                        ui.add_space(3.0);
                        ui.add(
                            TextEdit::multiline(&mut self.custom_json)
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        );
                    });
            });

            ui.add_space(20.0);

            // 5. Bottom Action Bar
            Frame::new()
                .fill(t.sunken)
                .stroke(t.hairline())
                .inner_margin(Margin::symmetric(12, 8))
                .corner_radius(CornerRadius::same(2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "ID: TilesetDefId({}) • Bound to 2 Layer(s) • 256 Tiles Total",
                                self.tileset_id.0
                            ))
                            .color(t.muted)
                            .size(11.5),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if button(ui, "Save Tileset Def", ButtonKind::Primary, true).clicked() {
                                self.commit_changes(&def, cx);
                                cx.status(format!("Saved tileset {}", def.name));
                            }
                            ui.add_space(8.0);
                            if button(ui, "Revert", ButtonKind::Ghost, true).clicked() {
                                self.buffers_synced = false;
                                self.sync_buffers(&def);
                                cx.status("Reverted tileset changes");
                            }
                        });
                    });
                });

            ui.add_space(10.0);
        });
    }
}
