---
id: T-0204
title: Phosphor icon set, icon font pipeline and semantic Icon API
status: done
owner: opencode
role: design
scope: [Cargo.toml, Cargo.lock, crates/lc_canvas, crates/lc_shell/src, assets/, ATTRIBUTION.md, docs/decisions/0014-icon-library.md, docs/tasks/T-0204-icon-system.md, docs/MANUAL_CHECKS.md]
depends_on: [T-0201]
skills: [incremental-implementation, test-driven-development, source-driven-development, documentation-and-adrs]
---

## Goal
Give the editor a real icon set instead of Unicode symbols and hand-painted
glyphs. Bundle Phosphor through `egui-phosphor` inside `lc_canvas`, expose a
semantic `Icon` enum, and make icons resolve in proportional, HUD/mono and
named font families.

## Acceptance criteria
- [x] `egui-phosphor = "0.14"` is a workspace dependency used only by `lc_canvas`.
- [x] `lc_canvas::icons` exposes a semantic `Icon` enum, `Icon::ALL`, `glyph()`,
      `name()`, `rich()`, `paint_icon()` and `family()`.
- [x] `theme::install_fonts` registers the `"phosphor"` family and appends it to
      every text family used by the editor.
- [x] `icon_button`, `chevron` and `close_icon` use the icon font.
- [x] Project settings shows an inspectable icon gallery.
- [x] MIT notices for Phosphor and `egui-phosphor` are bundled and attributed.
- [x] Headless tests cover glyph presence, fallback family and gallery rendering.
- [x] `cargo xtask check` is green.

## Out of scope
- Replacing every ad-hoc Unicode glyph in feature pages; those follow when the
  panels that own them land (T-0102, T-0104, T-0107, T-0111).
- Additional Phosphor weights (fill/bold) and font subsetting.
- A user-configurable icon theme.

## Notes
- Phosphor constants come from `egui_phosphor::regular`.
- The font pipeline already resolved named families before `ctx.set_fonts`;
  icons follow the same headless fallback rule.
- ADR: `docs/decisions/0014-icon-library.md`.

## Progress
- Added the workspace dependency and `lc_canvas::icons`.
- Registered the icon font in `theme::install_fonts`.
- Converted `icon_button`, `chevron` and `close_icon` to the icon font.
- Added an icon gallery to the project settings page.
- Added attribution and licence files.
