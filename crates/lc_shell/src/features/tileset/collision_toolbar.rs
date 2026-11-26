//! Toolbar controls for collision editing tools, presets, grid snapping, and clipboard.

use egui::{Key, RichText, Ui};
use lc_canvas::icons::Icon;
use lc_canvas::theme::Tokens;
use lc_core::collision::{CollisionShape, TileCollisionDef};
use lc_core::defs::TilesetDef;
use lc_feature_tileset::{
    compute_tileset_grid, BatchSetTileCollisionCommand, RemoveTileCollisionCommand,
    SetTileCollisionCommand,
};
use lc_ui_api::AssetService;

use super::state::CollisionTool;
use crate::shared::ui::icon_button::{icon_button, IconButtonStyle};
use crate::ShellApp;

/// Renders the collision toolbar row adjacent to the TILE ATLAS mode selector.
pub fn render_collision_toolbar(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    handle_collision_shortcuts(app, ui, tileset);

    let active_tile_id = app
        .tileset_view_state
        .collision_focus_tile
        .or(app.tileset_view_state.selected_tile);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;

        // 1. Tool selection buttons: Select, Box, Polygon
        let cur_tool = app.tileset_view_state.collision_tool;

        let sel_style = if cur_tool == CollisionTool::Select {
            IconButtonStyle::bordered().selected(true).square()
        } else {
            IconButtonStyle::bordered().square()
        };
        if icon_button(ui, Icon::Cursor, 22.0, sel_style)
            .on_hover_text("Select & Edit Collision Shape (V)")
            .clicked()
        {
            app.tileset_view_state.collision_tool = CollisionTool::Select;
        }

        let box_style = if cur_tool == CollisionTool::Box {
            IconButtonStyle::bordered().selected(true).square()
        } else {
            IconButtonStyle::bordered().square()
        };
        if icon_button(ui, Icon::Rectangle, 22.0, box_style)
            .on_hover_text("Draw Rectangle Shape (B)")
            .clicked()
        {
            app.tileset_view_state.collision_tool = CollisionTool::Box;
        }

        let poly_style = if cur_tool == CollisionTool::Polygon {
            IconButtonStyle::bordered().selected(true).square()
        } else {
            IconButtonStyle::bordered().square()
        };
        if icon_button(ui, Icon::Shape, 22.0, poly_style)
            .on_hover_text("Draw Polygon Shape (P)")
            .clicked()
        {
            app.tileset_view_state.collision_tool = CollisionTool::Polygon;
        }

        ui.add_space(4.0);

        // 2. Presets (moved from right panel, icon-only)
        if icon_button(ui, Icon::Square, 22.0, IconButtonStyle::bordered().square())
            .on_hover_text("Preset: Full Box (Fill entire tile)")
            .clicked()
        {
            if let Some(tid) = active_tile_id {
                let col = TileCollisionDef::full_tile(tileset.tile_size);
                app.execute_command(Box::new(SetTileCollisionCommand::new(tileset.id, tid, col)));
            }
        }

        if icon_button(
            ui,
            Icon::TrendUp,
            22.0,
            IconButtonStyle::bordered().square(),
        )
        .on_hover_text("Preset: Slope Up (45-degree / triangle)")
        .clicked()
        {
            if let Some(tid) = active_tile_id {
                let mut col = TileCollisionDef::new();
                col.shapes.push(CollisionShape::slope_up(
                    col.next_shape_id(),
                    tileset.tile_size,
                ));
                app.execute_command(Box::new(SetTileCollisionCommand::new(tileset.id, tid, col)));
            }
        }

        if icon_button(
            ui,
            Icon::TrendDown,
            22.0,
            IconButtonStyle::bordered().square(),
        )
        .on_hover_text("Preset: Slope Down (45-degree \\ triangle)")
        .clicked()
        {
            if let Some(tid) = active_tile_id {
                let mut col = TileCollisionDef::new();
                col.shapes.push(CollisionShape::slope_down(
                    col.next_shape_id(),
                    tileset.tile_size,
                ));
                app.execute_command(Box::new(SetTileCollisionCommand::new(tileset.id, tid, col)));
            }
        }

        if icon_button(ui, Icon::Clear, 22.0, IconButtonStyle::bordered().square())
            .on_hover_text("Preset: Clear All Collision on Active Tile")
            .clicked()
        {
            if let Some(tid) = active_tile_id {
                app.execute_command(Box::new(RemoveTileCollisionCommand::new(tileset.id, tid)));
            }
        }

        ui.add_space(4.0);

        // 3. Snapping toggle (Pixel 1px / Tile / Off)
        let snap_mode = app.tileset_view_state.collision_snap;
        let snap_label = format!("Snap: {}", snap_mode.label());
        let snap_btn = egui::Button::new(RichText::new(snap_label).size(11.0).color(
            if snap_mode == super::state::CollisionSnapMode::None {
                t.muted
            } else {
                t.ink_strong
            },
        ))
        .corner_radius(egui::CornerRadius::ZERO);

        if ui
            .add(snap_btn)
            .on_hover_text("Cycle Grid Snapping: Pixel (1px) -> Tile Size -> Off (Shortcut: G)")
            .clicked()
        {
            app.tileset_view_state.collision_snap.cycle();
        }

        ui.add_space(4.0);

        // 4. Clipboard actions: Copy & Paste (works inside and outside isolate)
        let has_col = active_tile_id.is_some_and(|id| tileset.has_collision(id));
        let has_clip = app.tileset_view_state.collision_clipboard.is_some();

        // Copy button
        let copy_btn = icon_button(ui, Icon::Copy, 22.0, IconButtonStyle::bordered().square())
            .on_hover_text(if has_col {
                "Copy Tile Collision (Ctrl+C)"
            } else {
                "No collision to copy on active tile"
            });
        if has_col && copy_btn.clicked() {
            if let Some(tid) = active_tile_id {
                if let Some(col) = tileset.collision(tid) {
                    app.tileset_view_state.collision_clipboard = Some(col.clone());
                }
            }
        }

        // Paste button
        let paste_btn = icon_button(ui, Icon::Paste, 22.0, IconButtonStyle::bordered().square())
            .on_hover_text(if has_clip {
                "Paste Collision to Active Tile / Selection (Ctrl+V)"
            } else {
                "Clipboard is empty (Copy a tile collision first)"
            });
        if has_clip && paste_btn.clicked() {
            if let Some(tid) = active_tile_id {
                paste_collision(app, ui, tileset, tid);
            }
        }

        ui.add_space(4.0);

        // 5. Collision Overlay Toggle
        let overlay_icon = if app.tileset_view_state.show_collision_overlay {
            Icon::Eye
        } else {
            Icon::EyeOff
        };
        let overlay_style = if app.tileset_view_state.show_collision_overlay {
            IconButtonStyle::bordered().selected(true).square()
        } else {
            IconButtonStyle::bordered().square()
        };
        if icon_button(ui, overlay_icon, 22.0, overlay_style)
            .on_hover_text(if app.tileset_view_state.show_collision_overlay {
                "Hide collision overlay on atlas"
            } else {
                "Show collision overlay on atlas"
            })
            .clicked()
        {
            app.tileset_view_state.show_collision_overlay =
                !app.tileset_view_state.show_collision_overlay;
        }

        ui.add_space(4.0);

        // 6. Isolate Mode Toggle Button
        let is_isolated = app.tileset_view_state.collision_focus_tile.is_some();
        let isolate_style = if is_isolated {
            IconButtonStyle::bordered().selected(true).square()
        } else {
            IconButtonStyle::bordered().square()
        };

        let isolate_tip = if let Some(fid) = app.tileset_view_state.collision_focus_tile {
            format!("Exit Isolate Mode for Tile #{fid} (Esc)")
        } else if let Some(tid) = active_tile_id {
            format!("Isolate Tile #{tid} on Canvas (Alt+Click / Esc)")
        } else {
            "Select a tile to Isolate on Canvas".to_string()
        };

        if icon_button(ui, Icon::Fit, 22.0, isolate_style)
            .on_hover_text(isolate_tip)
            .clicked()
        {
            if is_isolated {
                app.tileset_view_state.exit_collision_focus();
            } else if let Some(tid) = active_tile_id {
                app.tileset_view_state.collision_focus_tile = Some(tid);
            }
        }
    });
}

