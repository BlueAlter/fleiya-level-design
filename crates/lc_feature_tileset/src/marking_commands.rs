//! Commands for tileset tags and enum tile markings.

use glam::UVec2;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::TileMarking;
use lc_core::project::Project;

/// Calculates total tile count given image dimensions, tile dimensions, spacing and padding.
pub fn compute_tile_count(image_size: UVec2, tile_size: UVec2, spacing: u32, padding: u32) -> u32 {
    let tile_w = tile_size.x.max(1);
    let tile_h = tile_size.y.max(1);
    let eff_w = image_size.x.saturating_sub(padding * 2);
    let eff_h = image_size.y.saturating_sub(padding * 2);
    let stride_x = tile_w.saturating_add(spacing);
    let stride_y = tile_h.saturating_add(spacing);
    if stride_x == 0 || stride_y == 0 {
        return 0;
    }
    let cols = (eff_w.saturating_add(spacing)) / stride_x;
    let rows = (eff_h.saturating_add(spacing)) / stride_y;
    cols.saturating_mul(rows)
}

pub use crate::tag_commands::{
    AddTilesetTagCommand, RemoveTilesetTagCommand, RenameTilesetTagCommand,
};

/// Type alias for SetMarkingSourceCommand.
pub type SetTilesetSourceEnumCommand = SetMarkingSourceCommand;

/// Changes or clears the source enum for tileset markings.
/// Changing the source enum clears existing marking tags; revert restores source and tags exactly.
#[derive(Debug, Clone)]
pub struct SetMarkingSourceCommand {
    tileset_id: TilesetDefId,
    enum_id: Option<EnumDefId>,
    previous_marking: Option<TileMarking>,
}

impl SetMarkingSourceCommand {
    /// Creates a command to set or clear the marking source enum.
    pub fn new(tileset_id: TilesetDefId, enum_id: Option<EnumDefId>) -> Self {
        Self {
            tileset_id,
            enum_id,
            previous_marking: None,
        }
    }
}

impl Command for SetMarkingSourceCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if let Some(id) = self.enum_id {
            if p.defs.enum_def(id).is_none() {
                return Err(CoreError::DefNotFound(format!("EnumDef {:?}", id)));
            }
        }

        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.previous_marking = Some(ts.marking.clone());
        ts.marking.set_source(self.enum_id);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(prev), Some(ts)) = (
            self.previous_marking.take(),
            p.defs.tileset_mut(self.tileset_id),
        ) {
            ts.marking = prev;
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Change Tileset Marking Source".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Paints or removes tags for an enum value on a list of tile indices.
/// Records only the tiles that actually changed for exact undo parity.
#[derive(Debug, Clone)]
pub struct PaintTileTagsCommand {
    tileset_id: TilesetDefId,
    value_id: String,
    tiles: Vec<u32>,
    add: bool,
    changed_tiles: Vec<u32>,
}

impl PaintTileTagsCommand {
    /// Creates a command to paint or untag an enum value on tiles.
    pub fn new(
        tileset_id: TilesetDefId,
        value_id: impl Into<String>,
        tiles: Vec<u32>,
        add: bool,
    ) -> Self {
        Self {
            tileset_id,
            value_id: value_id.into(),
            tiles,
            add,
            changed_tiles: Vec::new(),
        }
    }
}

impl Command for PaintTileTagsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let source_enum = ts.marking.source_enum.ok_or_else(|| {
            CoreError::DefNotFound("Tileset has no marking source enum configured".to_string())
        })?;

        let enum_def = p
            .defs
            .enum_def(source_enum)
            .ok_or_else(|| CoreError::DefNotFound(format!("EnumDef {:?}", source_enum)))?;

        if enum_def.value(&self.value_id).is_none() {
            return Err(CoreError::DefNotFound(format!(
                "EnumValue {:?} not in EnumDef {:?}",
                self.value_id, source_enum
            )));
        }

        let ts_mut = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        if self.changed_tiles.is_empty() {
            for &tile in &self.tiles {
                if ts_mut.marking.set_tag(&self.value_id, tile, self.add) {
                    self.changed_tiles.push(tile);
                }
            }
        } else {
            for &tile in &self.changed_tiles {
                ts_mut.marking.set_tag(&self.value_id, tile, self.add);
            }
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
            for &tile in &self.changed_tiles {
                ts.marking.set_tag(&self.value_id, tile, !self.add);
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.tileset_id == other.tileset_id
                && self.value_id == other.value_id
                && self.add == other.add
            {
                for &tile in &other.changed_tiles {
                    if !self.changed_tiles.contains(&tile) {
                        self.changed_tiles.push(tile);
                    }
                }
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        if self.add {
            format!("Tag Tiles with \"{}\"", self.value_id)
        } else {
            format!("Untag Tiles from \"{}\"", self.value_id)
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper command alias to set or toggle marking on a single tile.
#[derive(Debug, Clone)]
pub struct SetTileMarkingCommand {
    inner: PaintTileTagsCommand,
}

impl SetTileMarkingCommand {
    /// Creates a command to set or clear marking for a single tile.
    pub fn new(
        tileset_id: TilesetDefId,
        value_id: impl Into<String>,
        tile: u32,
        active: bool,
    ) -> Self {
        Self {
            inner: PaintTileTagsCommand::new(tileset_id, value_id, vec![tile], active),
        }
    }
}

impl Command for SetTileMarkingCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        self.inner.apply(p)
    }

    fn revert(&mut self, p: &mut Project) {
        self.inner.revert(p);
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        self.inner.merge(next)
    }

    fn label(&self) -> String {
        self.inner.label()
    }

    fn focus(&self) -> Option<DocKind> {
        self.inner.focus()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Prunes tile markings that exceed the valid tile count.
#[derive(Debug, Clone)]
pub struct PruneTilesetMarkingsCommand {
    tileset_id: TilesetDefId,
    tile_count: u32,
    previous_marking: Option<TileMarking>,
}

impl PruneTilesetMarkingsCommand {
    /// Creates a command to prune out-of-range tile markings.
    pub fn new(tileset_id: TilesetDefId, tile_count: u32) -> Self {
        Self {
            tileset_id,
            tile_count,
            previous_marking: None,
        }
    }
}

impl Command for PruneTilesetMarkingsCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        self.previous_marking = Some(ts.marking.clone());
        ts.marking.prune_tiles_from(self.tile_count);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(prev), Some(ts)) = (
            self.previous_marking.take(),
            p.defs.tileset_mut(self.tileset_id),
        ) {
            ts.marking = prev;
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Prune Tileset Markings".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
