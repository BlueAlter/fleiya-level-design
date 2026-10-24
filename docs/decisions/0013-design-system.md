# ADR-0013 — Editor design system and font pipeline

## Status
ACCEPTED

## Context
The shell currently uses egui's default visual language: rounded controls,
default spacing, default fonts and hard-coded status colours. We want the
editor chrome to match the ArtCraft Launcher visual language without replacing
egui or the existing dock architecture.

Constraints:

- `lc_core`, `lc_io` and `lc_ui_api` are contract-frozen.
- Feature crates may depend on `lc_canvas` but not on `lc_shell`.
- No new runtime dependency without an ADR.
- The editor must stay denser than the launcher; launcher-sized controls would
  waste canvas space.

## Decision
1. Put the design system in `lc_canvas`:
   - `theme.rs` — colour tokens, egui `Visuals`, typography and font loading.
   - `widgets/` — reusable primitives and controls.
   - `grid.rs` — canvas grid, checkerboard, selection and handle helpers.
2. Use zero corner radius, 1 px hairlines, flat panels, and shadows only for
   menus, dialogs and toasts.
3. Use Inter for body text, Archivo for display text, and Geist Mono for HUD
   labels, buttons and the console.
4. Use editor density tokens (`CONTROL_H = 28`, `ROW_H = 24`, `TAB_H = 28`,
   `TOP_BAR_H = 40`, `STATUS_H = 24`) instead of the launcher's larger values.
5. Ship dark and light token sets; default to dark until light-mode QA is done.
6. Bundle the three variable fonts under `assets/fonts/` with their OFL files.

## Consequences
- `lc_canvas` grows beyond pure canvas helpers, but remains the only shared UI
  utility crate that feature crates are allowed to use. If it grows past roughly
  1,000 lines, extract a dedicated `lc_ui_kit` crate in a follow-up ADR.
- Fonts add about 1.7 MB to the binary.
- `Context::set_fonts` takes effect on the next frame, so the app needs a
  one-frame startup guard and `clear_color` to avoid a white flash.
- Custom title-bar chrome is deliberately out of scope for this ADR.
