//! Commands for managing per-tile collision definitions and shapes.

use lc_core::command::{Command, DocKind};
use lc_core::defs::TileCollisionDef;
use lc_core::error::CoreError;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;

/// Sets or updates the collision definition associated with a tile.
#[derive(Debug, Clone)]
pub struct SetTileCollisionCommand {
    tileset_id: TilesetDefId,
    tile: u32,
    collision: TileCollisionDef,
    old_collision: Option<TileCollisionDef>,
}

impl SetTileCollisionCommand {
    /// Creates a command to set or update collision shapes for a single tile.
    pub fn new(tileset_id: TilesetDefId, tile: u32, collision: TileCollisionDef) -> Self {
        Self {
            tileset_id,
            tile,
            collision,
            old_collision: None,
        }
    }

    /// Target tileset identifier.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Target tile index.
    pub fn tile(&self) -> u32 {
        self.tile
    }

    /// The collision definition being assigned.
    pub fn collision(&self) -> &TileCollisionDef {
        &self.collision
    }
}

impl Command for SetTileCollisionCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_collision = ts.collisions.get(&self.tile).cloned();

        if self.collision.is_empty() {
            ts.collisions.remove(&self.tile);
        } else {
            ts.collisions.insert(self.tile, self.collision.clone());
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            match self.old_collision.take() {
                Some(old) => {
                    ts.collisions.insert(self.tile, old);
                }
                None => {
                    ts.collisions.remove(&self.tile);
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<SetTileCollisionCommand>() {
            if self.tileset_id == other.tileset_id && self.tile == other.tile {
                self.collision = other.collision.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Collision for Tile {}", self.tile)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes all collision shapes from a single tile.
#[derive(Debug, Clone)]
pub struct RemoveTileCollisionCommand {
    tileset_id: TilesetDefId,
    tile: u32,
    old_collision: Option<TileCollisionDef>,
}

impl RemoveTileCollisionCommand {
    /// Creates a command to remove collision shapes from a tile.
    pub fn new(tileset_id: TilesetDefId, tile: u32) -> Self {
        Self {
            tileset_id,
            tile,
            old_collision: None,
        }
    }

    /// Target tileset identifier.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Target tile index.
    pub fn tile(&self) -> u32 {
        self.tile
    }
}

impl Command for RemoveTileCollisionCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_collision = ts.collisions.remove(&self.tile);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            if let Some(old) = self.old_collision.take() {
                ts.collisions.insert(self.tile, old);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Collision from Tile {}", self.tile)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Sets or pastes the same collision definition across multiple tiles in a single atomic undo step.
#[derive(Debug, Clone)]
pub struct BatchSetTileCollisionCommand {
    tileset_id: TilesetDefId,
    tiles: Vec<u32>,
    collision: TileCollisionDef,
    old_collisions: Vec<(u32, Option<TileCollisionDef>)>,
}

impl BatchSetTileCollisionCommand {
    /// Creates a batch paste command assigning `collision` to all specified `tiles`.
    pub fn new(tileset_id: TilesetDefId, tiles: Vec<u32>, collision: TileCollisionDef) -> Self {
        Self {
            tileset_id,
            tiles,
            collision,
            old_collisions: Vec::new(),
        }
    }

    /// Target tileset identifier.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// List of target tile indices.
    pub fn tiles(&self) -> &[u32] {
        &self.tiles
    }

    /// Collision definition being assigned to each target tile.
    pub fn collision(&self) -> &TileCollisionDef {
        &self.collision
    }
}

impl Command for BatchSetTileCollisionCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_collisions.clear();
        for &tile in &self.tiles {
            let old = ts.collisions.get(&tile).cloned();
            self.old_collisions.push((tile, old));

            if self.collision.is_empty() {
                ts.collisions.remove(&tile);
            } else {
                ts.collisions.insert(tile, self.collision.clone());
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            for (tile, old) in self.old_collisions.drain(..) {
                match old {
                    Some(prev) => {
                        ts.collisions.insert(tile, prev);
                    }
                    None => {
                        ts.collisions.remove(&tile);
                    }
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Paste Collision to {} Tiles", self.tiles.len())
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
