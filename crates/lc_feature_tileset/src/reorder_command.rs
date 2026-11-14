//! Command to reorder a tileset definition within the project.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;

/// Reorders a `TilesetDef` within the project's tileset definitions.
#[derive(Debug, Clone)]
pub struct ReorderTilesetDefCommand {
    tileset_id: TilesetDefId,
    new_index: usize,
    old_index: Option<usize>,
}

impl ReorderTilesetDefCommand {
    /// Creates a command to move `tileset_id` to `new_index`.
    pub fn new(tileset_id: TilesetDefId, new_index: usize) -> Self {
        Self {
            tileset_id,
            new_index,
            old_index: None,
        }
    }

    /// Accessor for the tileset ID to be moved.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Accessor for the target index.
    pub fn new_index(&self) -> usize {
        self.new_index
    }
}

impl Command for ReorderTilesetDefCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let old_idx = p
            .defs
            .tilesets
            .iter()
            .position(|t| t.id == self.tileset_id)
            .ok_or_else(|| {
                CoreError::DefNotFound(format!("TilesetDef {:?} not found", self.tileset_id))
            })?;

        if self.new_index >= p.defs.tilesets.len() {
            return Err(CoreError::DefNotFound(format!(
                "Index {} out of range for {} tilesets",
                self.new_index,
                p.defs.tilesets.len()
            )));
        }

        self.old_index = Some(old_idx);
        if old_idx != self.new_index {
            let item = p.defs.tilesets.remove(old_idx);
            p.defs.tilesets.insert(self.new_index, item);
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old_idx) = self.old_index.take() {
            if let Some(cur_pos) = p.defs.tilesets.iter().position(|t| t.id == self.tileset_id) {
                if cur_pos != old_idx {
                    let item = p.defs.tilesets.remove(cur_pos);
                    p.defs.tilesets.insert(old_idx, item);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Reorder Tileset".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
