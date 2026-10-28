//! Rectangular tile/int-grid clipboard buffer used by canvas copy/paste.

use glam::{IVec2, UVec2};
use lc_core::command::Command;
use lc_core::id::{LayerDefId, LevelId};
use lc_core::layer::{LayerData, LayerKind, TileRef};
use lc_core::level::Level;
use lc_ui_api::canvas::CanvasCtx;
use lc_ui_api::context::EditorCtx;
use lc_ui_api::selection::Selection;

use crate::paint_commands::{PaintIntGridCommand, PaintTilesCommand};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClipboardCell {
    Tile(Option<TileRef>),
    Int(i32),
}

/// A rectangular block of tile or int-grid cells captured from a level layer.
///
/// The buffer is owned by the calling tool, so copying never goes through the
/// operating-system clipboard and stays inside a single project session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileClipboard {
    kind: LayerKind,
    width: u32,
    height: u32,
    cells: Vec<ClipboardCell>,
}

impl TileClipboard {
    /// Captures the bounding box of `cells` from `layer_id` in `level`.
    ///
    /// Cells outside the layer bounds are stored as empty tile slots (or `0`
    /// for IntGrid layers). Returns `None` when no cell is usable or the layer
    /// kind is not copyable.
    pub fn copy(level: &Level, layer_id: LayerDefId, cells: &[IVec2]) -> Option<Self> {
        let layer = level.layer(layer_id)?;

        let mut min = IVec2::new(i32::MAX, i32::MAX);
        let mut max = IVec2::new(i32::MIN, i32::MIN);
        for cell in cells.iter().filter(|c| c.x >= 0 && c.y >= 0) {
            min = min.min(*cell);
            max = max.max(*cell);
        }
        if min.x > max.x || min.y > max.y {
            return None;
        }

        let width = (max.x - min.x + 1) as u32;
        let height = (max.y - min.y + 1) as u32;
        let mut captured = Vec::with_capacity((width as usize).saturating_mul(height as usize));

        match &layer.data {
            LayerData::Tiles(grid) => {
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let tile = u32::try_from(x)
                            .ok()
                            .zip(u32::try_from(y).ok())
                            .and_then(|(x, y)| grid.get(x, y).copied().flatten());
                        captured.push(ClipboardCell::Tile(tile));
                    }
                }
                Some(Self {
                    kind: LayerKind::Tiles,
                    width,
                    height,
                    cells: captured,
                })
            }
            LayerData::IntGrid(grid) => {
                for y in min.y..=max.y {
                    for x in min.x..=max.x {
                        let value = u32::try_from(x)
                            .ok()
                            .zip(u32::try_from(y).ok())
                            .and_then(|(x, y)| grid.get(x, y).copied())
                            .unwrap_or(0);
                        captured.push(ClipboardCell::Int(value));
                    }
                }
                Some(Self {
                    kind: LayerKind::IntGrid,
                    width,
                    height,
                    cells: captured,
                })
            }
            _ => None,
        }
    }

    /// Layer kind captured by this buffer.
    pub fn kind(&self) -> LayerKind {
        self.kind
    }

    /// Buffer width in cells.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Buffer height in cells.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Builds a paint command that stamps this buffer with its top-left cell at `at`.
    ///
    /// Returns `None` when the buffer is empty.
    pub fn paste_command(
        &self,
        level_id: LevelId,
        layer_id: LayerDefId,
        at: UVec2,
        stroke_id: u64,
    ) -> Option<Box<dyn Command>> {
        if self.cells.is_empty() || self.width == 0 {
            return None;
        }

        match self.kind {
            LayerKind::Tiles => {
                let mut cells = Vec::new();
                for (index, cell) in self.cells.iter().enumerate() {
                    let ClipboardCell::Tile(tile) = cell else {
                        continue;
                    };
                    let dx = index as u32 % self.width;
                    let dy = index as u32 / self.width;
                    let x = at.x.checked_add(dx)?;
                    let y = at.y.checked_add(dy)?;
                    cells.push((x, y, *tile));
                }
                if cells.is_empty() {
                    return None;
                }
                Some(Box::new(PaintTilesCommand::with_stroke(
                    level_id, layer_id, cells, stroke_id,
                )))
            }
            LayerKind::IntGrid => {
                let mut cells = Vec::new();
                for (index, cell) in self.cells.iter().enumerate() {
                    let ClipboardCell::Int(value) = cell else {
                        continue;
                    };
                    let dx = index as u32 % self.width;
                    let dy = index as u32 / self.width;
                    let x = at.x.checked_add(dx)?;
                    let y = at.y.checked_add(dy)?;
                    cells.push((x, y, *value));
                }
                if cells.is_empty() {
                    return None;
                }
                Some(Box::new(PaintIntGridCommand::with_stroke(
                    level_id, layer_id, cells, stroke_id,
                )))
            }
            _ => None,
        }
    }
}

