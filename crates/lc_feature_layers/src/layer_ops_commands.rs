//! Advanced LDtk layer operations: baking Auto-Layers to Tiles and moving Entities across layers.

use lc_core::auto_rules_eval::evaluate_auto_rules;
use lc_core::command::{Command, DocKind};
use lc_core::defs::LayerDef;
use lc_core::error::CoreError;
use lc_core::grid::Grid;
use lc_core::id::{LayerDefId, LevelId};
use lc_core::layer::{EntityInstance, LayerData, LayerKind, TileRef};
use lc_core::project::Project;

/// Bakes an Auto-Layer (`LayerKind::Auto`) or IntGrid+Tileset layer into a static `LayerKind::Tiles` layer.
#[derive(Debug, Clone)]
pub struct BakeAutoLayerCommand {
    layer_id: LayerDefId,
    seed: u64,
    prev_def: Option<LayerDef>,
    prev_level_layers: Vec<(LevelId, LayerData)>,
}

impl BakeAutoLayerCommand {
    /// Creates a command baking `layer_id` with deterministic evaluation `seed`.
    pub fn new(layer_id: LayerDefId, seed: u64) -> Self {
        Self {
            layer_id,
            seed,
            prev_def: None,
            prev_level_layers: Vec::new(),
        }
    }
}

