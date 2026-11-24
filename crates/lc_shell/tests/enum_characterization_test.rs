//! Comprehensive characterization tests for the Enum workspace behavior,
//! capturing current behavior of modals, tags, values rows, and identifier validation
//! before refactoring to shared components.

use egui::epaint::{ClippedShape, RectShape, TextShape};
use egui::{vec2, Context, FullOutput, Id, Pos2, RawInput, Rect, Shape};
use glam::UVec2;
use lc_core::defs::{EnumDef, EnumValueDef, TilesetDef};
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::project::Project;
use lc_shell::features::editor::app_mode::AppMode;
use lc_shell::features::editor::shell_app::TagDef;
use lc_shell::features::enums::file::{
    ConflictResolutionChoice, EnumConflict, FleiyaEnumDef, FleiyaEnumFile, FleiyaEnumValueDef,
    CURRENT_ENUM_FILE_VERSION, ENUM_FILE_FORMAT,
};
use lc_shell::features::enums::view::{
    render_enum_view, row_layout, ConflictModalState, DeleteTarget, EnumViewState,
};
use lc_shell::shared::ui::text_field::{validate_identifier, IdentifierError};
use lc_shell::ShellApp;

#[derive(Default)]
struct CollectedShapes {
    rects: Vec<RectShape>,
    texts: Vec<TextShape>,
}

fn collect_shapes(shape: &Shape, out: &mut CollectedShapes) {
    match shape {
        Shape::Rect(r) => out.rects.push(r.clone()),
        Shape::Text(t) => out.texts.push(t.clone()),
        Shape::Vec(vec) => {
            for s in vec {
                collect_shapes(s, out);
            }
        }
        _ => {}
    }
}

fn collect_from_output(shapes: &[ClippedShape]) -> CollectedShapes {
    let mut collected = CollectedShapes::default();
    for clipped in shapes {
        collect_shapes(&clipped.shape, &mut collected);
    }
    collected
}

fn setup_test_app() -> ShellApp {
    let mut app = ShellApp::new(Project::new("Characterization"));
    app.active_mode = AppMode::Enums;

    app.project.defs.tilesets.push(TilesetDef {
        id: TilesetDefId(1),
        name: "Dungeon_16".to_string(),
        path: "assets/tiles/dungeon.png".to_string(),
        tile_size: UVec2::new(16, 16),
        spacing: 0,
        padding: 0,
        tags: Vec::new(),
        marking: lc_core::defs::TileMarking::EMPTY,
        custom_data: Default::default(),
        animations: Default::default(),
    });

    app.tag_registry.push(TagDef {
        id: lc_shell::features::editor::shell_app::TagId(1),
        name: "Gameplay".to_string(),
    });

    let enum_id = EnumDefId(1);
    let values = vec![
        EnumValueDef {
            id: "Common".to_string(),
            name: "Common Item".to_string(),
            color: [156, 163, 175, 255],
            tile: None,
        },
        EnumValueDef {
            id: "Rare".to_string(),
            name: "Rare Item".to_string(),
            color: [61, 214, 184, 255],
            tile: None,
        },
    ];

    app.enum_draft.push(EnumDef {
        id: enum_id,
        name: "ItemRarity".to_string(),
        tileset_id: Some(TilesetDefId(1)),
        tags: vec!["Gameplay".to_string()],
        values,
    });

    app.enum_tags_map.insert(
        enum_id,
        vec![lc_shell::features::editor::shell_app::TagId(1)],
    );

    app.selected_enum_def = Some(enum_id);
    app.enum_draft_revision = 1;
    app.enum_saved_revision = 0;
    app
}

fn render_with_state(
    app: &mut ShellApp,
    configure_state: impl FnOnce(&mut EnumViewState),
) -> (FullOutput, CollectedShapes) {
    let ctx = Context::default();
    lc_canvas::theme::install_fonts(&ctx);
    lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);

    let state_id = Id::new("enum_mode_view_transient_state");
    let mut state = EnumViewState::default();
    configure_state(&mut state);
    ctx.data_mut(|d| d.insert_temp(state_id, state));

    let raw_input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1366.0, 768.0))),
        ..Default::default()
    };

    // Frame 1 to initialize layout & areas
    let mut out1 = ctx.run_ui(raw_input.clone(), |ui| {
        render_enum_view(app, ui);
    });
    out1.textures_delta.clear();

    // Frame 2 where anchored areas are positioned and rendered
    let mut output = ctx.run_ui(raw_input, |ui| {
        render_enum_view(app, ui);
    });
    output.textures_delta.clear();
    let collected = collect_from_output(&output.shapes);
    (output, collected)
}

#[test]
fn test_modal_characterization_revert() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        state.revert_confirm_open = true;
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Discard Changes?"),
        "revert modal heading must be present"
    );
    assert!(
        collected.texts.iter().any(|t| {
            t.galley
                .text()
                .contains("Discard all unsaved enum changes? This cannot be undone.")
        }),
        "revert modal description must be present"
    );
    assert!(
        collected.texts.iter().any(|t| t.galley.text() == "Discard"),
        "revert modal Discard button must be present"
    );
    assert!(
        collected.texts.iter().any(|t| t.galley.text() == "Cancel"),
        "revert modal Cancel button must be present"
    );
}

#[test]
fn test_modal_characterization_change_tileset() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        state.pending_tileset_change = Some(None);
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Change Tileset?"),
        "change tileset modal heading must be present"
    );
    assert!(
        collected.texts.iter().any(|t| {
            t.galley
                .text()
                .contains("Changing the tileset will clear all bound tiles in this enum. Continue?")
        }),
        "change tileset description must be present"
    );
    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Change Tileset"),
        "change tileset action button must be present"
    );
}

