//! Tile palette dockable panel for picking single or rectangular blocks of tiles.

use egui::{
    vec2, Color32, ComboBox, CornerRadius, Frame, Margin, Pos2, Rect, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, Ui,
};
use lc_core::id::TilesetDefId;
use lc_core::project::Project;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::events::EditorEvent;
use lc_ui_api::session::TileSelection;
use lc_ui_api::view::{PanelId, PanelView};

use lc_canvas::grid::draw_checkerboard;
use lc_canvas::theme::Tokens;
use lc_canvas::widgets::hud;

/// Dockable panel view for selecting tiles from loaded tilesets.
pub struct TilePalettePanel {
    active_tileset_id: Option<TilesetDefId>,
    zoom: f32,
    drag_start: Option<(u32, u32)>,
    drag_current: Option<(u32, u32)>,
}

impl Default for TilePalettePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TilePalettePanel {
    /// Panel ID constant for the tile palette.
    pub const PANEL_ID: &'static str = "tile_palette";

    /// Creates a new `TilePalettePanel`.
    pub fn new() -> Self {
        Self {
            active_tileset_id: None,
            zoom: 2.0,
            drag_start: None,
            drag_current: None,
        }
    }

    /// Resolves the currently active tileset definition, falling back to selection or first in project.
    fn resolve_active_tileset<'a>(
        &mut self,
        p: &'a Project,
        cx: &EditorCtx,
    ) -> Option<&'a lc_core::defs::TilesetDef> {
        if p.defs.tilesets.is_empty() {
            self.active_tileset_id = None;
            return None;
        }

        // 1. If currently selected ID is still in project, use it
        if let Some(id) = self.active_tileset_id {
            if let Some(t) = p.defs.tileset(id) {
                return Some(t);
            }
        }

        // 2. If session tile selection has a tileset ID in project, use that
        if let Some(ts) = cx.tile_selection() {
            if let Some(t) = p.defs.tileset(ts.tileset_id) {
                self.active_tileset_id = Some(ts.tileset_id);
                return Some(t);
            }
        }

        // 3. Fallback to first tileset
        let first = p.defs.tilesets.first()?;
        self.active_tileset_id = Some(first.id);
        Some(first)
    }
}

impl PanelView for TilePalettePanel {
    fn id(&self) -> PanelId {
        PanelId::new(Self::PANEL_ID)
    }

    fn title(&self) -> &str {
        "Tile Palette"
    }

