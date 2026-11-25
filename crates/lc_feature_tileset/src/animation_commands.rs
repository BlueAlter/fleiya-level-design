//! Commands for managing tileset tile animations.

use lc_core::command::{Command, DocKind};
use lc_core::defs::TileAnimationDef;
use lc_core::error::CoreError;
use lc_core::id::TilesetDefId;
use lc_core::project::Project;

/// Sets or updates a tile animation associated with an anchor tile.
#[derive(Debug, Clone)]
pub struct SetTileAnimationCommand {
    tileset_id: TilesetDefId,
    anim: TileAnimationDef,
    old_anim: Option<TileAnimationDef>,
}

impl SetTileAnimationCommand {
    /// Creates a command to set or update a tile animation definition.
    pub fn new(tileset_id: TilesetDefId, anim: TileAnimationDef) -> Self {
        Self {
            tileset_id,
            anim,
            old_anim: None,
        }
    }

    /// Target tileset identifier.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Anchor tile identifier.
    pub fn anchor_tile_id(&self) -> u32 {
        self.anim.anchor_tile_id
    }

    /// The animation definition being assigned.
    pub fn anim(&self) -> &TileAnimationDef {
        &self.anim
    }
}

impl Command for SetTileAnimationCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_anim = ts.animations.get(&self.anim.anchor_tile_id).cloned();
        ts.animations
            .insert(self.anim.anchor_tile_id, self.anim.clone());

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            match self.old_anim.take() {
                Some(old) => {
                    ts.animations.insert(self.anim.anchor_tile_id, old);
                }
                None => {
                    ts.animations.remove(&self.anim.anchor_tile_id);
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<SetTileAnimationCommand>() {
            if self.tileset_id == other.tileset_id
                && self.anim.anchor_tile_id == other.anim.anchor_tile_id
            {
                self.anim = other.anim.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        format!("Edit Animation for Tile {}", self.anim.anchor_tile_id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes a tile animation definition for an anchor tile.
#[derive(Debug, Clone)]
pub struct RemoveTileAnimationCommand {
    tileset_id: TilesetDefId,
    anchor_tile_id: u32,
    old_anim: Option<TileAnimationDef>,
}

impl RemoveTileAnimationCommand {
    /// Creates a command to remove an animation definition from a tileset.
    pub fn new(tileset_id: TilesetDefId, anchor_tile_id: u32) -> Self {
        Self {
            tileset_id,
            anchor_tile_id,
            old_anim: None,
        }
    }
}

impl Command for RemoveTileAnimationCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.old_anim = ts.animations.remove(&self.anchor_tile_id);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            if let Some(old) = self.old_anim.take() {
                ts.animations.insert(self.anchor_tile_id, old);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Animation for Tile {}", self.anchor_tile_id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Appends a new frame tile ID to an anchor tile's animation.
#[derive(Debug, Clone)]
pub struct AddAnimationFrameCommand {
    tileset_id: TilesetDefId,
    anchor_tile_id: u32,
    frame_tile_id: u32,
}

impl AddAnimationFrameCommand {
    /// Creates a command to add a frame to an animation.
    pub fn new(tileset_id: TilesetDefId, anchor_tile_id: u32, frame_tile_id: u32) -> Self {
        Self {
            tileset_id,
            anchor_tile_id,
            frame_tile_id,
        }
    }
}

impl Command for AddAnimationFrameCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let anim = ts
            .animations
            .entry(self.anchor_tile_id)
            .or_insert_with(|| TileAnimationDef::new(self.anchor_tile_id));
        anim.frames.push(self.frame_tile_id);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            if let Some(anim) = ts.animations.get_mut(&self.anchor_tile_id) {
                anim.frames.pop();
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!(
            "Add Frame {} to Animation {}",
            self.frame_tile_id, self.anchor_tile_id
        )
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes a frame at a specific index from an anchor tile's animation.
#[derive(Debug, Clone)]
pub struct RemoveAnimationFrameCommand {
    tileset_id: TilesetDefId,
    anchor_tile_id: u32,
    index: usize,
    removed_tile_id: Option<u32>,
}

impl RemoveAnimationFrameCommand {
    /// Creates a command to remove a frame at `index`.
    pub fn new(tileset_id: TilesetDefId, anchor_tile_id: u32, index: usize) -> Self {
        Self {
            tileset_id,
            anchor_tile_id,
            index,
            removed_tile_id: None,
        }
    }
}

impl Command for RemoveAnimationFrameCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let anim = ts.animations.get_mut(&self.anchor_tile_id).ok_or_else(|| {
            CoreError::DefNotFound(format!(
                "Animation anchor {} not found",
                self.anchor_tile_id
            ))
        })?;

        if self.index < anim.frames.len() {
            self.removed_tile_id = Some(anim.frames.remove(self.index));
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            if let Some(anim) = ts.animations.get_mut(&self.anchor_tile_id) {
                if let Some(tile_id) = self.removed_tile_id.take() {
                    let insert_idx = self.index.min(anim.frames.len());
                    anim.frames.insert(insert_idx, tile_id);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Frame from Animation {}", self.anchor_tile_id)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
