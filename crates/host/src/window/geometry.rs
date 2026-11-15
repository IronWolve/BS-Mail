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

/// The smallest the window may become: the minimum grid at this text size,
/// and never more than `limit` when one is known. A large text size makes the
/// minimum grid bigger than a small screen; a minimum the screen cannot hold
/// would stop the window from being brought back onto it.
pub fn minimum_pixels(
    metrics: &CellMetrics,
    cols: u16,
    rows: u16,
    limit: Option<(u32, u32)>,
) -> (u32, u32) {
    let (width, height) = metrics.grid_pixels(cols.max(1), rows.max(1));
    match limit {
        Some((max_width, max_height)) => (width.min(max_width.max(1)), height.min(max_height.max(1))),
        None => (width, height),
    }
}

/// A monitor as the window library reports it: where it starts and how big it
/// is, in physical pixels, and its scale factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorArea {
    pub origin: (i32, i32),
    pub size: (u32, u32),
    pub scale: f64,
}

impl MonitorArea {
    /// How much of the monitor a first window may take. The title bar, a
    /// taskbar or a dock come on top of the window's inner size, and the
    /// window library cannot say how big they are, so the rest is left over.
    pub const SHARE: f64 = 0.92;

    /// The most a window's inner size may be, in logical pixels.
    pub fn usable(&self) -> (u32, u32) {
        let scale = self.scale.max(0.1);
        let logical = |pixels: u32| (f64::from(pixels) / scale * Self::SHARE).max(1.0) as u32;
        (logical(self.size.0), logical(self.size.1))
    }

    /// Where a window of this inner size (logical pixels) sits centered on the
    /// monitor, as a physical position that never starts before the monitor.
    pub fn centered(&self, window: (u32, u32)) -> (i32, i32) {
        let scale = self.scale.max(0.1);
        let place = |origin: i32, room: u32, inner: u32| {
            let inner = (f64::from(inner) * scale).round() as i64;
            let spare = (i64::from(room) - inner).max(0) / 2;
            (i64::from(origin) + spare).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
        };
        (
            place(self.origin.0, self.size.0, window.0),
            place(self.origin.1, self.size.1, window.1),
        )
    }
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
