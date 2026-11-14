//! Placeholder view for the Tilesets workspace.

use egui::{CornerRadius, RichText, Ui};
use lc_canvas::theme::{self, Tokens};

use crate::features::editor::shell_app::ShellApp;

/// Renders the placeholder view for the Tilesets workspace.
///
/// This page is temporary: the real workspace task replaces it entirely, so plain English
/// literals are acceptable here. It must not mutate the project or queue commands.
pub fn render_tileset_view(app: &mut ShellApp, ui: &mut Ui) {
    let _ = app;
    let t = Tokens::get(ui.ctx());
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, CornerRadius::ZERO, t.bg);

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Tilesets")
                        .font(egui::FontId::new(26.0, theme::display_for(ui.ctx())))
                        .color(t.ink_strong),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "The tileset workspace is under construction. For now use the Tileset Window from the Windows menu.",
                    )
                    .font(egui::FontId::new(13.0, egui::FontFamily::Proportional))
                    .color(t.muted),
                );
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::defs::TilesetDef;
    use lc_core::id::TilesetDefId;
    use lc_core::project::Project;

    #[test]
    fn render_tileset_view_with_zero_tilesets() {
        let mut app = ShellApp::new(Project::new("Test"));
        let initial_project = app.project.clone();
        assert_eq!(app.project.defs.tilesets.len(), 0);

        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_tileset_view(&mut app, ui);
        });
        output.textures_delta.clear();

        assert_eq!(app.project, initial_project);
        assert!(app.pending_commands.is_empty());
    }

    #[test]
    fn render_tileset_view_with_three_tilesets() {
        let mut project = Project::new("Test");
        for i in 1..=3 {
            project.defs.tilesets.push(TilesetDef {
                id: TilesetDefId(i),
                name: format!("Tileset {i}"),
                path: format!("assets/tileset_{i}.png"),
                tile_size: UVec2::new(16, 16),
                spacing: 0,
                padding: 0,
            });
        }
        assert_eq!(project.defs.tilesets.len(), 3);

        let mut app = ShellApp::new(project);
        let initial_project = app.project.clone();

        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            render_tileset_view(&mut app, ui);
        });
        output.textures_delta.clear();

        assert_eq!(app.project, initial_project);
        assert!(app.pending_commands.is_empty());
    }
}
