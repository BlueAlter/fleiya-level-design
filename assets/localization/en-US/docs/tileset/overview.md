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
- **None**: Left-click opens the **Custom Data** modal for JSON or text. Right-click clears data.
- **Marking**: Left-click applies the active enum tag; right-click removes it.
- **Animation**: Creates frame-based animated tiles. Anchors cycle frames in real time on the canvas and in the preview card. The toolbar toggles playback, mode (`Loop`, `Ping-Pong`, `Once`), duration, and preview card.

## Safety

- All edits, frame reordering, and attribute changes support **Ctrl+Z** undo and **Ctrl+Y** redo.