/// Pastes the clipboard collision data onto the active tile (or multi-selected tiles).
fn paste_collision(app: &mut ShellApp, ui: &Ui, tileset: &TilesetDef, active_tile_id: u32) {
    if let Some(ref clip) = app.tileset_view_state.collision_clipboard {
        let multi_tiles = app
            .assets
            .load_texture(ui.ctx(), &tileset.path)
            .ok()
            .and_then(|tex| {
                let img_size = tex.size_vec2();
                let metrics = compute_tileset_grid(
                    [img_size.x as u32, img_size.y as u32],
                    tileset.tile_size,
                    tileset.spacing,
                    tileset.padding,
                );
                app.tileset_view_state
                    .canvas_states
                    .get(&tileset.id)
                    .and_then(|cs| cs.selected_tile_rect)
                    .map(|rect| {
                        rect.cells()
                            .map(|(c, r)| c + r * metrics.cols.max(1))
                            .collect::<Vec<u32>>()
                    })
            });

        if let Some(tiles) = multi_tiles {
            if tiles.len() > 1 {
                app.execute_command(Box::new(BatchSetTileCollisionCommand::new(
                    tileset.id,
                    tiles,
                    clip.clone(),
                )));
                return;
            }
        }

        app.execute_command(Box::new(SetTileCollisionCommand::new(
            tileset.id,
            active_tile_id,
            clip.clone(),
        )));
    }
}

