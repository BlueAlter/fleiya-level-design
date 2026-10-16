//! Dense grid container for layer cell storage.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Dense 2D grid storing elements of type `T` in row-major order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid<T> {
    width: u32,
    height: u32,
    cells: Vec<T>,
}

impl<T> Grid<T> {
    /// Creates a new grid filled with cloned instances of `default_val`.
    pub fn new(width: u32, height: u32, default_val: T) -> Self
    where
        T: Clone,
    {
        let len = (width as usize).saturating_mul(height as usize);
        let cells = vec![default_val; len];
        Self {
            width,
            height,
            cells,
        }
    }

    /// Constructs a grid from an existing vector of cells after validating the dimensions.
    pub fn from_cells(width: u32, height: u32, cells: Vec<T>) -> Result<Self, CoreError> {
        let expected = (width as usize).saturating_mul(height as usize);
        if cells.len() != expected {
            return Err(CoreError::InvalidGridSize {
                expected,
                actual: cells.len(),
            });
        }
        Ok(Self {
            width,
            height,
            cells,
        })
    }

    /// Returns the width of the grid in cells.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the grid in cells.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns `true` if `(x, y)` is within the grid boundaries.
    pub fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Computes the 1D linear index for the given 2D coordinates.
    pub fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if self.in_bounds(x, y) {
            let row_offset = (y as usize).checked_mul(self.width as usize)?;
            row_offset.checked_add(x as usize)
        } else {
            None
        }
    }

    /// Returns an immutable reference to the cell at `(x, y)`, or `None` if out of bounds.
    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        let idx = self.index_of(x, y)?;
        self.cells.get(idx)
    }

    /// Returns a mutable reference to the cell at `(x, y)`, or `None` if out of bounds.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut T> {
        let idx = self.index_of(x, y)?;
        self.cells.get_mut(idx)
    }

    /// Sets the value of the cell at `(x, y)`. Returns an error if coordinates are out of bounds.
    pub fn set(&mut self, x: u32, y: u32, value: T) -> Result<(), CoreError> {
        let width = self.width;
        let height = self.height;
        if let Some(cell) = self.get_mut(x, y) {
            *cell = value;
            Ok(())
        } else {
            Err(CoreError::OutOfBounds {
                x,
                y,
                width,
                height,
            })
        }
    }

    /// Returns a slice over all cells in row-major order.
    pub fn cells(&self) -> &[T] {
        self.cells.as_slice()
    }

    /// Returns a mutable slice over all cells in row-major order.
    pub fn cells_mut(&mut self) -> &mut [T] {
        self.cells.as_mut_slice()
    }
}
