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
  - [ ] When expanded, clicking the active tab header collapses the dock to a thin header bar showing tab titles.
  - [ ] Clicking any tab title in the collapsed header expands the dock and activates that tab.
  - [ ] Pressing `Ctrl+`` toggles between collapsed and expanded states.
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
