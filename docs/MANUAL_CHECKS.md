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

## Dock Interaction Polish (T-0203)
Headless Xvfb screenshots inspected at `/tmp/opencode/dock_reexpanded.png`,
`dock_drag_ghost2.png`, `dock_after_drop_right.png`,
`dock_reorder_indicator.png`, `dock_central_nodrop.png` and
`dock_max_height4.png`.
- [x] Expanding the Console after a collapse restores its remembered ~180 px height instead of filling the editor.
- [x] Drag the bottom-dock divider to the top: the expanded height stops at 55% of the central area and never covers the document area.
- [x] Dragging a panel tab shows a floating uppercase tab preview at the cursor.
- [x] Hovering a valid dock target highlights the slot with an accent hairline and shows a 2 px insertion line with end caps.
- [x] Dropping on a tab strip inserts at the indicated line; dropping on the slot content appends.
- [x] Same-slot reordering works (e.g. drag `PROJECT` before `CONSOLE` in the bottom dock).
- [x] Cross-slot moves work (e.g. `PROJECT` to the right or bottom dock) and update the status bar.
- [x] Dropping over the central document area shows a `NoDrop` cursor, no slot highlight, and cancels on release.
- [ ] Press `Esc` during a drag: the ghost and indicator disappear and the panel stays in its original dock.
- [ ] Drag a panel onto the collapsed bottom header: the dock expands and accepts the panel at the indicated index.
- [ ] Resize the window to 960x600 and repeat expand/collapse: the bottom dock stays within 55% and the Console toolbar still fits.

## Icon Set (T-0204)
- [ ] Project Settings shows an `ICON SET` gallery; every tile renders an icon and its semantic name without missing-glyph boxes.
- [ ] Dock collapse chevrons render the Phosphor caret at the same optical size as before the change.
- [ ] Close buttons in document tabs and panels render a crisp `x` glyph, not a painted line pair.
- [ ] Icons remain visible in HUD/mono labels (dock titles, status bar, console) and in proportional text.
- [ ] Light theme: icon ink uses the token colour and stays readable on panel/canvas surfaces.
- [ ] At 960x600 the icon gallery wraps and does not overflow the page width.
- [ ] Headless test mode (no installed fonts) does not panic and falls back to the proportional family.

## Level Tab: Canvas Host, Pan/Zoom & Inspector (T-0101)
- [ ] Opening a level from the Project panel opens a dedicated `LevelDocumentView` tab.
- [ ] Middle-mouse drag pans the canvas smoothly across world coordinates.
- [ ] Holding Space and dragging with primary mouse button pans the canvas.
- [ ] Mouse wheel zooms in and out focused on the current cursor position.
- [ ] Zoom buttons (+ / - / Fit) in the canvas overlay toolbar adjust zoom and center the level.
- [ ] The 16 px grid and level boundaries render crisp hairline strokes.
- [ ] Toggling grid snap and grid visibility via the overlay toolbar updates canvas drawing.
- [ ] Visible layers inside the level bounds are rendered; moving viewport completely away culls offscreen layers.
- [ ] The floating properties window is draggable by its header and clamps to canvas boundaries on resize.
- [ ] Clicking the collapse chevron on the floating window collapses the body to just the header bar.
- [ ] Clicking the pin button prevents dragging while pinned.

## Tileset Definition Tab & Tile Palette (T-0102)
- [ ] Opening a tileset opens a dedicated `TilesetDocumentView` document tab with name, asset path, tile size, spacing, and padding editors.
- [ ] Editing name, asset path, or dimensions dispatches an undoable `UpdateTilesetDefCommand`.
- [ ] Slicing preview renders the loaded texture with checkerboard background and overlay tile slice grid lines.
- [ ] Hovering over tiles in the preview highlights the hovered tile cell and displays its tile ID in the footer.
- [ ] Missing or invalid asset paths gracefully display an informative placeholder rather than panicking.
- [ ] The Tile Palette panel shows a tileset dropdown selector when multiple tilesets exist.
- [ ] Clicking a tile in the palette selects a single tile and emits `SetTileSelection(Some(TileSelection::single))`.
- [ ] Dragging across tiles in the palette selects a rectangular block and emits `SetTileSelection(Some(TileSelection::rect))`.
- [ ] Selected tiles render an accent outline and semi-transparent fill highlight in the palette.
- [ ] The "Clear" button in the palette toolbar deselects the current tile selection.
- [ ] Deleting a tileset cascades properly through referencing layer defs, enum values, and tile layers, and undo restores all references identically.

