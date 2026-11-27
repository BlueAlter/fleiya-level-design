# ADR-0018 — Workspace rail SVG icons via `egui_extras/svg`

## Status
ACCEPTED

## Context
The vertical left workspace switcher rail (`lc_shell::features::editor::app_mode`)
previously rendered Phosphor font glyphs via `lc_canvas::icons::Icon`. Custom
vector icons for each workspace (`Editor.svg`, `Layers.svg`, `Entity.svg`,
`Tileset.svg`, `Enum.svg`, `Setting.svg`) have been authored under
`assets/Icons/`.

Constraints:
- The workspace runs `egui = "0.36.0"` and `egui_extras = "0.36.0"` (see `Cargo.lock`).
- Golden Rule 8: no new dependency (including enabling transitive `resvg` via Cargo features) without an ADR.
- Architecture boundaries: icon presentation for the shell rail lives in `lc_shell` (never in `lc_core` or `lc_io`).
- The source SVGs under `assets/Icons/` use `#000000` fills, while `egui::Image::tint` multiplies RGBA values.
- Headless tests (`cargo test`) must render without touching disk at runtime or requiring a GPU.

## Decision
1. Enable the `"svg"` feature on `egui_extras` in root `[workspace.dependencies]`:
   `egui_extras = { version = "0.36.0", features = ["svg"] }`.
   This pulls `resvg = "0.45.1"` (already sharing `tiny-skia` in `Cargo.lock`) and
   registers `SvgLoader` when `egui_extras::install_image_loaders` is called.
2. Keep `egui_extras` restricted to `lc_shell` (enforced by workspace Cargo manifests).
3. Embed the six workspace rail SVGs (`Editor.svg`, `Layers.svg`, `Entity.svg`,
   `Tileset.svg`, `Enum.svg`, `Setting.svg`) at compile time with `include_str!`
   in `lc_shell::features::editor::app_mode`, normalizing `#000000` fills to
   `#ffffff` once per icon via `OnceLock<Arc<[u8]>>` so `egui::Image::tint`
   recolors them accurately to theme tokens (`t.invert_fg`, `t.ink_strong`,
   `t.muted`, `t.faint`) without modifying the source `.svg` files.
4. Call `egui_extras::install_image_loaders(ui.ctx())` inside
   `render_workspace_rail` (idempotent in `egui_extras`), so both `ShellApp`
   and standalone rail tests render SVG icons cleanly.

## Alternatives considered
- **Pre-rasterizing SVGs to PNGs.** Rejected: loses DPI-independent crispness on
  scaled displays and requires regenerating PNG files whenever `.svg` artwork changes.
- **Replacing `#000000` directly inside `assets/Icons/*.svg`.** Rejected: keeps
  source art unreadable on light backgrounds when previewed outside the app.

## Consequences
- `Cargo.lock` gains `resvg` (`0.45.1`) and `usvg` dependencies inside `lc_shell`.
- Rail buttons render crisp, DPI-aware custom SVG artwork tinted to the active
  theme without runtime filesystem reads.
