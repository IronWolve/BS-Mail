//! Making the window.
//!
//! The first size is the reference grid at the current text size, cut to the
//! monitor's usable area, and so is the smallest size. The title comes from the binary; nothing here knows
//! the program's name.

use std::sync::Arc;

use surface::HostError;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::font::CellMetrics;

use super::geometry::{MonitorArea, minimum_pixels, window_pixels};
use super::options::WindowOptions;

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

/// What the window asks for: title, first size, smallest size and place, all
/// in logical pixels (the place in physical ones) so the display's own scaling
/// applies them, and whether it opens maximized. Both sizes are cut to the
/// monitor's usable area, and the window opens centered on it. A window system
/// that does not let a program place a window ignores the position.
fn attributes(
    event_loop: &ActiveEventLoop,
    options: &WindowOptions,
    metrics: &CellMetrics,
) -> WindowAttributes {
    let monitor = monitor_area(event_loop);
    let limit = monitor.map(|area| area.usable());
    let (width, height) = window_pixels(metrics, options.cols, options.rows, limit);
    let (least_width, least_height) = minimum_pixels(metrics, options.min_cols, options.min_rows, limit);
    let attributes = Window::default_attributes()
        .with_title(options.title.clone())
        .with_inner_size(LogicalSize::new(width, height))
        .with_min_inner_size(LogicalSize::new(least_width, least_height))
        .with_maximized(options.maximized);
    match monitor {
        Some(area) if !options.maximized => {
            let (x, y) = area.centered((width, height));
            attributes.with_position(PhysicalPosition::new(x, y))
        }
        _ => attributes,
    }
}

/// The monitor the window opens on, when one is known.
fn monitor_area(event_loop: &ActiveEventLoop) -> Option<MonitorArea> {
    let monitor = event_loop
        .primary_monitor()
        .or_else(|| event_loop.available_monitors().next())?;
    let origin = monitor.position();
    let size = monitor.size();
    Some(MonitorArea {
        origin: (origin.x, origin.y),
        size: (size.width, size.height),
        scale: monitor.scale_factor(),
    })
}
