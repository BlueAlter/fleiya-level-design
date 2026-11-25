//! Commands for managing tags on tileset definitions.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::TilesetDefId;
use lc_core::marking::{normalize_tag, tags_eq};
use lc_core::project::Project;

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