/// Handles keyboard shortcuts for collision mode (scoped to Atlas / ToolAttributes focus).
fn handle_collision_shortcuts(app: &mut ShellApp, ui: &Ui, tileset: &TilesetDef) {
    let ctx = ui.ctx();
    let card_focused = matches!(
        app.tileset_view_state.focused_card,
        Some(super::state::TilesetCard::Atlas | super::state::TilesetCard::ToolAttributes)
    );
    if !card_focused || ctx.egui_wants_keyboard_input() {
        return;
    }

    // G: Cycle snap mode
    if ctx.input(|i| i.key_pressed(Key::G)) {
        app.tileset_view_state.collision_snap.cycle();
    }

    // V: Select tool
    if ctx.input(|i| i.key_pressed(Key::V)) {
        app.tileset_view_state.collision_tool = CollisionTool::Select;
    }

    // B: Box tool
    if ctx.input(|i| i.key_pressed(Key::B)) {
        app.tileset_view_state.collision_tool = CollisionTool::Box;
    }

    // P: Polygon tool
    if ctx.input(|i| i.key_pressed(Key::P)) {
        app.tileset_view_state.collision_tool = CollisionTool::Polygon;
    }

    // ESC: Exit focus mode
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        app.tileset_view_state.exit_collision_focus();
    }

    // Ctrl+C: Copy collision (works inside and outside isolate)
    if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::C)) {
        let active_tile_id = app
            .tileset_view_state
            .collision_focus_tile
            .or(app.tileset_view_state.selected_tile);
        if let Some(tid) = active_tile_id {
            if let Some(col) = tileset.collision(tid) {
                app.tileset_view_state.collision_clipboard = Some(col.clone());
            }
        }
    }

    // Ctrl+V: Paste collision (works inside and outside isolate)
    if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::V)) {
        let active_tile_id = app
            .tileset_view_state
            .collision_focus_tile
            .or(app.tileset_view_state.selected_tile);
        if let Some(tid) = active_tile_id {
            paste_collision(app, ui, tileset, tid);
        }
    }
}
