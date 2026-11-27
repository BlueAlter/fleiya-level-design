//! Localization catalog and keys for the Settings workspace.

use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

/// Domain name used for catalog registration.
pub const DOMAIN: &str = "settings";

/// Translation keys used by the Settings workspace UI.
pub mod keys {
    /// Header label in the left category sidebar.
    pub const HEADER: &str = "settings.header";
    /// Search box placeholder text.
    pub const SEARCH_HINT: &str = "settings.search_hint";
    /// Sidebar group label for app-level preferences.
    pub const GROUP_APPLICATION: &str = "settings.group.application";
    /// Sidebar group label for current project settings.
    pub const GROUP_PROJECT: &str = "settings.group.project";
    /// Footer message when preferences persist to disk.
    pub const FOOTER_SAVED: &str = "settings.footer.saved";
    /// Footer message when running without a disk preferences path.
    pub const FOOTER_MEMORY_ONLY: &str = "settings.footer.memory_only";
    /// Button label to reset the active category to defaults.
    pub const RESET_SECTION: &str = "settings.reset_section";
    /// Button label to reset all preferences to defaults.
    pub const RESET_ALL: &str = "settings.reset_all";

    /// Every key defined above, for completeness tests.
    pub const ALL: &[&str] = &[
        HEADER,
        SEARCH_HINT,
        GROUP_APPLICATION,
        GROUP_PROJECT,
        FOOTER_SAVED,
        FOOTER_MEMORY_ONLY,
        RESET_SECTION,
        RESET_ALL,
    ];
}

const SETTINGS_CATALOG: &[CatalogFile] = &[CatalogFile {
    language: DEFAULT_LANGUAGE,
    kind: CatalogKind::Editor,
    text: include_str!("../../../../../assets/localization/en-US/editor/settings.ron"),
}];

/// Registers the settings localization catalog with [`lc_ui_api::i18n`].
pub fn register() {
    register_catalog(DOMAIN, SETTINGS_CATALOG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use lc_ui_api::i18n::{contains_key, tr_for, DEFAULT_LANGUAGE};

    #[test]
    fn every_key_used_in_settings_exists_in_english() {
        register();
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn settings_keys_resolve_to_english_text() {
        register();
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::HEADER), "SETTINGS");
        assert_eq!(tr_for(DEFAULT_LANGUAGE, keys::GROUP_PROJECT), "CURRENT PROJECT");
    }
}
