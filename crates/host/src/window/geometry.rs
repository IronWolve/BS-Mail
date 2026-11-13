//! The window's size in pixels, from the grid it should show.
//!
//! First start asks for the reference grid at the current text size, cut down
//! to what the monitor's usable area allows, so the window always opens whole
//! on a small screen.

use crate::font::CellMetrics;

/// The pixel size of a `cols` x `rows` grid, never larger than `limit` when
/// one is known.
pub fn window_pixels(
    metrics: &CellMetrics,
    cols: u16,
    rows: u16,
    limit: Option<(u32, u32)>,
) -> (u32, u32) {
    let (width, height) = metrics.grid_pixels(cols, rows);
    match limit {
        Some((max_width, max_height)) => (width.min(max_width), height.min(max_height)),
        None => (width, height),
    }
}

/// The smallest the window may become: the minimum grid at this text size.
pub fn minimum_pixels(metrics: &CellMetrics, cols: u16, rows: u16) -> (u32, u32) {
    metrics.grid_pixels(cols.max(1), rows.max(1))
}

/// The grid a window of this pixel size holds, never below the minimum, so
/// the screen's own "window too small" line has room to show.
pub fn grid_for(metrics: &CellMetrics, size: (u32, u32), least: (u16, u16)) -> (u16, u16) {
    let (cols, rows) = metrics.grid_cells(size.0, size.1);
    (cols.max(least.0), rows.max(least.1))
}

#[cfg(test)]
include!("geometry.tests.rs");
