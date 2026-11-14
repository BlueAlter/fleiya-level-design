//! Localization lookup for editor UI strings and documentation pages.
//!
//! Every language is a folder under `assets/localization/<lang>/` holding short
//! UI strings (`editor/<domain>.ron`) and long-form Markdown documents
//! (`docs/<topic>/<page>.md`). The files are embedded at compile time, so the
//! editor never reads them from disk at runtime.
//!
//! This crate embeds only the shared [`COMMON_CATALOG`] (`editor/common.ron`).
//! Each editor domain owns its own catalog (keys, `enums.ron`, document pages)
//! and registers it at startup with [`register_catalog`], so this module never
//! names a domain file.
//!
//! Lookup order is the selected language, then `en-US`, then the key itself.
//! A missing key is logged once as a warning so translation gaps are visible
//! without flooding the console every frame.
//!
//! The selected language is an application preference and deliberately not part
//! of `Project`: it changes how the editor talks to the user, not their data.
//! v1 ships `en-US` only.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock, RwLock};

/// Language used when the selected one has no entry.
pub const DEFAULT_LANGUAGE: &str = "en-US";

/// Well-known keys shared by more than one editor domain.
///
/// Every constant is checked against `en-US` by the module tests, so a key can
/// never be used in shared code without being translated. Domain keys live in
/// the owning feature crate.
pub mod keys {
    /// Shown by the shared searchable picker when no item matches the query.
    pub const SEARCH_NO_RESULTS: &str = "common.search.no_results";

    /// Every key defined above, for completeness tests.
    pub const ALL: &[&str] = &[SEARCH_NO_RESULTS];
}

/// What a [`CatalogFile`] contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogKind {
    /// A short-string RON table (`editor/<domain>.ron`).
    Editor,
    /// A long-form Markdown page (`docs/<domain>/<page>.md`).
    Doc(&'static str),
}

/// One embedded file registered by a domain.
///
/// `domain` is passed to [`register_catalog`]; for [`CatalogKind::Doc`] it is
/// also the topic used by [`doc`] and the variant holds the page name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogFile {
    /// Language code of the file, for example `en-US`.
    pub language: &'static str,
    /// Whether the text is a string table or a document page.
    pub kind: CatalogKind,
    /// Embedded file contents (`include_str!`).
    pub text: &'static str,
}

/// Shared catalog embedded by this crate: only `editor/common.ron`.
const COMMON_CATALOG: &[CatalogFile] = &[CatalogFile {
    language: DEFAULT_LANGUAGE,
    kind: CatalogKind::Editor,
    text: include_str!("../../../assets/localization/en-US/editor/common.ron"),
}];

/// One documentation page after registration.
struct DocEntry {
    language: &'static str,
    topic: &'static str,
    page: &'static str,
    text: &'static str,
}

/// Parsed catalogs plus the list of registered domains.
///
/// Parsed strings are leaked once at startup so lookups can return
/// `&'static str` without holding a lock on the table.
#[derive(Default)]
struct Store {
    languages: Vec<&'static str>,
    strings: HashMap<&'static str, HashMap<&'static str, &'static str>>,
    docs: Vec<DocEntry>,
    domains: HashSet<&'static str>,
}

impl Store {
    /// Returns the canonical code for `language`, adding it on first use.
    fn canonical_language(&mut self, language: &'static str) -> &'static str {
        if let Some(code) = self
            .languages
            .iter()
            .copied()
            .find(|code| code.eq_ignore_ascii_case(language))
        {
            return code;
        }
        self.languages.push(language);
        language
    }

    fn insert_file(&mut self, domain: &'static str, file: &CatalogFile) {
        match file.kind {
            CatalogKind::Editor => {
                let code = self.canonical_language(file.language);
                match ron::from_str::<HashMap<String, String>>(file.text) {
                    Ok(parsed) => {
                        let table = self.strings.entry(code).or_default();
                        for (key, value) in parsed {
                            table.insert(
                                Box::leak(key.into_boxed_str()),
                                Box::leak(value.into_boxed_str()),
                            );
                        }
                    }
                    Err(error) => tracing::error!(
                        domain,
                        language = code,
                        %error,
                        "failed to parse localization table"
                    ),
                }
            }
            CatalogKind::Doc(page) => {
                let code = self.canonical_language(file.language);
                self.docs.push(DocEntry {
                    language: code,
                    topic: domain,
                    page,
                    text: file.text,
                });
            }
        }
    }

    fn language_code(&self, language: &str) -> Option<&'static str> {
        self.languages
            .iter()
            .copied()
            .find(|code| code.eq_ignore_ascii_case(language))
    }
}

/// Parsed catalogs. The shared catalog is registered on first access.
static STORE: OnceLock<RwLock<Store>> = OnceLock::new();

/// Keys already reported as missing, so each one warns only once.
static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Selected language, or `None` for [`DEFAULT_LANGUAGE`].
static LANGUAGE: RwLock<Option<String>> = RwLock::new(None);

fn store() -> &'static RwLock<Store> {
    STORE.get_or_init(|| {
        let mut store = Store::default();
        for file in COMMON_CATALOG {
            store.insert_file("common", file);
        }
        RwLock::new(store)
    })
}

fn read_store() -> std::sync::RwLockReadGuard<'static, Store> {
    match store().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_store() -> std::sync::RwLockWriteGuard<'static, Store> {
    match store().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Registers the compiled catalogs of one editor domain.
