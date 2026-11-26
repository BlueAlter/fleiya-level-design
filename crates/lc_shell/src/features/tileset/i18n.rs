//! Localization catalog and keys for the Tileset feature.
//!
//! Tileset owns its string tables and document pages, embedded at compile time
//! and registered through [`register`] so the shared i18n module never names
//! domain files.

use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

/// Domain name used for catalog registration and document lookups.
pub const DOMAIN: &str = "tileset";

/// Translation keys used by the Tileset feature UI.
pub mod keys {
    pub const TAB_DEFINITION: &str = "tileset.tab.definition";
    pub const TAB_MARKING: &str = "tileset.tab.marking";
    pub const TAB_COLLISION: &str = "tileset.tab.collision";
    pub const TAB_ANIMATION: &str = "tileset.tab.animation";
    pub const TAB_DOCUMENT: &str = "tileset.tab.document";
    pub const DESCRIPTION: &str = "tileset.description";
    pub const NEW_BUTTON: &str = "tileset.new_button";
    pub const SEARCH_PLACEHOLDER: &str = "tileset.search_placeholder";
    pub const HEADER: &str = "tileset.header";
    pub const CARD_IDENTIFIER_ATTRIBUTES: &str = "tileset.card.identifier_attributes";
    pub const FIELD_IDENTIFIER: &str = "tileset.field.identifier";
    pub const FIELD_SOURCE_IMAGE: &str = "tileset.field.source_image";
    pub const FIELD_TILES_LAYOUT: &str = "tileset.field.tiles_layout";
    pub const FIELD_TAGS: &str = "tileset.field.tags";
    pub const FIELD_MARKING_ENUM: &str = "tileset.field.marking_enum";
    pub const CHOOSE_IMAGE: &str = "tileset.button.choose_image";
    pub const BUTTON_BROWSE: &str = "tileset.button.browse";
    pub const BUTTON_CLEAR: &str = "tileset.button.clear";
    pub const FIELD_TILE_SIZE: &str = "tileset.field.tile_size";
    pub const FIELD_SPACING: &str = "tileset.field.spacing";
    pub const FIELD_PADDING: &str = "tileset.field.padding";
    pub const MENU_DUPLICATE: &str = "tileset.menu.duplicate";
    pub const MENU_DELETE: &str = "tileset.menu.delete";
    pub const EMPTY_MESSAGE: &str = "tileset.empty.message";
    pub const EMPTY_CREATE_BUTTON: &str = "tileset.empty.create_button";
    pub const DELETE_CONFIRM_TITLE: &str = "tileset.delete.confirm_title";
    pub const DELETE_CONFIRM_MESSAGE: &str = "tileset.delete.confirm_message";
    pub const DELETE_CONFIRM_BUTTON: &str = "tileset.delete.confirm_button";
    pub const DELETE_CANCEL_BUTTON: &str = "tileset.delete.cancel_button";
    pub const MODE_NONE: &str = "tileset.mode.none";
    pub const MODE_MARKING: &str = "tileset.mode.marking";
    pub const MODE_ANIMATION: &str = "tileset.mode.animation";
    pub const MODE_COMING_SOON: &str = "tileset.mode.coming_soon";
    pub const ERROR_OUTSIDE_ROOT: &str = "tileset.error.outside_root";
    pub const VALIDATION_NAME_REQUIRED: &str = "tileset.validation.name_required";
    pub const VALIDATION_NAME_DUPLICATE: &str = "tileset.validation.name_duplicate";

    pub const ALL: &[&str] = &[
        TAB_DEFINITION,
        TAB_MARKING,
        TAB_COLLISION,
        TAB_ANIMATION,
        TAB_DOCUMENT,
        DESCRIPTION,
        NEW_BUTTON,
        SEARCH_PLACEHOLDER,
        HEADER,
        CARD_IDENTIFIER_ATTRIBUTES,
        FIELD_IDENTIFIER,
        FIELD_SOURCE_IMAGE,
        FIELD_TILES_LAYOUT,
        FIELD_TAGS,
        FIELD_MARKING_ENUM,
        CHOOSE_IMAGE,
        BUTTON_BROWSE,
        BUTTON_CLEAR,
        FIELD_TILE_SIZE,
        FIELD_SPACING,
        FIELD_PADDING,
        MENU_DUPLICATE,
        MENU_DELETE,
        EMPTY_MESSAGE,
        EMPTY_CREATE_BUTTON,
        DELETE_CONFIRM_TITLE,
        DELETE_CONFIRM_MESSAGE,
        DELETE_CONFIRM_BUTTON,
        DELETE_CANCEL_BUTTON,
        MODE_NONE,
        MODE_MARKING,
        MODE_ANIMATION,
        MODE_COMING_SOON,
        ERROR_OUTSIDE_ROOT,
        VALIDATION_NAME_REQUIRED,
        VALIDATION_NAME_DUPLICATE,
    ];
}

/// Catalog files embedded for the Tileset feature.
const TILESET_CATALOG: &[CatalogFile] = &[
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Editor,
        text: include_str!("../../../../../assets/localization/en-US/editor/tileset.ron"),
    },
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("overview"),
        text: include_str!("../../../../../assets/localization/en-US/docs/tileset/overview.md"),
    },
];

/// Registers the tileset localization catalog with [`lc_ui_api::i18n`].
pub fn register() {
    register_catalog(DOMAIN, TILESET_CATALOG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_ui_api::i18n::{contains_key, doc_for, DEFAULT_LANGUAGE};

    #[test]
    fn every_key_used_in_tileset_exists_in_english() {
        register();
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn tileset_overview_doc_is_registered_and_non_empty() {
        register();
        let doc = doc_for(DEFAULT_LANGUAGE, DOMAIN, "overview");
        assert!(
            !doc.trim().is_empty(),
            "tileset overview doc must not be empty"
        );
        assert!(doc.contains("# Tilesets"));
    }
}
