# Level Craft — Manual Verification Checklist

Since agents cannot view graphical UI output directly, this document enumerates visual and interactive behaviors that a human tester should verify.

## Shell Layout and Docks (T-0008)
- [ ] **Window Launch**: Application launches with top menu bar, left dock, right dock, central document area, bottom dock, and bottom status bar.
- [ ] **Menu Bar**:
  - [ ] `File > New`: Prompts/resets to a new empty project.
  - [ ] `File > Open`: File dialog opens `.lcproj` file.
  - [ ] `File > Save`: Saves project to current path; status bar updates.
  - [ ] `Edit > Undo` (`Ctrl+Z`): Reverts last command when undo stack is not empty.
  - [ ] `Edit > Redo` (`Ctrl+Y` / `Ctrl+Shift+Z`): Re-applies undone command.
  - [ ] `View > Toggle Bottom Dock` (`Ctrl+``): Expands or collapses bottom dock.
  - [ ] `View > Reset Layout`: Restores docks and central tabs to default configuration.
- [ ] **Central Document Tabs**:
  - [ ] Tab title shows asterisk `*` when project has unsaved changes.
  - [ ] Double-clicking document in tree opens or focuses tab without duplicate tabs for the same `DocKind`.
  - [ ] Tabs can be dragged to split side-by-side or stacked in the central area.
  - [ ] Tab close button (`x`) closes the tab.
  - [ ] Deleting an underlying item (e.g. level or entity def) automatically closes the tab (`is_valid == false`).
- [ ] **Panel Docks (Left, Right, Bottom)**:
  - [ ] Panels cannot be dragged or dropped into the central document area.
  - [ ] Right-clicking panel tab header allows moving panel between Left, Right, and Bottom docks.
  - [ ] Active tab selection changes rendered panel content.
- [ ] **Collapsible Bottom Dock**:
  - [ ] Dragging the bottom dock handle resizes its height between the left and right docks without overlapping side docks.
  - [ ] Collapsing with `Ctrl+`` leaves a ~28px header bar in the center column only; editor area takes the freed height.
  - [ ] Moving the Console to the left or right dock and back via context menu works smoothly and updates the respective dock layout.
  - [ ] When expanded, clicking the active tab header collapses the dock to a thin header bar showing tab titles.
  - [ ] Clicking any tab title in the collapsed header expands the dock and activates that tab.
  - [ ] Resizing the bottom dock height is remembered when collapsed and re-expanded.
- [ ] **Status Bar**:
  - [ ] Displays grid coordinates under pointer (e.g. `[12, 8]`).
  - [ ] Displays active zoom percentage (e.g. `100%`).
  - [ ] Displays active tool name (e.g. `Select`).
  - [ ] Displays dirty status indicator (`* Unsaved changes` / `Saved`).

## Console Panel (T-0009)
- [ ] Log messages from `tracing` appear in Console panel with colored level indicators.
- [ ] Level filter buttons (Error, Warn, Info, Debug, Trace) toggle visibility of respective log levels.
- [ ] Search input filters messages in real time (case-insensitive on text and target module).
- [ ] Collapse checkbox groups consecutive identical log entries with a repeat badge `(xN)`.
- [ ] Auto-scroll checkbox keeps view pinned to the newest entries, pausing when unchecked.
- [ ] Clear button clears the log buffer.
- [ ] Copy button copies all filtered log entries to the system clipboard.
- [ ] Log lines with `DocKind` targets (e.g. `Level(42)`) show a navigation button that opens/focuses that tab.
- [ ] Worker thread logging streams into the console without blocking the UI thread.
- [ ] Rotating `logs/editor.log` file is maintained on disk and rolls over upon exceeding max file size.

## Project Panel (T-0010)
- [ ] Displays tree of Worlds, Levels, Entity Defs, Enum Defs, Tileset Defs, and Layer Defs.
- [ ] Double-clicking an item opens its document view in the central area.
- [ ] Context menus on World: Open World Tab, Add Level, Rename World, Delete World.
- [ ] Context menus on Level: Open Level, Rename Level, Duplicate Level, Move to Depth, Delete Level.
- [ ] Drag-and-drop a level onto a depth group moves level depth and issues an undoable `MoveLevelCommand`.
- [ ] Project save (`File > Save` or `Ctrl+S`) writes multi-file JSON hierarchy (`project.lcproj`, `worlds/world_X/world.json`, `levels/level_Y.json`) via `lc_io`.

## Design System (T-0201)
Visual pass captured from a headless Xvfb run; the dark-theme screenshot was inspected at `/tmp/opencode/levelcraft7.png`.
- [x] Dark theme is the default: near-black surfaces, 1 px hairlines, zero corner radius.
- [x] Top bar uses the accent wordmark, mono uppercase menu labels and right-aligned theme/dirty/project HUD.
- [x] Left/right dock tabs use mono uppercase labels; the active tab is inverted with an accent underline.
- [x] Project panel uses a mono section header, tree rows and a ghost `ADD WORLD` action.
- [x] Bottom console uses filter chips, a search field, dedupe/follow toggles and mono log rows that truncate instead of overflowing.
- [x] Status bar uses mono HUD labels and a right-aligned status message.
- [ ] Light theme (`THEME` menu) has readable contrast on every surface.
- [ ] Open a level placeholder and verify the read-only canvas preview grid and checkerboard; real canvas selection/handles at 50%/100%/200% zoom belong to T-0101.
- [ ] Menus, context menus and dialogs have the intended shadow and no rounded corners.
- [ ] Dock tab drag/drop and panel move context menus still work after the restyle.
- [ ] All three bundled fonts render (no missing-glyph boxes) for Latin text and editor symbols.

## Design System: Workspace and Document Pages (T-0202)
Dark-theme screenshots inspected at `/tmp/opencode/levelcraft10.png` (empty
workspace), `/tmp/opencode/levelcraft13a.png` and `levelcraft13b.png` (Project
Settings, before/after scroll), `levelcraft21.png` (World page),
`levelcraft22.png` (Level page preview) and `levelcraft23.png` (empty state
restored after closing the last tab).
- [x] Empty workspace: centred card over a quiet 32 px grid; no red/green world axes; `OPEN PROJECT SETTINGS` opens a tab.
- [x] Empty workspace footer shows the project name and version without overflowing at the default window size.
- [x] Project Settings page: header, right-aligned metadata values, readable `0` values, and a scrollbar that does not cover values.
- [x] Project Settings empty state appears after scrolling and uses the same bordered block as the other pages.
- [x] World page: metadata and empty state fit without scrolling at the default window size.
- [x] Console toolbar stays on one row at the default bottom-dock width, including `DBG 4` and larger counts.
- [ ] Light theme: workspace card, hairline, muted text and accent tag remain legible.
- [ ] Resize to 960x600: the workspace card stays inside the central area, page text wraps, and the page scrolls when needed.
- [x] Level page: read-only canvas preview shows the checkerboard and 16 px grid; it does not pan, zoom or select.
- [x] Closing the last document returns to the workspace empty state.
- [ ] Long project names and long tileset paths truncate in `meta_row` instead of overflowing.
