# Attribution

## Fonts

Level Craft bundles the following variable fonts:

- **Inter** — SIL Open Font License 1.1. See `assets/fonts/OFL-Inter.txt`.
- **Archivo** — SIL Open Font License 1.1. See `assets/fonts/OFL-Archivo.txt`.
- **Geist Mono** — SIL Open Font License 1.1. See `assets/fonts/OFL-GeistMono.txt`.

The font files are unmodified copies of the upstream variable fonts.

## Icons

The editor icon set is **Phosphor Icons** by Phosphor Icons, licensed under
MIT. Icons are consumed through the `egui-phosphor` crate, which is licensed
under MIT OR Apache-2.0. See:

- `assets/fonts/LICENSE-Phosphor.txt`
- `assets/fonts/LICENSE-egui-phosphor.txt`

Only the regular weight is currently embedded. Semantic names live in
`lc_canvas::icons`; feature crates should use those instead of upstream
codepoints.

## Visual design and widget patterns

The Level Craft editor design system is adapted from **ArtCraft Launcher**,
which is licensed under MIT OR Apache-2.0. The adaptation covers colour tokens,
typography choices, square/hairline controls, and a small number of widget
drawing patterns. No ArtCraft brand assets (logo, wordmark, app icons) are
included.
