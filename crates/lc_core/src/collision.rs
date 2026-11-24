//! Tile collision geometry and shape definitions.
//!
//! Provides data structures for defining collision boundaries (boxes, polygons, circles)
//! assigned per tile within a tileset. Supports tags, one-way platforms, and custom data.

use glam::{UVec2, Vec2};
use serde::{Deserialize, Serialize};

/// Kind of geometry for a collision shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollisionShapeGeometry {
    /// Axis-aligned rectangle relative to tile top-left `(0, 0)`.
    Rectangle {
        /// X offset in tile local coordinates.
        x: f32,
        /// Y offset in tile local coordinates.
        y: f32,
        /// Width in tile local coordinates.
        w: f32,
        /// Height in tile local coordinates.
        h: f32,
    },
    /// Polygon defined by vertices relative to tile top-left `(0, 0)`.
    Polygon {
        /// Ordered vertices forming the polygon boundary.
        points: Vec<Vec2>,
    },
    /// Circle with center and radius relative to tile top-left `(0, 0)`.
    Circle {
        /// Center X offset.
        cx: f32,
        /// Center Y offset.
        cy: f32,
        /// Radius in tile local coordinates.
        radius: f32,
    },
}

/// A single collision shape attached to a tile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollisionShape {
    /// Local shape ID within this tile (stable identifier).
    pub id: u32,
    /// Geometry of the shape.
    pub geometry: CollisionShapeGeometry,
    /// Semantic collision tag (e.g. "solid", "hazard", "ladder", "platform").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tag: String,
    /// Whether this shape acts as a one-way platform.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub one_way: bool,
    /// Arbitrary custom data/properties string (e.g. JSON, key-value, text).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub custom_data: String,
}

impl CollisionShape {
    /// Creates a full-tile box collision shape.
    pub fn full_tile(id: u32, tile_size: UVec2) -> Self {
        Self {
            id,
            geometry: CollisionShapeGeometry::Rectangle {
                x: 0.0,
                y: 0.0,
                w: tile_size.x as f32,
                h: tile_size.y as f32,
            },
            tag: "solid".to_string(),
            one_way: false,
            custom_data: String::new(),
        }
    }

    /// Creates a new rectangle collision shape.
    pub fn new_rect(id: u32, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            geometry: CollisionShapeGeometry::Rectangle { x, y, w, h },
            tag: "solid".to_string(),
            one_way: false,
            custom_data: String::new(),
        }
    }

    /// Creates a new polygon collision shape.
    pub fn new_polygon(id: u32, points: Vec<Vec2>) -> Self {
        Self {
            id,
            geometry: CollisionShapeGeometry::Polygon { points },
            tag: "solid".to_string(),
            one_way: false,
            custom_data: String::new(),
        }
    }

    /// Creates a 45-degree slope-up polygon (bottom-left to top-right / ).
    pub fn slope_up(id: u32, tile_size: UVec2) -> Self {
        let tw = tile_size.x as f32;
        let th = tile_size.y as f32;
        Self::new_polygon(
            id,
            vec![Vec2::new(0.0, th), Vec2::new(tw, 0.0), Vec2::new(tw, th)],
        )
    }

    /// Creates a 45-degree slope-down polygon (top-left to bottom-right \ ).
    pub fn slope_down(id: u32, tile_size: UVec2) -> Self {
        let tw = tile_size.x as f32;
        let th = tile_size.y as f32;
        Self::new_polygon(
            id,
            vec![Vec2::new(0.0, 0.0), Vec2::new(tw, th), Vec2::new(0.0, th)],
        )
    }
}

/// Collection of collision shapes assigned to a specific tile in a tileset.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TileCollisionDef {
    /// List of collision shapes on this tile.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shapes: Vec<CollisionShape>,
}

impl TileCollisionDef {
    /// Creates a new empty collision definition.
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    /// Creates a collision definition with a single full-tile box shape.
    pub fn full_tile(tile_size: UVec2) -> Self {
        Self {
            shapes: vec![CollisionShape::full_tile(1, tile_size)],
        }
    }

    /// Allocates the next stable shape ID within this tile.
    pub fn next_shape_id(&self) -> u32 {
        self.shapes.iter().map(|s| s.id).max().unwrap_or(0) + 1
    }

    /// Returns true if this tile has no collision shapes.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_shape_full_tile_and_slopes() {
        let tile_size = UVec2::new(16, 16);
        let full = CollisionShape::full_tile(1, tile_size);
        assert_eq!(full.id, 1);
        assert_eq!(full.tag, "solid");
        assert!(!full.one_way);
        assert!(matches!(
            full.geometry,
            CollisionShapeGeometry::Rectangle { x, y, w, h } if x == 0.0 && y == 0.0 && w == 16.0 && h == 16.0
        ));

        let slope = CollisionShape::slope_up(2, tile_size);
        assert!(matches!(
            slope.geometry,
            CollisionShapeGeometry::Polygon { points } if points.len() == 3
        ));
    }

    #[test]
    fn test_tile_collision_def_next_id() {
        let mut def = TileCollisionDef::new();
        assert!(def.is_empty());
        assert_eq!(def.next_shape_id(), 1);

        def.shapes
            .push(CollisionShape::new_rect(1, 0.0, 0.0, 8.0, 8.0));
        assert!(!def.is_empty());
        assert_eq!(def.next_shape_id(), 2);
    }
}