## Layer Definition Tab & Layers Panel (T-0104)
- [ ] Opening a layer definition opens a dedicated `LayerDefDocumentView` tab showing name, layer kind, grid size, tileset link, and IntGrid value editors.
- [ ] Editing layer definition name, kind, or grid dimensions dispatches an undoable `UpdateLayerDefCommand`.
- [ ] For IntGrid layers, adding/removing values or picking colors in the table updates definition and reverts cleanly.
- [ ] Layers panel displays an empty state ("No active level") when no level tab is active (`cx.active_level()` is None).
- [ ] When a level tab is active, the Layers panel lists its layer instances in top-to-bottom visual order (matching canvas render order).
- [ ] Clicking a layer row selects the layer, emits `SetActiveLayer(Some(id))`, and highlights the row with accent styling.
- [ ] Double-clicking a layer row opens the layer definition document view tab.
- [ ] Clicking the eye icon toggles layer visibility via `ToggleLayerVisibilityCommand` and updates the icon.
- [ ] Clicking "+ Add Layer" allows adding a new Tiles, IntGrid, or Entities layer, which instantiates layer instances across levels and selects the new layer.
- [ ] Moving layers up (▲) or down (▼) reorders project definitions and synchronizes level render order via `ReorderLayerDefsCommand`.
- [ ] Deleting a layer (🗑) cascades removal from the project definitions and all level layer instances with complete revert support.

## Tile and IntGrid Layer Painting (T-0103)
- [ ] In the Tile Palette panel, tool buttons (Paint, Erase, Rectangle, Fill, Eyedropper) switch the active tool and highlight the active button.
- [ ] With a Tiles layer active and a tile selected in the palette, clicking or dragging on the canvas stamps tiles with `PaintTilesCommand`.
- [ ] Dragging continuously across frame boundaries merges into a single undo entry; pressing `Ctrl+Z` reverts the entire stroke at once.
- [ ] Erase tool clears tiles to empty / sets IntGrid cells to 0, also merging continuous drags into a single undo entry.
- [ ] Rectangle tool drags from origin to destination and fills the rectangular region on release.
- [ ] Flood fill tool fills all contiguous matching cells iteratively without recursion overflow.
- [ ] Pressing `X` or `Y` key toggles horizontal / vertical flip of the placed tile.
- [ ] Eyedropper tool clicks any placed tile on the canvas and picks it into the tile palette selection.
- [ ] Viewport culling ensures only visible grid cells inside the canvas viewport are rendered.

## Entity Definition Tab & Field Editors (T-0106)
- [ ] Opening an entity definition in the Project panel opens an `EntityDefDocumentView` tab with name, colour, size, pivot, and tile controls.
- [ ] Editing metadata dispatches an undoable `UpdateEntityDefCommand`; one continuous drag is a single undo step.
- [ ] "Use Palette Tile" adopts the palette's primary tile; "Clear" removes it, both undoable.
- [ ] Adding a field with a valid unique name dispatches `AddEntityFieldCommand` and seeds the default on every existing instance.
- [ ] Duplicate or empty field names are rejected with an inline message and no command is emitted.
- [ ] Renaming a field or changing its type updates the definition and migrates instance values; undo restores both schema order and values.
- [ ] Changing a field type also rewrites its stored default to the new type (e.g. Int 10 -> String "10").
- [ ] Every field type renders a usable editor: Int, Float, Bool, String, Color, Enum, Point, EntityRef, and Array (add/remove items).
- [ ] Deleting a field removes it and its values; undo restores them.
- [ ] Deleting an entity definition is blocked while instances exist; "Force cascade" removes instances, clears dangling EntityRef values (instances, level fields, field defaults, arrays), and undo restores everything.

## Entity Layer, Tools & Floating Inspector (T-0107)
- [ ] With an Entities layer active, entity instances render as coloured rectangles at their level-local positions with pivot dots and name labels.
- [ ] The right/left dock shows an "Entities" panel listing every entity definition with its `#uid` and placed-instance count.
- [ ] Clicking "Place" in the Entities panel switches the tool to `Place #<uid>`; the status bar reads "Place Entity".
- [ ] Clicking the canvas while placing adds one instance snapped to the layer grid, selects it, and the Inspector shows its fields.
- [ ] Clicking an entity with the Select tool selects it; Shift-click toggles it in a multi-selection; clicking empty canvas clears the selection.
- [ ] Dragging a selected entity moves the whole selection snapped to the grid; one drag is a single undo step.
- [ ] Dragging the bottom-right corner of a single selected entity resizes it snapped to the grid; one drag is a single undo step.
- [ ] Delete/Backspace removes the selected entities; the selection clears and any `EntityRef` values (instances, level fields, field defaults, arrays) become `None`.
- [ ] Ctrl/Cmd+D duplicates the selection with new ids offset by one grid cell and selects the copies.
- [ ] The Inspector shows POSITION/SIZE for a single selection and edits them through move/resize commands.
- [ ] The Inspector lists all fields of a single selection; for multiple entities it shows only fields with the same name and type, marking differing values "(mixed)".
- [ ] Typing in a text field or dragging a numeric field commits one merged undo step per focus session; moving focus away starts a new step.
- [ ] EntityRef fields list all instances by `Def #id` and resolve the stored id after undo/redo.
- [ ] Undo after placing, moving, resizing, duplicating, deleting, or editing a field restores the exact previous project state.
- [ ] Switching between tile layers and entity layers keeps the matching tool active (tile painting vs entity editing).
- [ ] Focus an inspector text field, move the pointer over the canvas without clicking, press Backspace: only the text changes; the entity is not deleted.
- [ ] Drag a numeric inspector field, pause with the pointer held still, continue the drag: one undo reverts the whole drag.
- [ ] Open two levels; select an entity in A, switch to B: B's properties show level B (not A's entity), and Shift-click in B never keeps A's ids.
- [ ] Select an entity that lives on a non-active Entities layer and press Ctrl/Cmd+D: the copy stays on the same layer as its source.

