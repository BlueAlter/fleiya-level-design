# Tilesets

A **tileset** is a sliced image (atlas) containing graphical tiles used for
painting levels, decorating terrain, and defining tile animations.

Each tileset definition specifies:
- a unique identifier,
- a **source image** relative to the project root,
- uniform **tile dimensions** (width and height in pixels),
- optional border padding and inter-tile spacing,
- optional **tags**, a **marking enum**, and **tile animations**.

## Create a Tileset

1. Press **+ New Tileset** in the left panel.
2. Select a PNG image from your project folder.
3. Configure the tile width, height, and spacing.

## Atlas Viewport

The central workspace displays the source atlas with an exact-aspect preview:
- **Pan**: Middle-click drag, right-click drag, or `Space` + drag.
- **Zoom / Fit**: `Ctrl` + wheel to zoom; double-click or press `F` to fit.
- **Inspect**: Hover over any tile to view coordinates, tags, and data.
- **Select**: Left-click a tile to focus its attributes in the right panel.

## Tool Modes

Switch tools using the selector next to the **Tile Atlas** header:
- **None**: Direct inspection and custom data editing. Left-click opens the **Custom Data** modal for JSON or text; right-click clears custom data.
- **Marking**: Left-click applies the active enum tag; right-click removes it.
- **Animation**: Creates frame-based animated tiles. Anchors cycle frames in real time on the canvas and in the preview card.
- **Collision**: Defines geometric collision boundaries (rectangles, polygons) on individual tiles with snapping, one-way flags, tags, and custom shape payloads.

## Tile Animations

Tile animations allow any tile on the atlas to act as an **anchor** that cycles through a sequence of **frame tiles**. Anchors animate in real time on the atlas canvas and in the docked preview card.

### Canvas Shortcuts
- **Ctrl + Left-Click**: Create a new animation anchor from the clicked tile.
- **Ctrl + Right-Click**: Delete the animation anchor on the clicked tile.
- **Left-Click**: Append the clicked tile as a frame to the currently selected anchor.
- **Right-Click**: Remove the clicked tile from the selected anchor's frame list.
- **Left-Click + Drag**: Box-select a rectangular region to append all enclosed tiles as frames.
- **Right-Click + Drag**: Box-select a rectangular region to remove all enclosed tiles from frames.

### Animation Toolbar
The toolbar appears in the header row when the Animation tool is active:
- **Play / Pause**: Toggle real-time animation playback across the atlas and preview card.
- **Playback Mode**: Select between `Loop` (continuous cycle), `Ping-Pong` (alternates forward and backward), and `Once` (plays once and stops on the last frame).
- **Duration (ms)**: Milliseconds each frame is displayed before advancing.
- **Random Start**: Desynchronizes animation phase to prevent identical adjacent tiles from animating in lockstep.
- **Preview Toggle**: Toggles the floating preview card docked at the bottom-left corner of the atlas.

### Tool Attributes Panel
The right panel provides dedicated inspectors for animation data:
- **Anchors List**: Lists all defined animation anchors with thumbnail previews, tile IDs, and frame counts. Click any anchor to select it. Click `+` to create a new anchor from the selected tile.
- **Frames List**: Shows the ordered frames (`F1`, `F2`, ...) for the selected anchor. Use the Chevron Up/Down buttons to reorder frames, and the trash button to remove a frame. Click `+` to add the selected tile as a frame.

## Tile Collision

Collision mode allows defining exact physics boundaries (hitboxes) per tile using rectangles or polygons.

### Canvas Shortcuts & Isolate Focus Mode
- **Alt + Left-Click**: Focus and isolate the clicked tile. Non-focused tiles are dimmed, activating an accent frame, sub-grid guide, and interactive drawing tools.
- **Escape**: Exit isolate focus mode and return to full atlas view.
- **G**: Cycle grid snapping between `None` (freeform), `Pixel` (1px precision), and `TileSize` (half/full tile).
- **V / B / P**: Switch active collision tool (`V` = Select, `B` = Box, `P` = Polygon).
- **Delete / Backspace**: Delete the currently selected shape.
- **Ctrl + C**: Copy collision shapes from the active tile.
- **Ctrl + V**: Paste copied collision shapes onto the active tile (or batch paste across multi-selected tiles).

### Collision Tools
- **Select (V)**: Click any shape to select and inspect its properties in the right panel. Selected shapes display yellow accent outlines with corner resize handles.
- **Box (B)**: Click and drag within the isolated tile to draw an aligned rectangle boundary. Snapping applies dynamically to bounds.
- **Polygon (P)**: Click to place successive vertices. Double-click, press `Enter`, or click within 12px of the starting vertex to close the loop.

### Collision Toolbar & Properties Panel
The right panel provides tools, presets, and individual shape parameters:
- **Presets**: Quick-assign standard shapes: `Full Box`, `Slope /` (45° incline), `Slope \` (45° decline), and `Clear`.
- **Shape List**: Lists every shape on the tile with its geometry summary, `Tag` text input, `1-Way` collision checkbox, and `[{ }]` custom data code editor.
- **Copy & Paste**: Duplicate shapes across tiles individually or in bulk using the toolbar buttons or standard `Ctrl+C` / `Ctrl+V` shortcuts.

## Safety

- All edits, frame reordering, and attribute changes support **Ctrl+Z** undo and **Ctrl+Y** redo.
