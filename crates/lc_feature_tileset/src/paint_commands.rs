//! Commands for painting, flood filling, and drawing rectangles on Tiles and IntGrid layers.

use glam::UVec2;
use lc_core::command::{Command, DocKind};
use lc_core::error::CoreError;
use lc_core::id::{LayerDefId, LevelId};
use lc_core::layer::{LayerData, TileRef};
use lc_core::project::Project;

/// Value applied during flood fill or rectangle drawing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillValue {
    /// Tile placement or erasure.
    Tile(Option<TileRef>),
    /// Repeating rectangular tile pattern, anchored at the rectangle's top-left corner.
    TilePattern {
        /// Row-major tile block.
        tiles: Vec<TileRef>,
        /// Pattern width in tiles.
        width: u32,
    },
    /// Integer grid cell value.
    Int(i32),
}

/// Command for painting one or more tile cells on a Tiles layer.
#[derive(Debug, Clone)]
pub struct PaintTilesCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    cells: Vec<(u32, u32, Option<TileRef>)>,
    previous_cells: Vec<(u32, u32, Option<TileRef>)>,
    stroke_id: u64,
}

impl PaintTilesCommand {
    /// Constructs a command to paint a collection of tile cells.
    pub fn new(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        cells: Vec<(u32, u32, Option<TileRef>)>,
    ) -> Self {
        Self::with_stroke(level_id, layer_def_id, cells, 0)
    }

    /// Constructs a command to paint a single tile cell.
    pub fn single(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        x: u32,
        y: u32,
        tile: Option<TileRef>,
    ) -> Self {
        Self::new(level_id, layer_def_id, vec![(x, y, tile)])
    }

    /// Constructs a command with an associated stroke identifier for drag merging.
    pub fn with_stroke(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        cells: Vec<(u32, u32, Option<TileRef>)>,
        stroke_id: u64,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            cells,
            previous_cells: Vec::new(),
            stroke_id,
        }
    }

    /// Accessor for painted cells.
    pub fn cells(&self) -> &[(u32, u32, Option<TileRef>)] {
        &self.cells
    }

    /// Accessor for stroke ID.
    pub fn stroke_id(&self) -> u64 {
        self.stroke_id
    }
}

