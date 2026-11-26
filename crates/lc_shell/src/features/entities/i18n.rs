//! Localization catalog and keys for the Entities workspace.

use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

/// Domain name used for catalog registration and document lookups.
pub const DOMAIN: &str = "entity";

/// Translation keys used by the Entities workspace UI.
pub mod keys {
    pub const HEADER: &str = "entities.header";
    pub const NEW_BUTTON: &str = "entities.new_button";
    pub const TAB_DEFINITION: &str = "entities.tab.definition";
    pub const TAB_DOCUMENT: &str = "entities.tab.document";
    pub const DESCRIPTION: &str = "entities.description";
    pub const EMPTY_MESSAGE: &str = "entities.empty.message";
    pub const EMPTY_CREATE: &str = "entities.empty.create";
    pub const MENU_DUPLICATE: &str = "entities.menu.duplicate";
    pub const MENU_DELETE: &str = "entities.menu.delete";

    pub const ALL: &[&str] = &[
        HEADER,
        NEW_BUTTON,
        TAB_DEFINITION,
        TAB_DOCUMENT,
        DESCRIPTION,
        EMPTY_MESSAGE,
        EMPTY_CREATE,
        MENU_DUPLICATE,
        MENU_DELETE,
    ];
}

const ENTITY_CATALOG: &[CatalogFile] = &[
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Editor,
        text: include_str!("../../../../../assets/localization/en-US/editor/entities.ron"),
    },
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("overview"),
        text: include_str!("../../../../../assets/localization/en-US/docs/entity/overview.md"),
    },
];

/// Registers the entities localization catalog with [`lc_ui_api::i18n`].
pub fn register() {
    register_catalog(DOMAIN, ENTITY_CATALOG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_ui_api::i18n::{contains_key, doc_for, tr_for, DEFAULT_LANGUAGE};

    #[test]
    fn every_key_used_in_entities_exists_in_english() {
        register();
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn entities_keys_resolve_to_their_english_text() {
        register();
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::TAB_DEFINITION), "ENTITY DEF");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::TAB_DOCUMENT), "DOCUMENT");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::HEADER), "ENTITY DEFINITIONS");
    }

    #[test]
    fn entities_document_resolves_to_overview_markdown() {
        register();
        let overview = doc_for(DEFAULT_LANGUAGE, "entity", "overview");
        assert!(
            overview.contains("# Entities"),
            "expected overview markdown to contain '# Entities', got: {overview}"
        );
    }
}
