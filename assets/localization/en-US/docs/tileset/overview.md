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

| Shortcut | Action | Description |
|---|---|---|
| `Ctrl + Left-Click` | Create Anchor | Creates a new animation anchor from the clicked tile |
| `Ctrl + Right-Click` | Delete Anchor | Deletes the animation anchor on the clicked tile |
| `Left-Click` | Add Frame | Appends clicked tile as a frame to the active anchor |
| `Right-Click` | Remove Frame | Removes clicked tile from the active anchor's frames |
| `Left-Click + Drag` | Box Add Frames | Enclosed tiles are appended as animation frames |
| `Right-Click + Drag` | Box Remove Frames | Enclosed tiles are removed from animation frames |

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

| Shortcut | Action | Description |
|---|---|---|
| `Alt + Left-Click` | Isolate Tile | Focuses and isolates the clicked tile with background dimming |
| `Escape` | Exit Focus | Exits isolate mode and returns to full atlas view |
| `G` | Cycle Snapping | Cycles grid snap mode (`Pixel` -> `Tile` -> `Off`) |
| `V` | Select Tool | Selects shapes and allows dragging vertex / corner handle dots |
| `B` | Box Tool | Click-and-drag to create an aligned rectangle boundary |
| `P` | Polygon Tool | Click to place points; double-click, click start dot, or `Enter` to close |
| `Delete` / `Backspace` | Delete Shape | Deletes the currently selected collision shape |
| `Ctrl + C` | Copy Collision | Copies all collision shapes from active tile to clipboard |
| `Ctrl + V` | Paste Collision | Pastes collision shapes onto active or multi-selected tiles |

### Collision Tools
- **Select (V)**: Click any shape to select and inspect its properties in the right panel. Selected shapes display yellow accent outlines with interactive handle dots on every vertex and corner. Drag handle dots with the cursor to reshape geometry freely.
- **Box (B)**: Click and drag within the isolated tile to draw an aligned rectangle boundary. Snapping applies dynamically to bounds.
- **Polygon (P)**: Click to place successive vertices. Double-click, press `Enter`, or click within 16px of the starting vertex to close the loop.

### Collision Toolbar & Properties Panel
- **Toolbar Controls**: Provides drawing tools (`Select`, `Box`, `Polygon`), quick preset icon buttons (`Full Box`, `Slope /`, `Slope \`, `Clear`), grid snapping (`Pixel` / `Tile` / `Off`), `Copy` / `Paste`, overlay visibility toggle, and the `Isolate` mode toggle.
- **Copy & Paste**: Duplicate shapes across tiles individually or across multi-tile drag selections using the toolbar buttons or `Ctrl+C` / `Ctrl+V`, both inside and outside Isolate mode.
- **Shape List & Fast Custom Properties**: Lists every shape on the active tile with its geometry summary, `Tag` input, `1-Way` collision checkbox, and an inline custom properties editor with a `+ Add` dropdown (`String`, `Int`, `Float`, `Boolean`).

## Safety

- All edits, frame reordering, and attribute changes support **Ctrl+Z** undo and **Ctrl+Y** redo.
