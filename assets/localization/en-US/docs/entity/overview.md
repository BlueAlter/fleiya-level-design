# Entities

Entities represent game objects, triggers, spawn points, NPCs, and interactive regions placed onto **Entities** layers.

## Entity Definition Properties

| Property | Description |
|---|---|
| **Identifier** | Unique class name exported to JSON and used by game code. |
| **Dimensions & Resize** | Default `W x H` in pixels, `Resizable X/Y`, `Keep aspect ratio`, and `Allow out of bounds`. |
| **Pivot** | Normalized `[0.0..1.0]` anchor via 3x3 preset grid or custom coordinates. |
| **Render Mode** | `Rectangle`, `Ellipse`, `Cross`, or `Tile` with configurable fill, line, and tile opacities. |
| **Tile Render Mode** | `Cover`, `FitInside`, `Repeat`, `Stretch`, `FullSizeCropped`, `FullSizeUncropped`, or `NineSlice` with 4 pixel borders. |
| **Hollow** | Renders a transparent core and lets clicks pass through to underlying entities/tiles. |
| **Instance Limits** | `Max count` enforced `PerLayer`, `PerLevel`, or `PerWorld` via `DiscardOldOnes`, `PreventAdding`, or `MoveLastOne`. |

## Custom Fields

Entities support all 11 LDtk field types (`Int`, `Float`, `Bool`, `String`, `Multilines`, `Color`, `Enum`, `Point`, `FilePath`, `EntityRef`, `Tile`), optional `Array` wrapping, `Can be null`, `Min/Max` or `Regex` constraints, `AllowedRefs` (`Any`, `OnlySame`, `OnlyTags`, `OnlySpecificEntity`), `Symmetrical Ref`, `Smart Color`, and 15 canvas `Editor Display` modes.
