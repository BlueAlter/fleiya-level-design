//! Tags row and Enum for tile marking row for the Tileset attributes card.

use egui::Ui;
use lc_canvas::theme::Tokens;
use lc_core::defs::TilesetDef;
use lc_core::marking::known_tags;
use lc_feature_tileset::{
    AddTilesetTagCommand, RemoveTilesetTagCommand, RenameTilesetTagCommand, SetMarkingSourceCommand,
};

use super::attributes_card::def_attribute_row;
use super::i18n::keys;
use crate::features::editor::shell_app::ShellApp;
use crate::shared::i18n::tr;
use crate::shared::ui::tag_row::{tag_row, TagEvent};
use crate::shared::ui::tileset_select::{tileset_select, TilesetSelectItem};
use crate::shared::ui::workspace_metrics::CONTROL_HEIGHT_FORM;

/// Renders the Tags row for a tileset definition.
pub fn render_tags_row(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    def_attribute_row(ui, t, tr(keys::FIELD_TAGS), |ui| {
        ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;

        let mut extra_tags = Vec::new();
        for e in &app.project.defs.enums {
            extra_tags.extend(e.tags.clone());
        }
        for tag_def in &app.tag_registry {
            extra_tags.push(tag_def.name.clone());
        }
        let all_tags = known_tags(&app.project.defs.tilesets, &extra_tags);

        let events = tag_row(ui, &tileset.tags, &all_tags);
        for ev in events {
            match ev {
                TagEvent::Add(tag_name) => {
                    app.execute_command(Box::new(AddTilesetTagCommand::new(tileset.id, tag_name)));
                }
                TagEvent::Remove(tag_name) => {
                    app.execute_command(Box::new(RemoveTilesetTagCommand::new(
                        tileset.id, tag_name,
                    )));
                }
                TagEvent::Rename { from, to } => {
                    app.execute_command(Box::new(RenameTilesetTagCommand::new(
                        tileset.id, from, to,
                    )));
                }
            }
        }
    });
}

/// Renders the Enum for tile marking row for a tileset definition.
pub fn render_marking_enum_row(app: &mut ShellApp, ui: &mut Ui, t: &Tokens, tileset: &TilesetDef) {
    def_attribute_row(ui, t, tr(keys::FIELD_MARKING_ENUM), |ui| {
        ui.spacing_mut().interact_size.y = CONTROL_HEIGHT_FORM;

        let enum_items: Vec<TilesetSelectItem<lc_core::id::EnumDefId>> = app
            .project
            .defs
            .enums
            .iter()
            .map(|e| TilesetSelectItem::new(e.id, e.name.clone()))
            .collect();

        let prev_enum = tileset.marking.source_enum;
        let mut chosen_enum = prev_enum;

        if tileset_select(ui, &enum_items, &mut chosen_enum, true) && chosen_enum != prev_enum {
            app.execute_command(Box::new(SetMarkingSourceCommand::new(
                tileset.id,
                chosen_enum,
            )));
        }
        app.tileset_view_state.is_enum_dropdown_open =
            crate::shared::ui::tileset_select::is_tileset_select_open(ui);
    });
}