impl Command for BakeAutoLayerCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let def = p
            .defs
            .layer(self.layer_id)
            .cloned()
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.layer_id)))?;
        if !def.uses_auto_rules() {
            return Err(CoreError::DefNotFound(
                "Only Auto-Layers or IntGrid layers with a bound Tileset can be baked.".into(),
            ));
        }
        let tileset_id = def.tileset_id.ok_or_else(|| {
            CoreError::DefNotFound("Layer must have a bound Tileset to bake auto-tiles.".into())
        })?;
        let source_id = match def.kind {
            LayerKind::Auto => def.auto_source_layer_def_id.ok_or_else(|| {
                CoreError::DefNotFound(
                    "Auto-Layer requires a Source IntGrid layer before baking.".into(),
                )
            })?,
            _ => def.id,
        };
        let source_values = p
            .defs
            .layer(source_id)
            .map(|l| l.int_grid_values.clone())
            .unwrap_or_default();

        self.prev_def = Some(def.clone());
        self.prev_level_layers.clear();

        let killed_by_id = def.auto_tiles_killed_by_other_layer_id;

        for world in &mut p.worlds {
            for level in &mut world.levels {
                let Some(idx) = level.layers.iter().position(|l| l.def_id == self.layer_id) else {
                    continue;
                };
                self.prev_level_layers
                    .push((level.id, level.layers[idx].data.clone()));

                let cols = (level.size_px.x / def.grid_size.x.max(1)).max(1);
                let rows = (level.size_px.y / def.grid_size.y.max(1)).max(1);
                let mut tile_grid: Grid<Option<TileRef>> = Grid::new(cols, rows, None);

                if let Some(LayerData::IntGrid(src_grid)) = level.layer(source_id).map(|l| &l.data)
                {
                    let killer_grid = killed_by_id.and_then(|kid| {
                        level.layer(kid).and_then(|l| match &l.data {
                            LayerData::IntGrid(g) => Some(g),
                            _ => None,
                        })
                    });
                    let evaluated = evaluate_auto_rules(
                        src_grid,
                        &source_values,
                        &def.auto_rule_groups,
                        &[],
                        killer_grid,
                        self.seed,
                    );
                    for et in evaluated {
                        if let (Ok(gx), Ok(gy)) = (u32::try_from(et.x), u32::try_from(et.y)) {
                            let _ = tile_grid.set(
                                gx,
                                gy,
                                Some(TileRef {
                                    tileset_id,
                                    tile_id: et.tile_id,
                                    flip_x: et.flip_x,
                                    flip_y: et.flip_y,
                                }),
                            );
                        }
                    }
                }

                level.layers[idx].data = LayerData::Tiles(tile_grid);
            }
        }

        if let Some(layer_def) = p.defs.layer_mut(self.layer_id) {
            layer_def.kind = LayerKind::Tiles;
            layer_def.auto_source_layer_def_id = None;
            layer_def.auto_tiles_killed_by_other_layer_id = None;
            layer_def.auto_rule_groups.clear();
            layer_def.int_grid_values.clear();
            layer_def.int_grid_value_groups.clear();
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(prev) = self.prev_def.take() {
            if let Some(slot) = p.defs.layer_mut(self.layer_id) {
                *slot = prev;
            }
        }
        for (level_id, prev_data) in self.prev_level_layers.drain(..) {
            if let Some(level) = p.level_mut(level_id) {
                if let Some(layer) = level.layer_mut(self.layer_id) {
                    layer.data = prev_data;
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Bake Auto-Layer to Tiles".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Moves all entity instances from one `Entities` layer definition to another across every level.
#[derive(Debug, Clone)]
pub struct MoveEntitiesToLayerCommand {
    from_layer_id: LayerDefId,
    to_layer_id: LayerDefId,
    /// Snapshot per level `(level_id, prev_from_entities, prev_to_entities)`.
    snapshots: Vec<(LevelId, Vec<EntityInstance>, Vec<EntityInstance>)>,
}

impl MoveEntitiesToLayerCommand {
    /// Creates a command moving all entities from `from_layer_id` to `to_layer_id`.
    pub fn new(from_layer_id: LayerDefId, to_layer_id: LayerDefId) -> Self {
        Self {
            from_layer_id,
            to_layer_id,
            snapshots: Vec::new(),
        }
    }
}

impl Command for MoveEntitiesToLayerCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        if self.from_layer_id == self.to_layer_id {
            return Err(CoreError::DefNotFound(
                "Source and target Entities layers must be different.".into(),
            ));
        }
        let from_kind = p
            .defs
            .layer(self.from_layer_id)
            .map(|l| l.kind)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.from_layer_id)))?;
        let to_kind = p
            .defs
            .layer(self.to_layer_id)
            .map(|l| l.kind)
            .ok_or_else(|| CoreError::DefNotFound(format!("LayerDef {:?}", self.to_layer_id)))?;
        if from_kind != LayerKind::Entities || to_kind != LayerKind::Entities {
            return Err(CoreError::DefNotFound(
                "Both layers must have kind Entities.".into(),
            ));
        }

        self.snapshots.clear();
        for world in &mut p.worlds {
            for level in &mut world.levels {
                let from_entities = match level.layer(self.from_layer_id).map(|l| &l.data) {
                    Some(LayerData::Entities(v)) => v.clone(),
                    _ => continue,
                };
                if from_entities.is_empty() {
                    continue;
                }
                let to_entities = match level.layer(self.to_layer_id).map(|l| &l.data) {
                    Some(LayerData::Entities(v)) => v.clone(),
                    _ => continue,
                };
                self.snapshots
                    .push((level.id, from_entities.clone(), to_entities));

                if let Some(from_inst) = level.layer_mut(self.from_layer_id) {
                    from_inst.data = LayerData::Entities(Vec::new());
                }
                if let Some(to_inst) = level.layer_mut(self.to_layer_id) {
                    if let LayerData::Entities(ref mut target_vec) = to_inst.data {
                        target_vec.extend(from_entities);
                    }
                }
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        for (level_id, prev_from, prev_to) in self.snapshots.drain(..) {
            if let Some(level) = p.level_mut(level_id) {
                if let Some(from_inst) = level.layer_mut(self.from_layer_id) {
                    from_inst.data = LayerData::Entities(prev_from);
                }
                if let Some(to_inst) = level.layer_mut(self.to_layer_id) {
                    to_inst.data = LayerData::Entities(prev_to);
                }
            }
        }
    }

    fn merge(&mut self, _next: &dyn Command) -> bool {
        false
    }

    fn label(&self) -> String {
        "Move Entities to Layer".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::LayerDef(self.from_layer_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