/// Canvas clipboard state and Ctrl+C / Ctrl+V handling for tile tools.
#[derive(Debug, Default)]
pub struct CanvasClipboard {
    buffer: Option<TileClipboard>,
}

impl CanvasClipboard {
    /// Creates an empty canvas clipboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// Currently captured buffer, if anything has been copied.
    pub fn buffer(&self) -> Option<&TileClipboard> {
        self.buffer.as_ref()
    }

    /// Handles Ctrl+C / Ctrl+V on the canvas region selection.
    ///
    /// Returns `true` when a clipboard shortcut was consumed so the active
    /// sub-tool does not also react to the key press.
    pub fn handle_shortcuts(&mut self, cv: &mut CanvasCtx, cx: &mut EditorCtx) -> bool {
        let (copy_pressed, paste_pressed) = cv.response.ctx.input(|i| {
            (
                cv.modifiers.command && i.key_pressed(egui::Key::C),
                cv.modifiers.command && i.key_pressed(egui::Key::V),
            )
        });

        if copy_pressed {
            let selection = cx.selection().clone();
            let Selection::Tiles { layer_id, cells } = selection else {
                cx.status("Copy: select a tile or IntGrid region first");
                return true;
            };
            self.buffer = cx
                .project()
                .level(cv.active_level_id)
                .and_then(|level| TileClipboard::copy(level, layer_id, &cells));
            match &self.buffer {
                Some(buffer) => cx.status(format!(
                    "Copied {}x{} {} region",
                    buffer.width(),
                    buffer.height(),
                    match buffer.kind() {
                        LayerKind::Tiles => "tile",
                        _ => "IntGrid",
                    }
                )),
                None => cx.status("Copy: selection is not a tile or IntGrid region"),
            }
            return true;
        }

        if paste_pressed {
            let Some(buffer) = &self.buffer else {
                cx.status("Paste: clipboard is empty");
                return true;
            };
            let Some(pos) = cv.grid_pos else {
                cx.status("Paste: move the cursor over the canvas first");
                return true;
            };
            if pos.x < 0 || pos.y < 0 {
                cx.status("Paste: cursor is outside the level grid");
                return true;
            }
            let Some(layer_id) = cv.active_layer_id else {
                cx.status("Paste: no active layer");
                return true;
            };
            let layer_kind = cx
                .project()
                .defs
                .layers
                .iter()
                .find(|l| l.id == layer_id)
                .map(|l| l.kind);
            if layer_kind != Some(buffer.kind()) {
                cx.status("Paste: active layer kind does not match clipboard");
                return true;
            }

            let size = (buffer.width(), buffer.height());
            if let Some(cmd) = buffer.paste_command(
                cv.active_level_id,
                layer_id,
                UVec2::new(pos.x as u32, pos.y as u32),
                lc_canvas::ids::next_stroke_id(),
            ) {
                cx.exec_boxed(cmd);
                cx.status(format!("Pasted {}x{} region", size.0, size.1));
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::command::UndoStack;
    use lc_core::defs::LayerDef;
    use lc_core::grid::Grid;
    use lc_core::layer::LayerInstance;
    use lc_core::project::Project;

    /// Deterministic textual snapshot of every project field.
    ///
    /// `serde_json` is restricted to `lc_io` by the architecture lint, so
    /// byte comparisons use the full `Debug` representation instead.
    fn snapshot(project: &Project) -> String {
        format!("{project:?}")
    }

    fn project_with_layer(kind: LayerKind, grid_size: UVec2) -> (Project, LevelId, LayerDefId) {
        let level_id = LevelId::new(1);
        let layer_id = LayerDefId::new(10);
        let mut project = Project::new("Clipboard");
        project.defs.layers.push(LayerDef {
            id: layer_id,
            name: "Layer".to_string(),
            kind,
            grid_size,
            tileset_id: None,
            int_grid_values: Vec::new(),
        });

        let data = match kind {
            LayerKind::IntGrid => LayerData::IntGrid(Grid::new(8, 8, 0)),
            _ => LayerData::Tiles(Grid::new(8, 8, None)),
        };
        let mut level = Level::new(level_id, "Level", UVec2::new(128, 128));
        level.layers.push(LayerInstance::new(layer_id, data));
        let mut world = lc_core::world::World::new(lc_core::id::WorldId::new(1), "World");
        world.levels.push(level);
        project.worlds.push(world);
        (project, level_id, layer_id)
    }

    #[test]
    fn copy_paste_tiles_round_trips_byte_identical() {
        let (mut project, level_id, layer_id) =
            project_with_layer(LayerKind::Tiles, UVec2::splat(16));
        let tileset = lc_core::id::TilesetDefId::new(7);

        if let Some(level) = project.level_mut(level_id) {
            if let Some(layer) = level.layer_mut(layer_id) {
                if let LayerData::Tiles(grid) = &mut layer.data {
                    grid.set(1, 1, Some(TileRef::new(tileset, 3))).unwrap();
                    grid.set(2, 1, Some(TileRef::new(tileset, 4))).unwrap();
                    grid.set(1, 2, Some(TileRef::new(tileset, 5))).unwrap();
                }
            }
        }

        let before = snapshot(&project);
        let level = project.level(level_id).expect("level");
        let selection = vec![IVec2::new(1, 1), IVec2::new(2, 1), IVec2::new(1, 2)];
        let clipboard = TileClipboard::copy(level, layer_id, &selection).expect("copy");
        assert_eq!((clipboard.width(), clipboard.height()), (2, 2));
        assert_eq!(clipboard.kind(), LayerKind::Tiles);

        let mut undo = UndoStack::new();
        let command = clipboard
            .paste_command(level_id, layer_id, UVec2::new(4, 4), 11)
            .expect("paste command");
        undo.execute(command, &mut project).expect("paste applies");

        {
            let level = project.level(level_id).expect("level");
            let layer = level.layer(layer_id).expect("layer");
            let LayerData::Tiles(grid) = &layer.data else {
                panic!("expected tile layer");
            };
            assert_eq!(
                grid.get(4, 4).copied().flatten(),
                Some(TileRef::new(tileset, 3))
            );
            assert_eq!(
                grid.get(5, 4).copied().flatten(),
                Some(TileRef::new(tileset, 4))
            );
            assert_eq!(
                grid.get(4, 5).copied().flatten(),
                Some(TileRef::new(tileset, 5))
            );
            assert_eq!(grid.get(5, 5).copied().flatten(), None);
        }

        undo.undo(&mut project).expect("paste reverts");
        assert_eq!(snapshot(&project), before, "revert must be byte-identical");
    }

    #[test]
    fn copy_paste_int_grid_round_trips_byte_identical() {
        let (mut project, level_id, layer_id) =
            project_with_layer(LayerKind::IntGrid, UVec2::splat(16));

        if let Some(level) = project.level_mut(level_id) {
            if let Some(layer) = level.layer_mut(layer_id) {
                if let LayerData::IntGrid(grid) = &mut layer.data {
                    grid.set(0, 0, 4).unwrap();
                    grid.set(1, 0, 9).unwrap();
                }
            }
        }

        let before = snapshot(&project);
        let level = project.level(level_id).expect("level");
        let selection = vec![IVec2::new(0, 0), IVec2::new(1, 0)];
        let clipboard = TileClipboard::copy(level, layer_id, &selection).expect("copy");
        assert_eq!(clipboard.kind(), LayerKind::IntGrid);

        let mut undo = UndoStack::new();
        let command = clipboard
            .paste_command(level_id, layer_id, UVec2::new(3, 3), 12)
            .expect("paste command");
        undo.execute(command, &mut project).expect("paste applies");

        {
            let level = project.level(level_id).expect("level");
            let layer = level.layer(layer_id).expect("layer");
            let LayerData::IntGrid(grid) = &layer.data else {
                panic!("expected int grid layer");
            };
            assert_eq!(grid.get(3, 3).copied(), Some(4));
            assert_eq!(grid.get(4, 3).copied(), Some(9));
            assert_eq!(grid.get(3, 4).copied(), Some(0));
        }

        undo.undo(&mut project).expect("paste reverts");
        assert_eq!(snapshot(&project), before, "revert must be byte-identical");
    }

    #[test]
    fn copy_rejects_empty_or_non_grid_layers() {
        let (project, level_id, layer_id) = project_with_layer(LayerKind::Tiles, UVec2::splat(16));
        let level = project.level(level_id).expect("level");
        assert!(TileClipboard::copy(level, layer_id, &[]).is_none());
        assert!(TileClipboard::copy(level, layer_id, &[IVec2::new(-1, 0)]).is_none());
        assert!(TileClipboard::copy(level, LayerDefId::new(999), &[IVec2::ZERO]).is_none());
    }
}
