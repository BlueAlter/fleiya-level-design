# Tilesets

A **tileset** is a sliced image (atlas) containing graphical tiles used for
painting levels, decorating terrain, and assigning icons to enums and entities.

Each tileset definition specifies:

- an **identifier** used in project definitions and references,
- a **source image** path relative to the project root,
- a uniform **tile size** (width and height in pixels),
- optional **spacing** between tiles,
- optional border **padding** around the entire image.

## Create a tileset

1. Press **+ New Tileset** in the tileset list on the left.
2. Select a PNG image from your project folder using the file dialog.
3. Configure the **Tile Size**, **Spacing**, and **Padding** in the attributes card.
4. The atlas view immediately updates to show the sliced grid and dimensions.

## Atlas View

The central workspace displays the source atlas with an exact-aspect preview:

- **Pan**: Drag with the middle mouse button or hold `Space` and drag.
- **Zoom**: Hold `Ctrl` and scroll with the mouse wheel towards the cursor.
- **Grid Lines**: Outlines each sliceable tile cell in the atlas.
- **Inspect**: Hover over any tile to see its ID and grid coordinates.
- **Select**: Click a tile to highlight it and view its details.

## Features and Workflows

- **Layers**: Tile layers reference tilesets to place tiles onto level grids.
- **Tags** (planned): Categorize tilesets for quick filtering across large projects.
- **Marking** (planned): Associate tiles with enum values (e.g. terrain, collision).
- **Animation** (planned): Group consecutive tiles into frame-based animations.

## Safety and Undo

- All attribute changes, additions, deletions, and reordering actions are undoable.
- Press **Ctrl+Z** to undo and **Ctrl+Y** to redo changes at any time.
- Deleting a tileset referenced by layers, enums, or levels warns before removal.
