//! Commands for managing per-tile custom string data.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;

/// Sets, updates, or clears custom string data associated with a tile.
#[derive(Debug, Clone)]
pub struct SetTileCustomDataCommand {
    tileset_id: TilesetDefId,
    tile: u32,
    new_data: Option<String>,
    old_data: Option<String>,
}

impl SetTileCustomDataCommand {
    /// Creates a command to set, update or clear custom data for a tile.
    ///
    /// Passing `None` or an empty string clears the custom data.
    pub fn new(tileset_id: TilesetDefId, tile: u32, data: Option<impl Into<String>>) -> Self {
        let new_data = data.map(|s| s.into()).filter(|s| !s.trim().is_empty());
        Self {
            tileset_id,
            tile,
            new_data,
            old_data: None,
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

    /// The new custom data being assigned, if any.
    pub fn new_data(&self) -> Option<&str> {
        self.new_data.as_deref()
    }
}

impl Command for SetTileCustomDataCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_data = ts.custom_data.get(&self.tile).cloned();

        match &self.new_data {
            Some(data) => {
                ts.custom_data.insert(self.tile, data.clone());
            }
            None => {
                ts.custom_data.remove(&self.tile);
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            match self.old_data.take() {
                Some(data) => {
                    ts.custom_data.insert(self.tile, data);
                }
                None => {
                    ts.custom_data.remove(&self.tile);
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<SetTileCustomDataCommand>() {
            if self.tileset_id == other.tileset_id && self.tile == other.tile {
                self.new_data = other.new_data.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Custom Data for Tile {}", self.tile)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
