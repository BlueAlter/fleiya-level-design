# ADR-0014 — Phosphor icon set for the editor

## Status
ACCEPTED

## Context
The design system in ADR-0013 defines typography, colour tokens and controls,
but every icon is still either a Unicode symbol or a hand-painted primitive
(`close_icon`, `chevron`). That does not scale to the inspector, layers,
tileset palette and tool chrome planned for Phase B, and hand-drawn glyphs drift
between panels.

Constraints:

- The project runs egui `0.36`.
- No new runtime dependency without an ADR.
- Feature crates may depend on `lc_canvas` but not on `lc_shell` or each other.
- The icon set must be usable in HUD/mono labels, not only in proportional text.
- Headless tests must keep working without a GPU or bundled fonts.

## Decision
1. Use **Phosphor Icons** through `egui-phosphor = "0.14"` (regular weight).
   - Crate: MIT OR Apache-2.0.
   - Icon artwork: MIT.
   - `0.14.x` targets egui `^0.36`, matching the workspace.
2. Keep the dependency inside `lc_canvas`. Feature crates use
   `lc_canvas::icons::Icon`, never the upstream crate.
3. Expose a semantic `Icon` enum rather than raw Phosphor constant names, so
   the pack can be replaced later without touching feature crates.
4. Register the icon font as the named family `"phosphor"` and append it to
   every text family (`Proportional`, `Monospace`, and the named Inter/Archivo/
   Geist families). This keeps inline icons working in HUD and console text.
5. Fall back to egui's proportional family when `install_fonts` has not run,
   matching the existing `theme::*_for` helpers used by headless tests.
6. Ship the MIT notices for Phosphor and `egui-phosphor` under
   `assets/fonts/`, and record them in `ATTRIBUTION.md`.

## Consequences
- The binary grows by roughly the regular Phosphor font; other weights are not
  compiled in. If size becomes a problem, enable the crate's `subset` feature
  and list the used icons.
- Icon names become a UI contract of sorts: semantic variants are stable, while
  the underlying Phosphor glyph can change in one place.
- Tests assert that every icon has a glyph and that rendering does not panic;
  they deliberately do not compare rasterised glyphs.
- `egui-phosphor` is the only icon dependency. Switching to another pack means
  editing `lc_canvas/src/icons.rs` and one font registration call.