impl Command for PaintTilesCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;

        if let LayerData::Tiles(ref mut grid) = layer.data {
            let record_previous = self.previous_cells.is_empty();
            for &(x, y, new_tile) in &self.cells {
                if grid.in_bounds(x, y) {
                    if record_previous {
                        let prev = grid.get(x, y).copied().flatten();
                        self.previous_cells.push((x, y, prev));
                    }
                    grid.set(x, y, new_tile)?;
                }
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(level) = p.level_mut(self.level_id) {
            if let Some(layer) = level.layer_mut(self.layer_def_id) {
                if let LayerData::Tiles(ref mut grid) = layer.data {
                    for &(x, y, prev_tile) in self.previous_cells.iter().rev() {
                        let _ = grid.set(x, y, prev_tile);
                    }
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.level_id == other.level_id
                && self.layer_def_id == other.layer_def_id
                && self.stroke_id == other.stroke_id
                && self.stroke_id != 0
            {
                for &(nx, ny, new_val) in &other.cells {
                    if let Some(pos) = self
                        .cells
                        .iter()
                        .position(|(cx, cy, _)| *cx == nx && *cy == ny)
                    {
                        self.cells[pos] = (nx, ny, new_val);
                    } else {
                        self.cells.push((nx, ny, new_val));
                    }
                }

                for &(ox, oy, old_val) in &other.previous_cells {
                    if !self
                        .previous_cells
                        .iter()
                        .any(|(px, py, _)| *px == ox && *py == oy)
                    {
                        self.previous_cells.push((ox, oy, old_val));
                    }
                }
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        let is_erase = self.cells.iter().all(|(_, _, t)| t.is_none());
        if is_erase {
            "Erase Tiles".to_string()
        } else {
            "Paint Tiles".to_string()
        }
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command for painting integer values on an IntGrid layer.
#[derive(Debug, Clone)]
pub struct PaintIntGridCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    cells: Vec<(u32, u32, i32)>,
    previous_cells: Vec<(u32, u32, i32)>,
    stroke_id: u64,
}

impl PaintIntGridCommand {
    /// Constructs a command to paint integer grid cells.
    pub fn new(level_id: LevelId, layer_def_id: LayerDefId, cells: Vec<(u32, u32, i32)>) -> Self {
        Self::with_stroke(level_id, layer_def_id, cells, 0)
    }

    /// Constructs a command to paint a single integer grid cell.
    pub fn single(level_id: LevelId, layer_def_id: LayerDefId, x: u32, y: u32, value: i32) -> Self {
        Self::new(level_id, layer_def_id, vec![(x, y, value)])
    }

    /// Constructs a command with a stroke identifier.
    pub fn with_stroke(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        cells: Vec<(u32, u32, i32)>,
        stroke_id: u64,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            cells,
            previous_cells: Vec::new(),
            stroke_id,
        }
    }
}

impl Command for PaintIntGridCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;

        if let LayerData::IntGrid(ref mut grid) = layer.data {
            let record_previous = self.previous_cells.is_empty();
            for &(x, y, new_val) in &self.cells {
                if grid.in_bounds(x, y) {
                    if record_previous {
                        let prev = grid.get(x, y).copied().unwrap_or(0);
                        self.previous_cells.push((x, y, prev));
                    }
                    grid.set(x, y, new_val)?;
                }
            }
        }
        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(level) = p.level_mut(self.level_id) {
            if let Some(layer) = level.layer_mut(self.layer_def_id) {
                if let LayerData::IntGrid(ref mut grid) = layer.data {
                    for &(x, y, prev_val) in self.previous_cells.iter().rev() {
                        let _ = grid.set(x, y, prev_val);
                    }
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.level_id == other.level_id
                && self.layer_def_id == other.layer_def_id
                && self.stroke_id == other.stroke_id
                && self.stroke_id != 0
            {
                for &(nx, ny, new_val) in &other.cells {
                    if let Some(pos) = self
                        .cells
                        .iter()
                        .position(|(cx, cy, _)| *cx == nx && *cy == ny)
                    {
                        self.cells[pos] = (nx, ny, new_val);
                    } else {
                        self.cells.push((nx, ny, new_val));
                    }
                }

                for &(ox, oy, old_val) in &other.previous_cells {
                    if !self
                        .previous_cells
                        .iter()
                        .any(|(px, py, _)| *px == ox && *py == oy)
                    {
                        self.previous_cells.push((ox, oy, old_val));
                    }
                }
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Paint IntGrid".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command for flood filling an area with tiles or integer values.
#[derive(Debug, Clone)]
pub struct FillCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    start: UVec2,
    fill_value: FillValue,
    previous_cells: Vec<(u32, u32, FillValue)>,
    stroke_id: u64,
}

impl FillCommand {
    /// Constructs a flood fill command for a Tiles layer.
    pub fn tiles(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        start: UVec2,
        tile: Option<TileRef>,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            start,
            fill_value: FillValue::Tile(tile),
            previous_cells: Vec::new(),
            stroke_id: 0,
        }
    }

    /// Constructs a flood fill command for an IntGrid layer.
    pub fn int_grid(level_id: LevelId, layer_def_id: LayerDefId, start: UVec2, val: i32) -> Self {
        Self {
            level_id,
            layer_def_id,
            start,
            fill_value: FillValue::Int(val),
            previous_cells: Vec::new(),
            stroke_id: 0,
        }
    }

    /// Constructs a flood fill command with stroke identifier.
    pub fn with_stroke(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        start: UVec2,
        fill_value: FillValue,
        stroke_id: u64,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            start,
            fill_value,
            previous_cells: Vec::new(),
            stroke_id,
        }
    }
}

impl Command for FillCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;

        let record_previous = self.previous_cells.is_empty();

        match (&self.fill_value, &mut layer.data) {
            (FillValue::Tile(target_tile), LayerData::Tiles(ref mut grid)) => {
                if !grid.in_bounds(self.start.x, self.start.y) {
                    return Ok(());
                }

                let origin_tile = grid.get(self.start.x, self.start.y).copied().flatten();
                if origin_tile == *target_tile {
                    return Ok(());
                }

                // If this is a redo, apply directly from previous_cells coordinates
                if !record_previous {
                    for &(x, y, _) in &self.previous_cells {
                        grid.set(x, y, *target_tile)?;
                    }
                    return Ok(());
                }

                let width = grid.width();
                let height = grid.height();
                let total = (width as usize).saturating_mul(height as usize);
                let mut visited = vec![false; total];
                let mut stack = Vec::new();

                let start_idx =
                    (self.start.y as usize) * (width as usize) + (self.start.x as usize);
                visited[start_idx] = true;
                stack.push(self.start);

                while let Some(pos) = stack.pop() {
                    let current_tile = grid.get(pos.x, pos.y).copied().flatten();
                    if current_tile != origin_tile {
                        continue;
                    }

                    self.previous_cells
                        .push((pos.x, pos.y, FillValue::Tile(origin_tile)));
                    grid.set(pos.x, pos.y, *target_tile)?;

                    let (x, y) = (pos.x, pos.y);
                    if x > 0 {
                        let nx = x - 1;
                        let idx = (y as usize) * (width as usize) + (nx as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(nx, y).copied().flatten() == origin_tile {
                                stack.push(UVec2::new(nx, y));
                            }
                        }
                    }
                    if x + 1 < width {
                        let nx = x + 1;
                        let idx = (y as usize) * (width as usize) + (nx as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(nx, y).copied().flatten() == origin_tile {
                                stack.push(UVec2::new(nx, y));
                            }
                        }
                    }
                    if y > 0 {
                        let ny = y - 1;
                        let idx = (ny as usize) * (width as usize) + (x as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(x, ny).copied().flatten() == origin_tile {
                                stack.push(UVec2::new(x, ny));
                            }
                        }
                    }
                    if y + 1 < height {
                        let ny = y + 1;
                        let idx = (ny as usize) * (width as usize) + (x as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(x, ny).copied().flatten() == origin_tile {
                                stack.push(UVec2::new(x, ny));
                            }
                        }
                    }
                }
            }
            (FillValue::Int(target_val), LayerData::IntGrid(ref mut grid)) => {
                if !grid.in_bounds(self.start.x, self.start.y) {
                    return Ok(());
                }

                let origin_val = grid.get(self.start.x, self.start.y).copied().unwrap_or(0);
                if origin_val == *target_val {
                    return Ok(());
                }

                if !record_previous {
                    for &(x, y, _) in &self.previous_cells {
                        grid.set(x, y, *target_val)?;
                    }
                    return Ok(());
                }

                let width = grid.width();
                let height = grid.height();
                let total = (width as usize).saturating_mul(height as usize);
                let mut visited = vec![false; total];
                let mut stack = Vec::new();

                let start_idx =
                    (self.start.y as usize) * (width as usize) + (self.start.x as usize);
                visited[start_idx] = true;
                stack.push(self.start);

                while let Some(pos) = stack.pop() {
                    let current_val = grid.get(pos.x, pos.y).copied().unwrap_or(0);
                    if current_val != origin_val {
                        continue;
                    }

                    self.previous_cells
                        .push((pos.x, pos.y, FillValue::Int(origin_val)));
                    grid.set(pos.x, pos.y, *target_val)?;

                    let (x, y) = (pos.x, pos.y);
                    if x > 0 {
                        let nx = x - 1;
                        let idx = (y as usize) * (width as usize) + (nx as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(nx, y).copied().unwrap_or(0) == origin_val {
                                stack.push(UVec2::new(nx, y));
                            }
                        }
                    }
                    if x + 1 < width {
                        let nx = x + 1;
                        let idx = (y as usize) * (width as usize) + (nx as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(nx, y).copied().unwrap_or(0) == origin_val {
                                stack.push(UVec2::new(nx, y));
                            }
                        }
                    }
                    if y > 0 {
                        let ny = y - 1;
                        let idx = (ny as usize) * (width as usize) + (x as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(x, ny).copied().unwrap_or(0) == origin_val {
                                stack.push(UVec2::new(x, ny));
                            }
                        }
                    }
                    if y + 1 < height {
                        let ny = y + 1;
                        let idx = (ny as usize) * (width as usize) + (x as usize);
                        if !visited[idx] {
                            visited[idx] = true;
                            if grid.get(x, ny).copied().unwrap_or(0) == origin_val {
                                stack.push(UVec2::new(x, ny));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(level) = p.level_mut(self.level_id) {
            if let Some(layer) = level.layer_mut(self.layer_def_id) {
                match &mut layer.data {
                    LayerData::Tiles(ref mut grid) => {
                        for &(x, y, ref prev) in self.previous_cells.iter().rev() {
                            if let FillValue::Tile(t) = prev {
                                let _ = grid.set(x, y, *t);
                            }
                        }
                    }
                    LayerData::IntGrid(ref mut grid) => {
                        for &(x, y, ref prev) in self.previous_cells.iter().rev() {
                            if let FillValue::Int(v) = prev {
                                let _ = grid.set(x, y, *v);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.level_id == other.level_id
                && self.layer_def_id == other.layer_def_id
                && self.stroke_id == other.stroke_id
                && self.stroke_id != 0
            {
                for (ox, oy, old_val) in &other.previous_cells {
                    if !self
                        .previous_cells
                        .iter()
                        .any(|(px, py, _)| px == ox && py == oy)
                    {
                        self.previous_cells.push((*ox, *oy, old_val.clone()));
                    }
                }
                self.fill_value = other.fill_value.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Flood Fill".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Command for filling a rectangular region on Tiles or IntGrid layers.
#[derive(Debug, Clone)]
pub struct RectCommand {
    level_id: LevelId,
    layer_def_id: LayerDefId,
    from: UVec2,
    to: UVec2,
    value: FillValue,
    previous_cells: Vec<(u32, u32, FillValue)>,
    stroke_id: u64,
}

impl RectCommand {
    /// Constructs a rectangle command for a Tiles layer.
    pub fn tiles(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        from: UVec2,
        to: UVec2,
        tile: Option<TileRef>,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            from,
            to,
            value: FillValue::Tile(tile),
            previous_cells: Vec::new(),
            stroke_id: 0,
        }
    }

    /// Constructs a rectangle command that repeats a multi-tile pattern across the region.
    pub fn tiles_pattern(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        from: UVec2,
        to: UVec2,
        tiles: Vec<TileRef>,
        width: u32,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            from,
            to,
            value: FillValue::TilePattern {
                tiles,
                width: width.max(1),
            },
            previous_cells: Vec::new(),
            stroke_id: 0,
        }
    }

    /// Constructs a rectangle command for an IntGrid layer.
    pub fn int_grid(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        from: UVec2,
        to: UVec2,
        val: i32,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            from,
            to,
            value: FillValue::Int(val),
            previous_cells: Vec::new(),
            stroke_id: 0,
        }
    }

    /// Constructs a rectangle command with stroke identifier.
    pub fn with_stroke(
        level_id: LevelId,
        layer_def_id: LayerDefId,
        from: UVec2,
        to: UVec2,
        value: FillValue,
        stroke_id: u64,
    ) -> Self {
        Self {
            level_id,
            layer_def_id,
            from,
            to,
            value,
            previous_cells: Vec::new(),
            stroke_id,
        }
    }
}

impl Command for RectCommand {
    fn apply(&mut self, p: &mut Project) -> Result<(), CoreError> {
        let level = p
            .level_mut(self.level_id)
            .ok_or(CoreError::LevelNotFound(self.level_id))?;
        let layer = level
            .layer_mut(self.layer_def_id)
            .ok_or(CoreError::LayerNotFound(self.layer_def_id))?;

        let min_x = self.from.x.min(self.to.x);
        let max_x = self.from.x.max(self.to.x);
        let min_y = self.from.y.min(self.to.y);
        let max_y = self.from.y.max(self.to.y);

        let record_previous = self.previous_cells.is_empty();

        match (&self.value, &mut layer.data) {
            (FillValue::Tile(target_tile), LayerData::Tiles(ref mut grid)) => {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if grid.in_bounds(x, y) {
                            if record_previous {
                                let prev = grid.get(x, y).copied().flatten();
                                self.previous_cells.push((x, y, FillValue::Tile(prev)));
                            }
                            grid.set(x, y, *target_tile)?;
                        }
                    }
                }
            }
            (FillValue::TilePattern { tiles, width }, LayerData::Tiles(ref mut grid)) => {
                let pattern_width = (*width).max(1);
                let pattern_height = (tiles.len() as u32).div_ceil(pattern_width).max(1);
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if !grid.in_bounds(x, y) {
                            continue;
                        }
                        let dx = (x - min_x) % pattern_width;
                        let dy = (y - min_y) % pattern_height;
                        let index = dy * pattern_width + dx;
                        let Some(tile) = tiles.get(index as usize).copied() else {
                            continue;
                        };
                        if record_previous {
                            let prev = grid.get(x, y).copied().flatten();
                            self.previous_cells.push((x, y, FillValue::Tile(prev)));
                        }
                        grid.set(x, y, Some(tile))?;
                    }
                }
            }
            (FillValue::Int(target_val), LayerData::IntGrid(ref mut grid)) => {
                for y in min_y..=max_y {
                    for x in min_x..=max_x {
                        if grid.in_bounds(x, y) {
                            if record_previous {
                                let prev = grid.get(x, y).copied().unwrap_or(0);
                                self.previous_cells.push((x, y, FillValue::Int(prev)));
                            }
                            grid.set(x, y, *target_val)?;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn revert(&mut self, p: &mut Project) {
        if let Some(level) = p.level_mut(self.level_id) {
            if let Some(layer) = level.layer_mut(self.layer_def_id) {
                match &mut layer.data {
                    LayerData::Tiles(ref mut grid) => {
                        for &(x, y, ref prev) in self.previous_cells.iter().rev() {
                            if let FillValue::Tile(t) = prev {
                                let _ = grid.set(x, y, *t);
                            }
                        }
                    }
                    LayerData::IntGrid(ref mut grid) => {
                        for &(x, y, ref prev) in self.previous_cells.iter().rev() {
                            if let FillValue::Int(v) = prev {
                                let _ = grid.set(x, y, *v);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn merge(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<Self>() {
            if self.level_id == other.level_id
                && self.layer_def_id == other.layer_def_id
                && self.stroke_id == other.stroke_id
                && self.stroke_id != 0
            {
                self.to = other.to;
                for (ox, oy, old_val) in &other.previous_cells {
                    if !self
                        .previous_cells
                        .iter()
                        .any(|(px, py, _)| px == ox && py == oy)
                    {
                        self.previous_cells.push((*ox, *oy, old_val.clone()));
                    }
                }
                self.value = other.value.clone();
                return true;
            }
        }
        false
    }

    fn label(&self) -> String {
        "Draw Rectangle".to_string()
    }

    fn focus(&self) -> Option<DocKind> {
        Some(DocKind::Level(self.level_id))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec2;
    use lc_core::command::UndoStack;
    use lc_core::defs::LayerDef;
    use lc_core::grid::Grid;
    use lc_core::id::{LayerDefId, LevelId, TilesetDefId};
    use lc_core::layer::{LayerInstance, LayerKind, TileRef};
    use lc_core::level::Level;

    fn create_test_project_with_tile_layer(w: u32, h: u32) -> (Project, LevelId, LayerDefId) {
        let mut p = Project::new("Test");
        let level_id = LevelId::new(1);
        let layer_def_id = LayerDefId::new(10);

        let layer_def = LayerDef {
            id: layer_def_id,
            name: "TilesLayer".to_string(),
            kind: LayerKind::Tiles,
            grid_size: UVec2::new(16, 16),
            tileset_id: None,
            int_grid_values: Vec::new(),
        };
        p.defs.layers.push(layer_def);

        let mut level = Level::new(level_id, "Level_0", UVec2::new(w * 16, h * 16));
        let grid = Grid::new(w, h, None);
        level
            .layers
            .push(LayerInstance::new(layer_def_id, LayerData::Tiles(grid)));
        let mut world = lc_core::world::World::new(lc_core::id::WorldId::new(1), "World");
        world.levels.push(level);
        p.worlds.push(world);

        (p, level_id, layer_def_id)
    }

    fn create_test_project_with_intgrid_layer(w: u32, h: u32) -> (Project, LevelId, LayerDefId) {
        let mut p = Project::new("Test");
        let level_id = LevelId::new(2);
        let layer_def_id = LayerDefId::new(20);

        let layer_def = LayerDef {
            id: layer_def_id,
            name: "IntGridLayer".to_string(),
            kind: LayerKind::IntGrid,
            grid_size: UVec2::new(16, 16),
            tileset_id: None,
            int_grid_values: Vec::new(),
        };
        p.defs.layers.push(layer_def);

        let mut level = Level::new(level_id, "Level_0", UVec2::new(w * 16, h * 16));
        let grid = Grid::new(w, h, 0);
        level
            .layers
            .push(LayerInstance::new(layer_def_id, LayerData::IntGrid(grid)));
        let mut world = lc_core::world::World::new(lc_core::id::WorldId::new(1), "World");
        world.levels.push(level);
        p.worlds.push(world);

        (p, level_id, layer_def_id)
    }

    #[test]
    fn test_paint_tiles_command_apply_revert_merge() {
        let (mut p, level_id, layer_def_id) = create_test_project_with_tile_layer(10, 10);
        let mut undo = UndoStack::new();

        let tile_a = Some(TileRef::new(TilesetDefId::new(1), 5));
        let tile_b = Some(TileRef::new(TilesetDefId::new(1), 8));

        let cmd1 = PaintTilesCommand::with_stroke(
            level_id,
            layer_def_id,
            vec![(1, 1, tile_a), (2, 2, tile_a)],
            42,
        );

        undo.execute(Box::new(cmd1), &mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(1, 1).copied().flatten(), tile_a);
                assert_eq!(grid.get(2, 2).copied().flatten(), tile_a);
                assert_eq!(grid.get(3, 3).copied().flatten(), None);
            } else {
                panic!("Expected Tiles layer");
            }
        }

        // Merge contiguous stroke
        let cmd2 = PaintTilesCommand::with_stroke(level_id, layer_def_id, vec![(3, 3, tile_b)], 42);
        undo.execute(Box::new(cmd2), &mut p).unwrap();

        assert!(undo.can_undo(), "Expected undo to be available");

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(1, 1).copied().flatten(), tile_a);
                assert_eq!(grid.get(2, 2).copied().flatten(), tile_a);
                assert_eq!(grid.get(3, 3).copied().flatten(), tile_b);
            }
        }

        // Revert entire stroke: exactly one undo step should clear all painted cells
        undo.undo(&mut p).unwrap();
        assert!(
            !undo.can_undo(),
            "Expected stroke to have merged into exactly one undo step"
        );

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(1, 1).copied().flatten(), None);
                assert_eq!(grid.get(2, 2).copied().flatten(), None);
                assert_eq!(grid.get(3, 3).copied().flatten(), None);
            }
        }

        // Redo entire stroke
        undo.redo(&mut p).unwrap();
        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(1, 1).copied().flatten(), tile_a);
                assert_eq!(grid.get(2, 2).copied().flatten(), tile_a);
                assert_eq!(grid.get(3, 3).copied().flatten(), tile_b);
            }
        }
    }

    #[test]
    fn test_rect_command_apply_revert_merge() {
        let (mut p, level_id, layer_def_id) = create_test_project_with_tile_layer(10, 10);
        let mut undo = UndoStack::new();

        let tile = Some(TileRef::new(TilesetDefId::new(1), 12));
        let cmd1 = RectCommand::with_stroke(
            level_id,
            layer_def_id,
            UVec2::new(1, 1),
            UVec2::new(3, 3),
            FillValue::Tile(tile),
            99,
        );

        undo.execute(Box::new(cmd1), &mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                for y in 1..=3 {
                    for x in 1..=3 {
                        assert_eq!(grid.get(x, y).copied().flatten(), tile);
                    }
                }
                assert_eq!(grid.get(0, 0).copied().flatten(), None);
                assert_eq!(grid.get(4, 4).copied().flatten(), None);
            }
        }

        // Merge updated drag rectangle
        let cmd2 = RectCommand::with_stroke(
            level_id,
            layer_def_id,
            UVec2::new(1, 1),
            UVec2::new(4, 4),
            FillValue::Tile(tile),
            99,
        );
        undo.execute(Box::new(cmd2), &mut p).unwrap();
        assert!(undo.can_undo());

        // Revert: exactly one undo step should revert all cells
        undo.undo(&mut p).unwrap();
        assert!(
            !undo.can_undo(),
            "Expected rect drag to have merged into single undo step"
        );
        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                for y in 0..10 {
                    for x in 0..10 {
                        assert_eq!(grid.get(x, y).copied().flatten(), None);
                    }
                }
            }
        }
    }

    #[test]
    fn test_flood_fill_iterative_1024x1024_no_stack_overflow() {
        // Acceptance criterion: Flood fill is iterative (no recursion overflow) and tested on a 1024x1024 grid.
        let (mut p, level_id, layer_def_id) = create_test_project_with_tile_layer(1024, 1024);
        let mut undo = UndoStack::new();

        let tile = Some(TileRef::new(TilesetDefId::new(1), 999));
        let cmd = FillCommand::tiles(level_id, layer_def_id, UVec2::new(512, 512), tile);

        // Apply on 1024x1024 grid without recursion overflow
        undo.execute(Box::new(cmd), &mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(0, 0).copied().flatten(), tile);
                assert_eq!(grid.get(512, 512).copied().flatten(), tile);
                assert_eq!(grid.get(1023, 1023).copied().flatten(), tile);
            }
        }

        // Revert restores 1024x1024 to None
        undo.undo(&mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::Tiles(grid) = &layer.data {
                assert_eq!(grid.get(0, 0).copied().flatten(), None);
                assert_eq!(grid.get(512, 512).copied().flatten(), None);
                assert_eq!(grid.get(1023, 1023).copied().flatten(), None);
            }
        }
    }

    #[test]
    fn test_flood_fill_intgrid_bounded_barrier() {
        let (mut p, level_id, layer_def_id) = create_test_project_with_intgrid_layer(10, 10);
        let mut undo = UndoStack::new();

        // Draw a barrier wall of 1s in column 5
        let barrier =
            PaintIntGridCommand::new(level_id, layer_def_id, (0..10).map(|y| (5, y, 1)).collect());
        undo.execute(Box::new(barrier), &mut p).unwrap();

        // Flood fill on left side (x=2, y=2) with value 9
        let fill = FillCommand::int_grid(level_id, layer_def_id, UVec2::new(2, 2), 9);
        undo.execute(Box::new(fill), &mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::IntGrid(grid) = &layer.data {
                // Left side is filled
                assert_eq!(grid.get(2, 2).copied(), Some(9));
                assert_eq!(grid.get(0, 0).copied(), Some(9));
                // Barrier remains
                assert_eq!(grid.get(5, 5).copied(), Some(1));
                // Right side is untouched
                assert_eq!(grid.get(6, 6).copied(), Some(0));
                assert_eq!(grid.get(9, 9).copied(), Some(0));
            }
        }

        // Revert fill
        undo.undo(&mut p).unwrap();
        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            if let LayerData::IntGrid(grid) = &layer.data {
                assert_eq!(grid.get(2, 2).copied(), Some(0));
                assert_eq!(grid.get(5, 5).copied(), Some(1));
            }
        }
    }

    /// Deterministic textual snapshot of every project field.
    ///
    /// `serde_json` is restricted to `lc_io` by the architecture lint, so
    /// byte comparisons use the full `Debug` representation instead.
    fn snapshot(project: &Project) -> String {
        format!("{project:?}")
    }

    #[test]
    fn test_multi_tile_paint_revert_is_byte_identical() {
        let (mut p, level_id, layer_def_id) = create_test_project_with_tile_layer(10, 10);
        let before = snapshot(&p);
        let mut undo = UndoStack::new();

        let tileset = TilesetDefId::new(1);
        let cells = vec![
            (1, 1, Some(TileRef::new(tileset, 10))),
            (2, 1, Some(TileRef::new(tileset, 11))),
            (1, 2, Some(TileRef::new(tileset, 12))),
            (2, 2, Some(TileRef::new(tileset, 13))),
        ];
        let cmd = PaintTilesCommand::with_stroke(level_id, layer_def_id, cells, 77);
        undo.execute(Box::new(cmd), &mut p).unwrap();
        assert_ne!(
            snapshot(&p),
            before,
            "multi-tile stamp must mutate the project"
        );

        undo.undo(&mut p).unwrap();
        assert_eq!(
            snapshot(&p),
            before,
            "multi-tile paint revert must be byte-identical"
        );
    }

    #[test]
    fn test_rect_multi_tile_pattern_tiles_and_reverts_byte_identically() {
        let (mut p, level_id, layer_def_id) = create_test_project_with_tile_layer(10, 10);
        let before = snapshot(&p);
        let mut undo = UndoStack::new();

        let tileset = TilesetDefId::new(1);
        let pattern: Vec<TileRef> = [10u32, 11, 12, 13]
            .into_iter()
            .map(|tile_id| TileRef::new(tileset, tile_id))
            .collect();
        let cmd = RectCommand::tiles_pattern(
            level_id,
            layer_def_id,
            UVec2::new(1, 1),
            UVec2::new(4, 4),
            pattern,
            2,
        );
        undo.execute(Box::new(cmd), &mut p).unwrap();

        {
            let level = p.level(level_id).unwrap();
            let layer = level.layer(layer_def_id).unwrap();
            let LayerData::Tiles(grid) = &layer.data else {
                panic!("Expected Tiles layer");
            };
            // The 2x2 pattern repeats across the 4x4 rectangle.
            for (x, y, expected) in [
                (1, 1, 10),
                (2, 1, 11),
                (3, 1, 10),
                (4, 1, 11),
                (1, 2, 12),
                (2, 2, 13),
                (3, 2, 12),
                (4, 2, 13),
                (3, 3, 10),
                (4, 4, 13),
            ] {
                assert_eq!(
                    grid.get(x, y).copied().flatten().map(|t| t.tile_id),
                    Some(expected),
                    "unexpected pattern tile at ({x}, {y})"
                );
            }
        }

        undo.undo(&mut p).unwrap();
        assert_eq!(
            snapshot(&p),
            before,
            "rect pattern revert must be byte-identical"
        );
    }
}
