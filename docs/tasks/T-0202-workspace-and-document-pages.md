---
id: T-0202
title: Design-system: workspace empty state and document pages
status: in_review
owner: deepseek-v4.1-flash-free
role: shell
scope: [crates/lc_canvas/src/widgets/, crates/lc_shell/src/, docs/tasks/T-0202-workspace-and-document-pages.md, docs/MANUAL_CHECKS.md, docs/QUESTIONS.md]
depends_on: [T-0201]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Give the central workspace a designed empty state and turn the fallback
document tabs into real design-system pages. The shell layout, docking
behaviour and feature ownership stay unchanged; this is a visual layer over the
existing `DocumentView` placeholders.

## Acceptance criteria
- [x] With no document open, the central area shows a centred empty-state card
  over a quiet grid, with a project summary and a primary action that opens
  Project Settings.
- [x] Fallback document tabs render a page header, HUD metadata rows and a
  bordered empty state instead of raw `ui.heading` / `ui.label` output.
- [x] Level pages include a read-only canvas preview using the shared grid and
  checkerboard; the preview has no input handling or tool state.
- [x] Page primitives live in `lc_canvas::widgets` (`page`, `page_header`,
  `meta_row`, `empty_state`, `canvas_preview`) and have a headless render test.
- [x] Every `DocKind` fallback page has a headless render test.
- [x] Metadata values stay clear of the overlay scrollbar.
- [x] The console toolbar stays on one row at the default bottom-dock width,
  including three-digit filter counts.
- [x] `cargo fmt`, clippy (`-D warnings`) and tests pass for the touched crates.
- [x] `cargo xtask check` is green after the `lc_app_slint` prototype was moved
  to the `slint-app` branch.

## Out of scope
- Real feature editors and canvas hosts (T-0101–T-0108).
- File dialogs (T-0109), the frameless title bar (T-0210), and custom menu-item
  widgets beyond the existing theme-level menu styling.
- Project mutations from document pages; every page in this task is read-only.
- Replacing egui/eframe/egui_dock or changing dock layout behaviour.

## Notes
- No new dependency; the page layer uses `egui`, `lc_canvas` tokens/widgets and
  the existing `lc_ui_api::viewport::Viewport` type.
- The workspace grid is decorative. Its axis colours are remapped to
  `grid_major` so it does not look like an active canvas with world axes.
- `meta_row` reserves a 16 px gutter for the scroll area's overlay scrollbar;
  without it, short values such as `0` were hidden under the scrollbar.
- The level preview is intentionally read-only. T-0101 owns pan/zoom, selection,
  layers and tools; this task only proves the visual tokens compose.
- The fallback pages are also the place to state which task will replace them,
  so the shell never shows an unexplained blank tab.
- The console toolbar spacing was tightened after adding a world increased the
  `DBG` count enough to orphan `COPY` on a second row.
- The level-page screenshot used a temporary local fix for the pre-existing
  command-focus bug; that fix was reverted. The bug is recorded in
  `docs/QUESTIONS.md` and is not part of this visual task.

## Progress
- Added `lc_canvas/src/widgets/page.rs` with the page, header, metadata, empty
  state and read-only canvas preview primitives.
- Added `lc_shell/src/workspace.rs` and wired it into the central area when the
  dock has no tabs.
- Replaced the raw placeholder body with per-`DocKind` pages and metadata.
- Added headless render tests for the page primitives, every document kind and
  the workspace empty state.
- Verified dark-theme screenshots of the empty workspace, Project Settings and
  World pages under Xvfb; see `docs/MANUAL_CHECKS.md`.
