---
id: T-0211
title: Apply the Level Craft Example UI reference across the editor chrome and panels
status: todo
owner: -
role: shell
scope: [crates/lc_canvas, crates/lc_shell, crates/lc_feature_level, crates/lc_feature_layers, crates/lc_feature_tileset, crates/lc_feature_entities, crates/lc_feature_world, crates/lc_feature_enums, docs/tasks/T-0211-editor-ui-refresh.md, docs/MANUAL_CHECKS.md, docs/03-ui-layout.md, docs/decisions/0015-ui-refresh.md]
depends_on: [T-0201, T-0202, T-0204, T-0205]
skills: [incremental-implementation, test-driven-development, source-driven-development]
---

## Goal
Turn the visual foundation from T-0201/T-0202 into the actual editor skin shown in
`Level Craft Example UI.figcraft`: a 42 px app header with document context and
primary actions, a 44 px vertical tool rail, 38 px panel headers with icons and
tags, reference-style layer rows, tileset palette, canvas rulers, entity cards,
inspector cards, console tabs and a compact status bar. Layout behaviour, docks,
commands and data contracts stay unchanged.

Reference artboards (extracted from the `.figcraft` zip, page `Editor Workspace &
Dynamic Tileset`): tile mode, IntGrid mode, entities mode, auto-layer mode, Zen
mode and bottom-docked mode. The shared palette is `#121316` background,
`#1c1e24` panels, `#16181d` headers, `#2d3139` hairlines, `#2d81ff` accent,
`#3fd6b8` success, Inter 7.5-11 px body text and mono HUD labels.

## Acceptance criteria
- [ ] `lc_canvas::theme` exposes the reference surface tokens (`header`, `row`,
  `ink_dim`, `ink_soft`, `ink_body`, layer-kind colours) and metrics (`HEADER_H`
  42, `PANEL_HEADER_H` 38, `SIDE_RAIL_W` 44, `TAB_H` 30); existing tokens keep
  their meaning and both palettes stay readable.
- [ ] `lc_canvas::widgets` provides reusable `panel_header`, `card`, `meta_row`,
  `segmented` and `icon_toggle` primitives in the reference style, with headless
  render tests.
- [ ] The shell top bar matches the reference header: logo mark, `Level Craft`
  wordmark, project/document context, Save action, Zen toggle, theme picker and
  compact File/Edit/View menus. Every existing menu action and shortcut still
  works.
- [ ] A right-hand 44 px tool rail focuses/opens Project, Layers, Tilesets,
  entity definitions, Project Settings and Zen. The active entry uses the accent
  outlined square; the rail hides in Zen mode.
- [ ] Zen mode hides the side docks, rail and bottom dock and can be exited from
  the header or with `Esc`; it does not change document or selection state.
- [ ] Side and bottom dock tabs use the reference 38 px header (icon, upper-case
  title, optional tag/action), and the active tab keeps an accent marker.
- [ ] The Layers panel uses reference rows: kind icon, name, meta line
  (`Visible • 100% • Locked: Off`), right-aligned eye toggle, selected row with
  an accent hairline and a footer `+ Add Layer` action; reorder/edit/delete and
  context menus remain reachable.
- [ ] The Tile Palette panel uses a `TILESET PALETTE` header with the active
  layer tag, a tileset source well, segmented 1x/2x/4x zoom, grid toggle, stamp
  properties card (tile ID, size, selection extent) and icon tool buttons; drag
  selection, copy/paste and all paint tools still work.
- [ ] The level canvas draws 16 px top/left rulers with viewport-aware tick
  labels and a level title overlay (`name (WxH px) • zoom`), and the floating
  inspector uses the card/header style.
- [ ] The Entities panel groups definitions by tag with count badges and cards
  (icon box, name, id/usage meta, placing pill, Place action); the entity
  inspector renders as header + property cards.
- [ ] Project panel, workspace empty state, document pages, console filter tabs
  and status bar use the same header/typography/surface language; the console
  keeps search, dedupe, follow, copy and clear behaviour.
- [ ] `docs/03-ui-layout.md` and `docs/MANUAL_CHECKS.md` describe the new chrome
  and list the desktop checks (header actions, rail, Zen, rulers, panels).
- [ ] `cargo xtask check` is green (fmt, clippy `-D warnings`, workspace tests,
  architecture lint).

## Out of scope
- Replacing egui/eframe/egui_dock, changing the contract crates, or adding a
  dependency (no new crate is needed).
- OS-level frameless window controls; `eframe` keeps native decorations. The
  in-app header only styles the client area. The frameless title bar remains
  T-0210.
- New editor features (run/play runtime, auto-layers, preferences, export).
- Pixel-perfect reproduction of the mock artboards; the reference supplies the
  visual language and structure, not exact window-control chrome.
- Cross-level clipboard or OS clipboard.

## Notes
- The reference is a FigCraft document: a ZIP whose `document.json` holds node
  trees and geometry. Use it read-only; do not vendor the file or its images.
- `lc_canvas` tokens/widgets are the only shared styling layer; feature crates
  must not reach into `lc_shell`.
- Keep the diff focused on presentation. Where a reference element has no
  behaviour (window min/max/close, Run Level), omit it rather than adding a dead
  control; record the omission here.
- Zen mode is shell view state only (`ShellApp`), never project state.
- Manual screenshots are not available to the agent; list every visual check in
  `docs/MANUAL_CHECKS.md` and cover layout math/widget rendering with tests.
