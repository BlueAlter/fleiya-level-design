---
id: T-0201
title: Design-system foundation: tokens, fonts, widgets, grid and shell chrome
status: in_progress
owner: -
role: shell
scope: [crates/lc_canvas, crates/lc_app, crates/lc_shell/src, assets/, ATTRIBUTION.md, docs/decisions/0013-design-system.md, docs/tasks/T-0201-design-system-foundation.md, docs/MANUAL_CHECKS.md]
depends_on: [T-0008]
skills: [incremental-implementation, test-driven-development, source-driven-development, documentation-and-adrs]
---

## Goal
Introduce the editor design system in `lc_canvas`, wire it into the app, and
apply it to the shared shell chrome: top bar, status bar, side/bottom docks,
project panel, console and document tabs. The shell layout and behaviour stay
the same; only the visual system changes.

## Acceptance criteria
- [x] `lc_canvas::theme` exposes `Tokens`, `ThemeChoice`, `apply` and `install_fonts`.
- [x] Dark and light `Visuals` use zero radius, 1 px hairlines, flat panels, and shadows only for menus/windows.
- [x] Inter, Archivo and Geist Mono variable fonts are bundled with OFL files and attribution.
- [x] `lc_app` installs fonts/theme, sets a sensible viewport size, and avoids a white first frame.
- [x] `lc_canvas::widgets` provides HUD text, rules, tags, buttons, tabs, tree rows, chips and progress.
- [x] `lc_canvas::grid` draws a zoom-aware grid, checkerboard, selection and handles.
- [x] Unit tests cover token/visual application, font installation, and grid drawing.
- [x] The shared shell chrome uses the new tokens/widgets and `egui_dock` is styled from them.
- [ ] `cargo xtask check` is green. The architecture-lint gate is blocked by the
  pre-existing untracked `crates/lc_app_slint` package using `serde_json`; fmt,
  clippy (`-D warnings`) and tests pass for the touched crates.

## Out of scope
- Custom frameless title bar (planned as T-0210).
- Light-mode visual QA on a real display.
- Restyling feature-specific document views and dialogs beyond the shared shell
  chrome; those follow in later tasks.
- Replacing egui/eframe/egui_dock or changing the shell layout/behaviour.

## Notes
- ArtCraft Launcher is MIT OR Apache-2.0; its brand assets are not used.
- Fonts are SIL OFL 1.1; see `ATTRIBUTION.md`.
- `lc_canvas` is used because feature crates are allowed to depend on it and
  contract crates must not be modified.
- Headless tests render without installing the bundled fonts. `theme::*_for`
  helpers fall back to egui's proportional/monospace families until
  `install_fonts` has run, so tests do not panic on unbound families.
- `apply` resolves font families before `all_styles_mut`; calling context-data
  helpers inside that closure deadlocks on the context write lock.

## Progress
- Added `lc_canvas/src/theme.rs` with Dark/Light tokens, `Visuals`, typography,
  spacing, variable-font installation and tests.
- Added `lc_canvas/src/widgets/` primitives and controls (HUD text, rules, tags,
  buttons, tabs, tree rows, chips, progress) and `lc_canvas/src/grid.rs`.
- Bundled Inter, Archivo and Geist Mono with OFL licences; added attribution.
- Added `docs/decisions/0013-design-system.md`.
- Wired fonts/theme/viewport into `lc_app`; added a one-frame font guard with a
  repaint request.
- Added `lc_shell/src/dock_style.rs` and applied it to `DockArea`.
- Restyled the top bar, status bar, side/bottom docks, project panel, console and
  document tab titles/context menu.
- Verified a headless Xvfb screenshot of the dark theme; see
  `docs/MANUAL_CHECKS.md` for the remaining human checks.