## World Tab (T-0108)
- [ ] Opening a world from the project panel shows a world tab with level rectangles at their world positions.
- [ ] The active depth is full colour; levels on other depths are dimmed and painted underneath overlapping active levels.
- [ ] Clicking the active-depth level selects it (status/inspector reflect the level); clicking empty canvas clears the selection.
- [ ] Clicking or dragging a dimmed level on another depth does nothing until the depth selector switches to that depth.
- [ ] Double-clicking an active-depth level opens or focuses its level tab.
- [ ] In Free layout, dragging a level snaps to the grid (magnet state) and one undo reverts the whole drag.
- [ ] With grid snap off in Free layout, a dragged level keeps arbitrary world coordinates.
- [ ] GridVania snaps dragged levels to the world grid (project grid fallback when the world has no default grid).
- [ ] Linear H/V drags reorder the line and re-space all levels; one undo restores every position.
- [ ] Switching the layout selector to Linear H/V auto-arranges levels; the toolbar "Tidy" button re-runs arrangement/snapping for the current layout.
- [ ] "Add" creates a level at the view centre on the active depth and keeps the world tab focused.
- [ ] "Duplicate" copies the selected level with a new id offset by one grid cell and selects the copy.
- [ ] "Move" moves the selected level to the active depth and keeps the world tab focused.
- [ ] Each world opens in its own tab with independent camera/depth state; closing and reopening fits the view again.
- [ ] Middle-drag or Space+drag pans, the wheel zooms at the pointer, and "Fit" frames every level including an empty world.

## Native File Dialogs (T-0109)
- [ ] `File > Open...` opens the OS file picker filtered to `.lcproj`; choosing a project loads it and refreshes the project panel.
- [ ] Cancelling the open picker leaves the current project unchanged and shows `File dialog cancelled`.
- [ ] `File > Save As...` opens the OS save picker prefilled with `project.lcproj`; choosing a path writes the workspace and shows the saved path.
- [ ] Omitting the extension in `Save As` writes `<name>.lcproj` and remembers the normalized path.
- [ ] Launching with `LC_DISABLE_FILE_DIALOGS=1` and choosing Open/Save As shows the headless fallback status instead of a dialog, and the editor stays responsive.
- [ ] While a native dialog is open, the editor window keeps repainting and cancelling returns control immediately.

## Multi-tile Selection, Stamping & Canvas Copy/Paste (T-0205)
- [ ] Dragging a rectangle in the Tile Palette highlights every cell in the block and the status line reports `SELECTED: <w>x<h> BLOCK`.
- [ ] With a multi-tile palette block selected, the Paint tool stamps the whole block at the cursor, preserving its row-major layout and `X`/`Y` flip flags.
- [ ] Dragging with the Rectangle tool repeats the selected block across the dragged rectangle (pattern anchored at the rectangle's top-left) instead of filling it with the first tile only.
- [ ] Choosing the new "Select" tool and dragging on a Tiles or IntGrid canvas draws a rectangular region selection (`Selection::Tiles`), and clicking selects a single cell.
- [ ] With a canvas region selected, `Ctrl+C` reports `Copied <w>x<h> tile/IntGrid region` and `Ctrl+V` at the cursor stamps the same layout into the active layer of the matching kind.
- [ ] `Ctrl+Z` after a multi-tile stamp or a paste removes the whole operation in one step.
- [ ] `Ctrl+V` with an empty clipboard, no active layer, or a layer of the wrong kind reports a clear status message and does not mutate the project.

## Shared Searchable Picker / Tag Recall Popup (feature/tileset)
- [ ] Opening the enum window tag popup places the caret in the search field (it is focused and accepts typing immediately).
- [ ] Typing filters the tag list case-insensitively; clearing the field restores every tag; a query matching nothing shows "No results" and the popup stays open.
- [ ] Rows are 32 px tall with a flat hover fill, and hovering a row moves the keyboard highlight to it.
- [ ] Arrow Down/Up move the highlight, skip group headers and disabled (greyed) rows, and stop at both ends of the list.
- [ ] Enter assigns the highlighted tag and closes the popup; Escape closes it without assigning; clicking a row assigns that tag.
- [ ] With more than five tags the list scrolls inside the popup while the search field stays fixed; group headers do not count toward the five-row cap.
- [ ] Long tag names are truncated with an ellipsis and the full text appears in the hover tooltip.
- [ ] The enum window tag popup still renders as a flat list: same width (200 px), no group header above the tags, same row height as before.
- [ ] Grouped lists (multi-tag items, used by later tasks) list "Untagged" first, then tag names A-Z case-insensitively; an item with several tags appears under each of its groups.
- [ ] The "-- None --" row, when enabled by a caller, is the first row, is keyboard-selectable and reports "none" rather than an item.
- [ ] The tileset window has not changed in this task (its enum pickers are still plain combo boxes).