///
/// Call this once from the domain's startup/registration path, before the UI
/// reads any of its keys. Repeated calls for the same `domain` are ignored, so
/// the catalog is parsed and leaked only once even when several editors (or
/// tests) start in the same process.
pub fn register_catalog(domain: &'static str, files: &'static [CatalogFile]) {
    let mut store = write_store();
    if !store.domains.insert(domain) {
        return;
    }
    for file in files {
        store.insert_file(domain, file);
    }
}

/// Returns the language currently selected by the application.
pub fn language() -> String {
    match LANGUAGE.read() {
        Ok(guard) => guard.clone().unwrap_or_else(default_language),
        Err(poisoned) => poisoned
            .into_inner()
            .clone()
            .unwrap_or_else(default_language),
    }
}

/// Selects the language used by [`tr`] and [`doc`].
///
/// Unknown languages are accepted: every lookup then falls back to
/// [`DEFAULT_LANGUAGE`], which keeps an unsupported preference harmless.
pub fn set_language(language: impl Into<String>) {
    let value = language.into();
    match LANGUAGE.write() {
        Ok(mut guard) => *guard = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}

fn default_language() -> String {
    DEFAULT_LANGUAGE.to_owned()
}

fn lookup(language: &str, key: &str) -> Option<&'static str> {
    let store = read_store();
    let code = store.language_code(language)?;
    store.strings.get(code)?.get(key).copied()
}

fn warn_missing_once(key: &str, language: &str) {
    let warned = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    let mut warned = match warned.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if warned.insert(key.to_owned()) {
        tracing::warn!(
            key,
            language,
            "localization key missing; falling back to en-US or the key itself"
        );
    }
}

/// Looks up `key` for an explicit language, with the `en-US` → key fallback.
pub fn tr_for(language: &str, key: &'static str) -> &'static str {
    if let Some(text) = lookup(language, key) {
        return text;
    }
    warn_missing_once(key, language);
    lookup(DEFAULT_LANGUAGE, key).unwrap_or(key)
}

/// Looks up a short UI string for the selected language.
pub fn tr(key: &'static str) -> &'static str {
    tr_for(&language(), key)
}

fn find_doc(language: &str, topic: &str, page: &str) -> Option<&'static str> {
    let store = read_store();
    let code = store.language_code(language)?;
    store
        .docs
        .iter()
        .find(|entry| entry.language == code && entry.topic == topic && entry.page == page)
        .map(|entry| entry.text)
}

/// Returns the Markdown of `topic/page` for an explicit language.
///
/// Falls back to `en-US`, then to an empty page (with a one-time warning).
pub fn doc_for(language: &str, topic: &str, page: &str) -> &'static str {
    if let Some(text) = find_doc(language, topic, page) {
        return text;
    }
    warn_missing_once(&format!("doc:{topic}/{page}"), language);
    find_doc(DEFAULT_LANGUAGE, topic, page).unwrap_or("")
}

/// Returns the Markdown of `topic/page` for the selected language.
pub fn doc(topic: &str, page: &str) -> &'static str {
    doc_for(&language(), topic, page)
}

/// Returns `true` when `language` defines `key`.
pub fn contains_key(language: &str, key: &str) -> bool {
    lookup(language, key).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document catalog used only by the fallback tests.
    const FIXTURE: &[CatalogFile] = &[CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("page"),
        text: "# Fixture\n",
    }];

    #[test]
    fn common_keys_resolve_to_their_english_text() {
        assert_eq!(
            tr_for(DEFAULT_LANGUAGE, keys::SEARCH_NO_RESULTS),
            "No results"
        );
    }

    #[test]
    fn every_key_used_in_shared_code_exists_in_english() {
        for key in keys::ALL {
            assert!(
                contains_key(DEFAULT_LANGUAGE, key),
                "missing en-US translation for `{key}`"
            );
        }
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(
            tr_for("fr-FR", keys::SEARCH_NO_RESULTS),
            tr_for(DEFAULT_LANGUAGE, keys::SEARCH_NO_RESULTS)
        );
    }

    #[test]
    fn unknown_key_falls_back_to_itself() {
        assert_eq!(tr_for("en-US", "common.unknown.key"), "common.unknown.key");
    }

    #[test]
    fn language_codes_are_case_insensitive() {
        assert_eq!(
            tr_for("EN-us", keys::SEARCH_NO_RESULTS),
            tr_for("en-US", keys::SEARCH_NO_RESULTS)
        );
    }

    #[test]
    fn documents_fall_back_to_english() {
        register_catalog("i18n-fixture", FIXTURE);
        assert_eq!(doc_for(DEFAULT_LANGUAGE, "i18n-fixture", "page"), "# Fixture\n");
        assert_eq!(doc_for("fr-FR", "i18n-fixture", "page"), "# Fixture\n");
    }

    #[test]
    fn missing_document_is_empty() {
        assert_eq!(
            doc_for(DEFAULT_LANGUAGE, "i18n-fixture", "missing-page"),
            ""
        );
    }

    #[test]
    fn language_preference_defaults_to_english() {
        // Keep the global preference untouched for other tests.
        let previous = language();
        set_language(DEFAULT_LANGUAGE);
        assert_eq!(language(), DEFAULT_LANGUAGE);
        set_language(previous);
    }
}