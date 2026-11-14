# Localization assets

Each language lives in its own folder: `assets/localization/<lang>/`.

- `editor/<domain>.ron` — short UI strings, one `key = text` map per domain.
- `docs/<topic>/<page>.md` — long-form Markdown documents rendered in DOCUMENT tabs.
- Keys use `<domain>.<section>.<item>`, for example `enums.tab.definition`.
- `en-US` is the source of truth and the only language edited by hand in code changes.
- Add a language by copying `en-US` and translating every key; never rename `en-US` keys.
- Missing keys/pages fall back to `en-US`, then to the key itself (a warning is logged once).

## Registration

`lc_ui_api::i18n` owns the lookup and embeds only `editor/common.ron` (labels
shared by more than one domain, such as the search picker's empty result). A
domain owns its own files and registers them from its feature crate at startup:

```rust
use lc_ui_api::i18n::{register_catalog, CatalogFile, CatalogKind, DEFAULT_LANGUAGE};

pub const CATALOG: &[CatalogFile] = &[
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Editor,
        text: include_str!("../../../assets/localization/en-US/editor/enums.ron"),
    },
    CatalogFile {
        language: DEFAULT_LANGUAGE,
        kind: CatalogKind::Doc("overview"),
        text: include_str!("../../../assets/localization/en-US/docs/enum/overview.md"),
    },
];

pub fn register() {
    register_catalog("enum", CATALOG);
}
```

`register_catalog` is called once per domain; repeated calls are ignored. The
domain test must assert that every key used in Rust exists in `en-US`.