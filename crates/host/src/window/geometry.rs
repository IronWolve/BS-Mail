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

/// The grid a window of this pixel size holds, as it is: when fewer cells fit
/// than the screen needs (a small window, or the text zoomed up), the screen
/// is told so and draws its own "window too small" line in what there is,
/// instead of laying a full screen out over a canvas too small for it and
/// clipping the controls. One cell is the least it can be.
pub fn grid_for(metrics: &CellMetrics, size: (u32, u32)) -> (u16, u16) {
    let (cols, rows) = metrics.grid_cells(size.0, size.1);
    (cols.max(1), rows.max(1))
}

#[cfg(test)]
include!("geometry.tests.rs");
