//! Command to change the tile marking source enum for a tileset definition.

use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{EnumDefId, TilesetDefId};
use lc_core::project::Project;

/// Sets or clears the source enum for tile marking on a `TilesetDef`.
#[derive(Debug, Clone)]
pub struct SetMarkingSourceCommand {
    tileset_id: TilesetDefId,
    new_source: Option<EnumDefId>,
    old_source: Option<Option<EnumDefId>>,
}

impl SetMarkingSourceCommand {
    /// Creates a command to update the tile marking source enum of a tileset.
    pub fn new(tileset_id: TilesetDefId, new_source: Option<EnumDefId>) -> Self {
        Self {
            tileset_id,
            new_source,
            old_source: None,
        }
    }

    /// Accessor for the tileset ID being configured.
    pub fn tileset_id(&self) -> TilesetDefId {
        self.tileset_id
    }

    /// Accessor for the target source enum.
    pub fn new_source(&self) -> Option<EnumDefId> {
        self.new_source
    }
}

impl Command for SetMarkingSourceCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let ts = p
            .defs
            .tileset_mut(self.tileset_id)
            .ok_or_else(|| CoreError::DefNotFound(format!("TilesetDef {:?}", self.tileset_id)))?;

        // Selecting the same enum is a no-op
        if ts.marking.source_enum == self.new_source {
            return Ok(());
        }

        self.old_source = Some(ts.marking.source_enum);
        ts.marking.source_enum = self.new_source;
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(old) = self.old_source.take() {
            if let Some(ts) = p.defs.tileset_mut(self.tileset_id) {
                ts.marking.source_enum = old;
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Set Marking Source Enum".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::TilesetDef(self.tileset_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::defs::{TileMarking, TilesetDef};

    fn make_test_project() -> (Project, TilesetDefId) {
        let mut p = Project::new("Test Project");
        let ts_id = TilesetDefId(1);
        p.defs.tilesets.push(TilesetDef {
            id: ts_id,
            name: "TestTileset".to_string(),
            path: "test.png".to_string(),
            tile_size: UVec2::new(16, 16),
            spacing: 0,
            padding: 0,
            tags: Vec::new(),
            marking: TileMarking::EMPTY,
            custom_data: Default::default(),
            animations: Default::default(),
            collisions: Default::default(),
        });
        (p, ts_id)
    }

    #[test]
    fn test_set_marking_source_apply_and_revert() {
        let (mut p, ts_id) = make_test_project();
        let enum_id = EnumDefId(42);

        let mut cmd = SetMarkingSourceCommand::new(ts_id, Some(enum_id));
        cmd.apply(&mut p).expect("apply must succeed");

        assert_eq!(
            p.defs.tileset(ts_id).unwrap().marking.source_enum,
            Some(enum_id)
        );

        cmd.revert(&mut p);
        assert_eq!(p.defs.tileset(ts_id).unwrap().marking.source_enum, None);
    }

    #[test]
    fn test_set_marking_source_same_is_noop() {
        let (mut p, ts_id) = make_test_project();
        let enum_id = EnumDefId(42);

        p.defs.tileset_mut(ts_id).unwrap().marking.source_enum = Some(enum_id);

        let mut cmd = SetMarkingSourceCommand::new(ts_id, Some(enum_id));
        cmd.apply(&mut p).expect("apply must succeed");
        assert_eq!(
            p.defs.tileset(ts_id).unwrap().marking.source_enum,
            Some(enum_id)
        );

        // Since it was no-op, old_source was not set
        cmd.revert(&mut p);
        assert_eq!(
            p.defs.tileset(ts_id).unwrap().marking.source_enum,
            Some(enum_id)
        );
    }
}