#[test]
fn test_modal_characterization_delete_enum() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        state.pending_delete = Some(DeleteTarget::Enum {
            id: EnumDefId(1),
            name: "ItemRarity".to_string(),
            entity_count: 2,
            field_count: 3,
        });
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Delete Enum Definition?"),
        "delete modal heading must be present"
    );
    assert!(
        collected.texts.iter().any(|t| {
            t.galley
                .text()
                .contains("Are you sure you want to delete enum 'ItemRarity'?")
        }),
        "delete modal target description must be present"
    );
    assert!(
        collected.texts.iter().any(|t| {
            t.galley
                .text()
                .contains("Used by 2 Entity Def(s) & 3 Field(s)")
        }),
        "delete modal usage warning must be present"
    );
    assert!(
        collected.texts.iter().any(|t| t.galley.text() == "Delete"),
        "delete modal action button must be present"
    );
}

#[test]
fn test_modal_characterization_delete_value() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        state.pending_delete = Some(DeleteTarget::EnumValue {
            enum_id: EnumDefId(1),
            val_id: "Common".to_string(),
            name: "Common".to_string(),
        });
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Delete Enum Value?"),
        "delete value modal heading must be present"
    );
    assert!(
        collected.texts.iter().any(|t| {
            t.galley
                .text()
                .contains("Are you sure you want to delete value 'Common'?")
        }),
        "delete value description must be present"
    );
}

#[test]
fn test_modal_characterization_conflict() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        let file_enum = FleiyaEnumDef {
            identifier: "ItemRarity".to_string(),
            uid: 100,
            values: vec![FleiyaEnumValueDef {
                id: "Epic".to_string(),
                tile_rect: None,
                color: 0xFF0000,
            }],
            icon_tileset_uid: None,
            external_rel_path: None,
            external_file_checksum: None,
            tags: vec![],
        };
        state.conflict_modal = Some(ConflictModalState {
            file_data: FleiyaEnumFile {
                format: ENUM_FILE_FORMAT.to_string(),
                version: CURRENT_ENUM_FILE_VERSION,
                enums: vec![file_enum.clone()],
            },
            conflicts: vec![EnumConflict {
                file_enum,
                existing_id: EnumDefId(1),
                existing_name: "ItemRarity".to_string(),
                existing_values: vec![],
                entity_count: 0,
                field_count: 0,
                choice: ConflictResolutionChoice::KeepBoth,
            }],
            understand_checked: false,
        });
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Enum Identifier Conflicts"),
        "conflict modal heading must be present"
    );
    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Apply Import"),
        "apply import button must be present"
    );
}

#[test]
fn test_modal_characterization_error() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |state| {
        state.error_modal = Some(vec!["Invalid enum name characters".to_string()]);
    });

    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Validation Errors"),
        "error modal heading must be present"
    );
    assert!(
        collected
            .texts
            .iter()
            .any(|t| { t.galley.text().contains("• Invalid enum name characters") }),
        "validation error message must be present"
    );
    assert!(
        collected.texts.iter().any(|t| t.galley.text() == "Close"),
        "error modal close button must be present"
    );
}

#[test]
fn test_identifier_validation_rules() {
    let existing = ["ItemRarity", "WeaponType"];

    assert_eq!(
        validate_identifier("", &existing, None),
        Err(IdentifierError::Empty)
    );
    assert_eq!(
        validate_identifier("   ", &existing, None),
        Err(IdentifierError::Empty)
    );
    assert_eq!(
        validate_identifier("123Item", &existing, None),
        Err(IdentifierError::InvalidCharacters)
    );
    assert_eq!(
        validate_identifier("Item Rarity", &existing, None),
        Err(IdentifierError::InvalidCharacters)
    );
    assert_eq!(
        validate_identifier("Item-Rarity", &existing, None),
        Err(IdentifierError::InvalidCharacters)
    );
    assert_eq!(
        validate_identifier("itemrarity", &existing, None),
        Err(IdentifierError::Duplicate)
    );
    assert_eq!(
        validate_identifier("ItemRarity", &existing, Some("ItemRarity")),
        Ok(())
    );
    assert_eq!(validate_identifier("New_Enum_1", &existing, None), Ok(()));
}

#[test]
fn test_tags_row_characterization() {
    let mut app = setup_test_app();
    let (_, collected) = render_with_state(&mut app, |_| {});

    // In default setup_test_app, tag is "Gameplay"
    assert!(
        collected.texts.iter().any(|t| t.galley.text() == "Tags:"),
        "Tags row label must be present"
    );
    assert!(
        collected
            .texts
            .iter()
            .any(|t| t.galley.text() == "Gameplay"),
        "Assigned tag chip 'Gameplay' must be rendered"
    );
}

#[test]
fn test_values_row_characterization() {
    for width in [300.0, 500.0, 800.0, 1200.0] {
        let row_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), vec2(width, 32.0));
        let r = row_layout(row_rect);

        assert_eq!(r.handle.width(), 20.0);
        assert_eq!(r.handle.height(), 32.0);
        assert_eq!(r.bound_tile.width(), 32.0);
        assert_eq!(r.bound_tile.height(), 32.0);
        assert_eq!(r.color.width(), 32.0);
        assert_eq!(r.color.height(), 32.0);
        assert_eq!(r.delete_btn.width(), 32.0);
        assert_eq!(r.delete_btn.height(), 32.0);
        assert_eq!(r.delete_btn.max.x, row_rect.max.x);
    }
}
