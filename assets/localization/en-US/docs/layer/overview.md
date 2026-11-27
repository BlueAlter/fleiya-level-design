# Layers

Layers define the ordered rendering and gameplay planes present in every level of the project. The top-most layer in **LAYER DEFINITIONS** renders in front of layers below it.

## Layer Kinds

| Kind | Purpose | Key Settings |
|---|---|---|
| **IntGrid** | Integer cell grid for collisions, terrain logic, hazards, and zones. | `Grid Size`, `IntGrid Values` (`value >= 1`), `Value Groups`, optional `Tileset Link` (enables Auto-Tile rules). |
| **Tiles** | Manual tilemap grid painted from a bound tileset. | `Grid Size`, `Tileset Link`. |
| **Entities** | Vector placement plane for entity instances and custom fields. | `Grid Size`, `Required/Excluded Tags`, `Hide fields when inactive`. |
| **Auto** | Procedural tile layer generated from an external `IntGrid` source layer via Auto-Tile rules. | `Source IntGrid`, `Tileset Link`, `Killed By Layer`, `Biome Field`. |

## Auto-Tile Rules (`AUTO TILE` Tab)

The **AUTO TILE** tab is interactive for **Auto** layers and **IntGrid** layers that have a bound **Tileset**. For **Tiles**, **Entities**, and **IntGrid** layers without a tileset, the **AUTO TILE** workspace enters **Uninteractable Mode** (only tab switching and layer selection remain active).

- **Rule Groups**: Organize rules into collapsible groups with active toggles, optional flags, and biome filters (`Require` / `Exclude`).
- **Pattern Matrix (`1x1` / `3x3` / `5x5` / `7x7`)**:
  - **Left-Click**: Require IntGrid value or group (`+v`).
  - **Right-Click**: Forbid IntGrid value or group (`-v`).
  - **Middle-Click / Shift+Click**: Clear condition (`0` = ignore).
- **Execution & Modifiers**: `Break on match`, `Flip X / Y`, `Single` vs `Stamp` (with `Pivot X/Y`), `Chance`, `Alpha`, `Modulo / Offset`, `Checker`, `Out of Bounds`, and `Perlin Noise`.

## World Layer Filtering by Tags

- Assign `Tags` to any `LayerDef` in **LAYERS DEF** to group layers in the sidebar and filter which layers each **World** uses.
- Each `World` can configure `Included Layer Tags` (whitelist; empty = all) and `Excluded Layer Tags` (blacklist; always overrides include) without destroying existing level layer data.
