//! Localization catalog and keys for the Layers workspace.

use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

/// Domain name used for catalog registration and document lookups.
pub const DOMAIN: &str = "layer";

/// Translation keys used by the Layers workspace UI.
pub mod keys {
    pub const HEADER: &str = "layers.header";
    pub const NEW_BUTTON: &str = "layers.new_button";
    pub const TAB_DEFINITION: &str = "layers.tab.definition";
    pub const TAB_AUTO_TILE: &str = "layers.tab.auto_tile";
    pub const TAB_DOCUMENT: &str = "layers.tab.document";
    pub const DESCRIPTION: &str = "layers.description";
    pub const EMPTY_MESSAGE: &str = "layers.empty.message";
    pub const EMPTY_CREATE: &str = "layers.empty.create";
    pub const MENU_DUPLICATE: &str = "layers.menu.duplicate";
    pub const MENU_DELETE: &str = "layers.menu.delete";

    pub const ALL: &[&str] = &[
        HEADER,
        NEW_BUTTON,
        TAB_DEFINITION,
        TAB_AUTO_TILE,
        TAB_DOCUMENT,
        DESCRIPTION,
        EMPTY_MESSAGE,
        EMPTY_CREATE,
        MENU_DUPLICATE,
        MENU_DELETE,
    ];
}

const LAYER_CATALOG: &[CatalogFile] = &[
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Editor,
        text: include_str!("../../../../../assets/localization/en-US/editor/layers.ron"),
    },
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("overview"),
        text: include_str!("../../../../../assets/localization/en-US/docs/layer/overview.md"),
    },
];

/// Registers the layers localization catalog with [`lc_ui_api::i18n`].
pub fn register() {
    register_catalog(DOMAIN, LAYER_CATALOG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_ui_api::i18n::{contains_key, doc_for, tr_for, DEFAULT_LANGUAGE};

    #[test]
    fn every_key_used_in_layers_exists_in_english() {
        register();
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn layers_keys_resolve_to_their_english_text() {
        register();
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::TAB_DEFINITION), "LAYERS DEF");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::TAB_AUTO_TILE), "AUTO TILE");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::TAB_DOCUMENT), "DOCUMENT");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::HEADER), "LAYER DEFINITIONS");
    }

    #[test]
    fn layers_document_resolves_to_overview_markdown() {
        register();
        let overview = doc_for(DEFAULT_LANGUAGE, "layer", "overview");
        assert!(
            overview.contains("# Layers"),
            "expected overview markdown to contain '# Layers', got: {overview}"
        );
    }
}
