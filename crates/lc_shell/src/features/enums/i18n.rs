//! Localization catalog and keys for the Enums feature.
//!
//! Enums owns its string tables and document pages, embedded at compile time
//! and registered through [`register`] so the shared i18n module never names
//! domain files.

use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

/// Domain name used for catalog registration and document lookups.
pub const DOMAIN: &str = "enum";

/// Translation keys used by the Enums feature UI.
pub mod keys {
    /// Label for the enum definition editor sub-tab.
    pub const ENUMS_TAB_DEFINITION: &str = "enums.tab.definition";
    /// Label for the enum documentation sub-tab.
    pub const ENUMS_TAB_DOCUMENT: &str = "enums.tab.document";
    /// Description text shown below the enum header.
    pub const ENUMS_DESCRIPTION: &str = "enums.description";

    /// Every key defined by this domain, for completeness tests.
    pub const ALL: &[&str] = &[ENUMS_TAB_DEFINITION, ENUMS_TAB_DOCUMENT, ENUMS_DESCRIPTION];
}

/// Catalog files embedded for the Enums feature.
const ENUM_CATALOG: &[CatalogFile] = &[
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Editor,
        text: include_str!("../../../../../assets/localization/en-US/editor/enums.ron"),
    },
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("overview"),
        text: include_str!("../../../../../assets/localization/en-US/docs/enum/overview.md"),
    },
];

/// Registers the enums localization catalog with [`lc_ui_api::i18n`].
///
/// Safe to call multiple times; the registry will ignore duplicate calls.
pub fn register() {
    register_catalog(DOMAIN, ENUM_CATALOG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_ui_api::i18n::{contains_key, doc_for, tr_for, DEFAULT_LANGUAGE};

    #[test]
    fn every_key_used_in_enums_exists_in_english() {
        register();
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn enums_keys_resolve_to_their_english_text() {
        register();
        assert_eq!(
            tr_for(DEFAULT_LANGUAGE, keys::ENUMS_TAB_DEFINITION),
            "ENUM DEF"
        );
        assert_eq!(
            tr_for(DEFAULT_LANGUAGE, keys::ENUMS_TAB_DOCUMENT),
            "DOCUMENT"
        );
        assert_eq!(
            tr_for(DEFAULT_LANGUAGE, keys::ENUMS_DESCRIPTION),
            "Define enumeration keys, localized labels, badge colors, and optional bound tile graphics."
        );
    }

    #[test]
    fn enums_document_resolves_to_overview_markdown() {
        register();
        let overview = doc_for(DEFAULT_LANGUAGE, "enum", "overview");
        assert!(
            overview.contains("# Enums"),
            "expected overview markdown to contain '# Enums', got: {overview}"
        );
    }
}
