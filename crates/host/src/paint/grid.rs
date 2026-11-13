//! Where a cell sits in pixels.
//!
//! The grid is whole cells; the pixels a grid does not fill become an even
//! margin in the background color, so the screen stays centered in the window.

use surface::Rect;

use crate::font::CellMetrics;

/// A rectangle of pixels on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    /// The same rectangle grown by `pixels` on every side.
    pub fn grow(&self, pixels: u32) -> Self {
        Self::new(
            self.x - pixels as i32,
            self.y - pixels as i32,
            self.width + pixels * 2,
            self.height + pixels * 2,
        )
    }

    /// The same rectangle pulled in by `pixels` on every side, or nothing
    /// when that leaves no room.
    pub fn shrink(&self, pixels: u32) -> Option<Self> {
        let width = self.width.checked_sub(pixels * 2)?;
        let height = self.height.checked_sub(pixels * 2)?;
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self::new(
            self.x + pixels as i32,
            self.y + pixels as i32,
            width,
            height,
        ))
    }

    /// The part the two rectangles share.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= left || bottom <= top {
            return None;
        }
        Some(Self::new(
            left,
            top,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    pub metrics: CellMetrics,
    /// Pixels from the canvas's top left to the first cell.
    pub origin: (i32, i32),
}

impl Grid {
    pub fn new(metrics: CellMetrics, origin: (i32, i32)) -> Self {
        Self { metrics, origin }
    }

    /// A `cols` x `rows` grid centered in a pixel box, the leftover split
    /// evenly on both sides.
    pub fn centered(metrics: CellMetrics, size: (u32, u32), cols: u16, rows: u16) -> Self {
        let (width, height) = metrics.grid_pixels(cols, rows);
        let left = (i64::from(size.0) - i64::from(width)).max(0) / 2;
        let top = (i64::from(size.1) - i64::from(height)).max(0) / 2;
        Self::new(metrics, (left as i32, top as i32))
    }

    pub fn x(&self, col: u16) -> i32 {
        self.origin.0 + (u32::from(col) * self.metrics.width) as i32
    }

    pub fn y(&self, row: u16) -> i32 {
        self.origin.1 + (u32::from(row) * self.metrics.height) as i32
    }

    /// The pixel box of one cell.
    pub fn cell(&self, col: u16, row: u16) -> PixelRect {
        PixelRect::new(self.x(col), self.y(row), self.metrics.width, self.metrics.height)
    }

    /// The pixel box of a run of cells.
    pub fn area(&self, area: Rect) -> PixelRect {
        PixelRect::new(
            self.x(area.x),
            self.y(area.y),
            u32::from(area.width) * self.metrics.width,
            u32::from(area.height) * self.metrics.height,
        )
    }

    /// A run of `cells` cells starting at one cell.
    pub fn slot(&self, col: u16, row: u16, cells: u16) -> PixelRect {
        PixelRect::new(
            self.x(col),
            self.y(row),
            u32::from(cells) * self.metrics.width,
            self.metrics.height,
        )
    }

    /// The baseline of a row, in canvas pixels.
    pub fn baseline(&self, row: u16) -> i32 {
        self.y(row) + self.metrics.baseline
    }
}

#[cfg(test)]
include!("grid.tests.rs");
