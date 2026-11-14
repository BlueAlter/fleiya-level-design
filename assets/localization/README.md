# Localization assets

Each language lives in its own folder: `assets/localization/<lang>/`.

- `editor/<feature>.ron` — short UI strings, one `key = text` map per feature.
- `docs/<topic>/<page>.md` — long-form Markdown documents rendered in DOCUMENT tabs.
- Keys use `feature.section.item`, for example `enums.tab.definition`.
- `en-US` is the source of truth and the only language edited by hand in code changes.
- Add a language by copying `en-US` and translating every key; never rename `en-US` keys.
- Missing keys/pages fall back to `en-US`, then to the key itself (a warning is logged once).