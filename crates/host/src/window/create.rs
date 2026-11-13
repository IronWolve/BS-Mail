//! Making the window.
//!
//! The first size is the reference grid at the current text size, cut to the
//! monitor's usable area. The title comes from the binary; nothing here knows
//! the program's name.

use std::sync::Arc;

use surface::HostError;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::font::CellMetrics;

use super::geometry::{minimum_pixels, window_pixels};
use super::options::WindowOptions;

/// How much of the monitor the first window may take.
const SCREEN_SHARE: f64 = 0.92;

/// Opens the window for `options`, measured with `metrics` at scale factor 1.
pub fn create_window(
    event_loop: &ActiveEventLoop,
    options: &WindowOptions,
    metrics: &CellMetrics,
) -> Result<Arc<Window>, HostError> {
    let window = event_loop
        .create_window(attributes(event_loop, options, metrics))
        .map_err(|error| HostError(format!("the window did not open: {error}")))?;
    Ok(Arc::new(window))
}

/// What the window asks for: title, first size and smallest size, all in
/// logical pixels so the display's own scaling applies them.
fn attributes(
    event_loop: &ActiveEventLoop,
    options: &WindowOptions,
    metrics: &CellMetrics,
) -> WindowAttributes {
    let limit = work_area(event_loop);
    let (width, height) = window_pixels(metrics, options.cols, options.rows, limit);
    let (least_width, least_height) = minimum_pixels(metrics, options.min_cols, options.min_rows);
    Window::default_attributes()
        .with_title(options.title.clone())
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(least_width, least_height))
}

/// The usable part of the monitor, in logical pixels, when one is known.
fn work_area(event_loop: &ActiveEventLoop) -> Option<(u32, u32)> {
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())?;
    let size = monitor.size();
    let scale = monitor.scale_factor().max(0.1);
    let logical = |pixels: u32| (f64::from(pixels) / scale * SCREEN_SHARE).max(1.0) as u32;
    Some((logical(size.width), logical(size.height)))
}