    fn ui(&mut self, ui: &mut Ui, cx: &mut EditorCtx) {
        let t = Tokens::get(ui.ctx());

        let tileset_def = self.resolve_active_tileset(cx.project(), cx).cloned();

        // 1. Top toolbar
        Frame::new()
            .fill(t.sunken)
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref def) = tileset_def {
                        ComboBox::from_id_salt("palette_tileset_selector")
                            .selected_text(&def.name)
                            .show_ui(ui, |ui| {
                                for t_def in &cx.project().defs.tilesets {
                                    let selected = self.active_tileset_id == Some(t_def.id);
                                    if ui.selectable_label(selected, &t_def.name).clicked() {
                                        self.active_tileset_id = Some(t_def.id);
                                    }
                                }
                            });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if cx.tile_selection().is_some() && ui.button("Clear").clicked() {
                            cx.emit(EditorEvent::SetTileSelection(None));
                        }

                        if ui.button("4x").clicked() {
                            self.zoom = 4.0;
                        }
                        if ui.button("2x").clicked() {
                            self.zoom = 2.0;
                        }
                        if ui.button("1x").clicked() {
                            self.zoom = 1.0;
                        }
                    });
                });
            });

        // 1b. Canvas Tools toolbar
        Frame::new()
            .fill(t.panel)
            .inner_margin(Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let active_tool = cx.active_tool_name().to_string();

                    let tools = [
                        ("Paint", "Paint / Stamp Tiles (B)"),
                        ("Erase", "Erase Tiles (E)"),
                        ("Rectangle", "Rectangle Fill (R)"),
                        ("Fill", "Flood Fill (F)"),
                        ("Eyedropper", "Pick Tile from Canvas (I)"),
                        ("Select", "Select a canvas region for copy/paste"),
                    ];

                    for (tool_name, tooltip) in tools {
                        let is_active = active_tool == tool_name;
                        if ui
                            .selectable_label(is_active, tool_name)
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            cx.emit(EditorEvent::SetActiveTool(tool_name.to_string()));
                        }
                    }
                });
            });

        ui.add_space(4.0);

        // 2. Palette content area
        let Some(def) = tileset_def else {
            ui.vertical_centered(|ui| {
                ui.add_space(24.0);
                ui.label(RichText::new("No tilesets defined.").color(t.muted));
                ui.label(
                    RichText::new("Add a tileset definition in the Project panel.")
                        .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                        .color(t.faint),
                );
            });
            return;
        };

        match cx.assets().load_texture(ui.ctx(), &def.path) {
            Ok(texture) => {
                let img_size = texture.size_vec2();
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

                let scaled_size = img_size * self.zoom;

                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let (rect, response) = ui.allocate_exact_size(
                            scaled_size + vec2(16.0, 16.0),
                            Sense::click_and_drag(),
                        );

                        let origin = rect.min + vec2(8.0, 8.0);
                        let tex_rect = Rect::from_min_size(origin, scaled_size);

                        let painter = ui.painter().with_clip_rect(rect);
                        painter.rect_filled(tex_rect, CornerRadius::ZERO, t.canvas);
                        draw_checkerboard(&painter, tex_rect, 16.0, &t);

                        // Draw texture
                        painter.image(
                            texture.id(),
                            tex_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );

                        // Helper for cell bounds
                        let cell_rect_for = |c: u32, r: u32| -> Rect {
                            let cx =
                                origin.x + (padding as f32 + (c * stride_x) as f32) * self.zoom;
                            let cy =
                                origin.y + (padding as f32 + (r * stride_y) as f32) * self.zoom;
                            Rect::from_min_size(
                                Pos2::new(cx, cy),
                                vec2(tile_w as f32 * self.zoom, tile_h as f32 * self.zoom),
                            )
                        };

                        // Draw grid lines
                        let grid_stroke = Stroke::new(1.0, t.grid_major);
                        for r in 0..rows {
                            for c in 0..cols {
                                let c_rect = cell_rect_for(c, r);
                                painter.rect_stroke(
                                    c_rect,
                                    CornerRadius::ZERO,
                                    grid_stroke,
                                    StrokeKind::Inside,
                                );
                            }
                        }

                        // Mouse interaction: click & drag tile selection
                        let get_tile_at = |pos: Pos2| -> Option<(u32, u32)> {
                            if !tex_rect.contains(pos) || cols == 0 || rows == 0 {
                                return None;
                            }
                            let rel_x = (pos.x - origin.x) / self.zoom;
                            let rel_y = (pos.y - origin.y) / self.zoom;
                            if rel_x < padding as f32 || rel_y < padding as f32 {
                                return None;
                            }
                            let c = ((rel_x - padding as f32) / stride_x as f32).floor() as u32;
                            let r = ((rel_y - padding as f32) / stride_y as f32).floor() as u32;
                            if c < cols && r < rows {
                                Some((c, r))
                            } else {
                                None
                            }
                        };

                        if response.drag_started()
                            || (response.clicked() && self.drag_start.is_none())
                        {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some((c, r)) = get_tile_at(pos) {
                                    self.drag_start = Some((c, r));
                                    self.drag_current = Some((c, r));
                                }
                            }
                        }

                        if response.dragged() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some((c, r)) = get_tile_at(pos) {
                                    self.drag_current = Some((c, r));
                                }
                            }
                        }

                        if response.drag_stopped() || response.clicked() {
                            if let (Some((c0, r0)), Some((c1, r1))) =
                                (self.drag_start.take(), self.drag_current.take())
                            {
                                let c_min = c0.min(c1);
                                let c_max = c0.max(c1);
                                let r_min = r0.min(r1);
                                let r_max = r0.max(r1);

                                let width = c_max - c_min + 1;
                                let mut tiles = Vec::new();
                                for r in r_min..=r_max {
                                    for c in c_min..=c_max {
                                        tiles.push(r * cols + c);
                                    }
                                }

                                let selection = if tiles.len() == 1 {
                                    TileSelection::single(def.id, tiles[0])
                                } else {
                                    TileSelection::rect(def.id, tiles, width)
                                };

                                cx.emit(EditorEvent::SetTileSelection(Some(selection)));
                            }
                        }

                        // Draw drag selection in progress
                        if let (Some((c0, r0)), Some((c1, r1))) =
                            (self.drag_start, self.drag_current)
                        {
                            let c_min = c0.min(c1);
                            let c_max = c0.max(c1);
                            let r_min = r0.min(r1);
                            let r_max = r0.max(r1);

                            let top_left = cell_rect_for(c_min, r_min).min;
                            let bottom_right = cell_rect_for(c_max, r_max).max;
                            let sel_box = Rect::from_min_max(top_left, bottom_right);

                            painter.rect_filled(sel_box, CornerRadius::ZERO, t.selection_fill);
                            painter.rect_stroke(
                                sel_box,
                                CornerRadius::ZERO,
                                Stroke::new(2.0, t.accent),
                                StrokeKind::Inside,
                            );
                        } else if let Some(ts) = cx.tile_selection() {
                            // Draw active selection from session if matching this tileset
                            if ts.tileset_id == def.id {
                                for &tile_id in &ts.tiles {
                                    if cols > 0 {
                                        let c = tile_id % cols;
                                        let r = tile_id / cols;
                                        if c < cols && r < rows {
                                            let c_rect = cell_rect_for(c, r);
                                            painter.rect_filled(
                                                c_rect,
                                                CornerRadius::ZERO,
                                                t.selection_fill,
                                            );
                                            painter.rect_stroke(
                                                c_rect,
                                                CornerRadius::ZERO,
                                                Stroke::new(2.0, t.selection_stroke),
                                                StrokeKind::Inside,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    });

                // Status info
                ui.add_space(4.0);
                Frame::new()
                    .fill(t.sunken)
                    .inner_margin(Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        if let Some(ts) = cx.tile_selection() {
                            if ts.tileset_id == def.id {
                                if ts.tiles.len() == 1 {
                                    hud(
                                        ui,
                                        &format!("SELECTED: Tile #{}", ts.tiles[0]),
                                        t.accent_ink,
                                    );
                                } else {
                                    let h = (ts.tiles.len() as u32) / ts.width.max(1);
                                    hud(
                                        ui,
                                        &format!(
                                            "SELECTED: {}x{} BLOCK ({} TILES)",
                                            ts.width,
                                            h,
                                            ts.tiles.len()
                                        ),
                                        t.accent_ink,
                                    );
                                }
                            } else {
                                hud(ui, "SELECTION IN ANOTHER TILESET", t.faint);
                            }
                        } else {
                            hud(ui, "CLICK / DRAG TILES TO SELECT", t.faint);
                        }
                    });
            }
            Err(err) => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        RichText::new(format!("Failed to load texture: {err}"))
                            .color(ui.visuals().error_fg_color),
                    );
                    ui.label(
                        RichText::new(format!("Asset path: \"{}\"", def.path))
                            .font(egui::FontId::new(12.0, egui::FontFamily::Proportional))
                            .color(t.muted),
                    );
                });
            }
        }
    }
}
