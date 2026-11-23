//! Commands for tileset tags and enum tile markings.

use glam::UVec2;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::marking::{normalize_tag, tags_eq, TileMarking};
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

/// Adds a tag to a tileset definition. Rejects empty and case-insensitive duplicate tags.
#[derive(Debug, Clone)]
pub struct AddTilesetTagCommand {
    tileset_id: TilesetDefId,
    tag: String,
    added_tag: Option<String>,
}

impl AddTilesetTagCommand {
    /// Creates a command to add a tag to the specified tileset.
    pub fn new(tileset_id: TilesetDefId, tag: impl Into<String>) -> Self {
        Self {
            tileset_id,
            tag: tag.into(),
            added_tag: None,
        }
    }
}

impl Command for AddTilesetTagCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let norm = normalize_tag(&self.tag).ok_or_else(|| {
            CoreError::DefNotFound("Tag cannot be empty or whitespace".to_string())
        })?;

        if ts.tags.iter().any(|existing| tags_eq(existing, &norm)) {
            return Err(CoreError::DefNotFound(format!(
                "Tag {:?} already exists on tileset {:?}",
                norm, self.tileset_id
            )));
        }

        self.added_tag = Some(norm.clone());
        ts.tags.push(norm);
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(added), Some(ts)) =
            (self.added_tag.take(), p.defs.tileset_mut(self.tileset_id))
        {
            ts.tags.retain(|t| !tags_eq(t, &added));
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Add Tag \"{}\" to Tileset", self.tag)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Removes a tag from a tileset definition, preserving original order on revert.
#[derive(Debug, Clone)]
pub struct RemoveTilesetTagCommand {
    tileset_id: TilesetDefId,
    tag: String,
    removed_index: Option<usize>,
    removed_tag: Option<String>,
}

impl RemoveTilesetTagCommand {
    /// Creates a command to remove a tag from the specified tileset.
    pub fn new(tileset_id: TilesetDefId, tag: impl Into<String>) -> Self {
        Self {
            tileset_id,
            tag: tag.into(),
            removed_index: None,
            removed_tag: None,
        }
    }
}

impl Command for RemoveTilesetTagCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let pos = ts
            .tags
            .iter()
            .position(|t| tags_eq(t, &self.tag))
            .ok_or_else(|| CoreError::DefNotFound(format!("Tag {:?} not found", self.tag)))?;

        self.removed_index = Some(pos);
        self.removed_tag = Some(ts.tags.remove(pos));
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(pos), Some(tag), Some(ts)) = (
            self.removed_index.take(),
            self.removed_tag.take(),
            p.defs.tileset_mut(self.tileset_id),
        ) {
            if pos <= ts.tags.len() {
                ts.tags.insert(pos, tag);
            } else {
                ts.tags.push(tag);
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Remove Tag \"{}\" from Tileset", self.tag)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Renames a tag on a single tileset definition. Merges on collision with existing tag.
#[derive(Debug, Clone)]
pub struct RenameTilesetTagCommand {
    tileset_id: TilesetDefId,
    old_tag: String,
    new_tag: String,
    previous_tags: Option<Vec<String>>,
}

impl RenameTilesetTagCommand {
    /// Creates a command to rename a tag on the specified tileset.
    pub fn new(
        tileset_id: TilesetDefId,
        old_tag: impl Into<String>,
        new_tag: impl Into<String>,
    ) -> Self {
        Self {
            tileset_id,
            old_tag: old_tag.into(),
            new_tag: new_tag.into(),
            previous_tags: None,
        }
    }
}

impl Command for RenameTilesetTagCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let norm = normalize_tag(&self.new_tag).ok_or_else(|| {
            CoreError::DefNotFound("Tag cannot be empty or whitespace".to_string())
        })?;

        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        let old_pos = ts
            .tags
            .iter()
            .position(|t| tags_eq(t, &self.old_tag))
            .ok_or_else(|| CoreError::DefNotFound(format!("Tag {:?} not found", self.old_tag)))?;

        self.previous_tags = Some(ts.tags.clone());

        if tags_eq(&self.old_tag, &norm) {
            if let Some(target) = ts.tags.get_mut(old_pos) {
                *target = norm;
            }
        } else if ts.tags.iter().any(|t| tags_eq(t, &norm)) {
            // Collision: old tag merges into existing tag
            ts.tags.remove(old_pos);
        } else if let Some(target) = ts.tags.get_mut(old_pos) {
            *target = norm;
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let (Some(prev), Some(ts)) = (
            self.previous_tags.take(),
            p.defs.tileset_mut(self.tileset_id),
        ) {
            ts.tags = prev;
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Rename Tag \"{}\" to \"{}\"", self.old_tag, self.new_tag)
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

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
    pub fn new(tileset_id: TilesetDefId, value_id: impl Into<String>, tile: u32, active: bool) -> Self {
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
