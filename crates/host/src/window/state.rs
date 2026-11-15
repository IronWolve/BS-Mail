//! What the window keeps between frames, and the presenter the screen code
//! talks to: the faces, the glyph cache, the cell size, the canvas and the
//! grid the window currently holds.

use std::sync::Arc;

use surface::{Capabilities, Color, Frame, HostError, PointerShape, Presenter, Waker};
use winit::event_loop::EventLoopProxy;
use winit::window::{CursorIcon, Fullscreen, Window};

use crate::clipboard::SystemClipboard;
use crate::font::{CellMetrics, FontSet, GlyphCache};
use crate::paint::{Canvas, Grid, Ink, PaintContext, Palette, paint_frame};
use crate::present::SoftwarePresenter;

use super::geometry::grid_for;
use super::options::WindowOptions;

pub struct WindowState<'a> {
    fonts: &'a FontSet,
    cache: GlyphCache,
    options: WindowOptions,
    points: u8,
    scale_factor: f64,
    metrics: CellMetrics,
    canvas: Canvas,
    grid: (u16, u16),
    window: Option<Arc<Window>>,
    presenter: Option<SoftwarePresenter>,
    proxy: Option<EventLoopProxy<()>>,
    full_screen: bool,
    clipboard: SystemClipboard,
    pub failure: Option<HostError>,
}

impl<'a> WindowState<'a> {
    pub fn new(fonts: &'a FontSet, options: WindowOptions) -> Self {
        let points = options.points;
        let metrics = CellMetrics::new(fonts, CellMetrics::pixel_size(points, 1.0), options.line_spacing);
        let canvas = Canvas::new(1, 1, options.palette.bg);
        let grid = (options.cols, options.rows);
        Self {
            fonts,
            cache: GlyphCache::new(),
            options,
            points,
            scale_factor: 1.0,
            metrics,
            canvas,
            grid,
            window: None,
            presenter: None,
            proxy: None,
            full_screen: false,
            clipboard: SystemClipboard::new(),
            failure: None,
        }
    }

    /// The cell size at scale factor 1, for sizing the window before it exists.
    pub fn logical_metrics(&self) -> CellMetrics {
        CellMetrics::new(
            self.fonts,
            CellMetrics::pixel_size(self.points, 1.0),
            self.options.line_spacing,
        )
    }

    pub fn options(&self) -> &WindowOptions {
        &self.options
    }

    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    pub fn grid(&self) -> (u16, u16) {
        self.grid
    }

    pub fn set_proxy(&mut self, proxy: EventLoopProxy<()>) {
        self.proxy = Some(proxy);
    }

    /// Takes the window and makes the surface that shows its pixels.
    pub fn attach(&mut self, window: Arc<Window>) -> Result<(), HostError> {
        self.presenter = Some(SoftwarePresenter::new(window.clone())?);
        self.scale_factor = window.scale_factor();
        self.window = Some(window);
        self.remeasure();
        Ok(())
    }

    /// The window changed size: a new canvas and, where it changed, a new grid.
    pub fn resized(&mut self, width: u32, height: u32) -> bool {
        self.canvas.resize(width.max(1), height.max(1), self.options.palette.bg);
        let grid = grid_for(&self.metrics, (width, height));
        let changed = grid != self.grid;
        self.grid = grid;
        changed
    }

    pub fn set_scale_factor(&mut self, factor: f64) -> bool {
        if (factor - self.scale_factor).abs() < f64::EPSILON {
            return false;
        }
        self.scale_factor = factor;
        self.remeasure()
    }

    pub fn points(&self) -> u8 {
        self.points
    }

    /// A new text size: the glyphs are all the wrong size now, so the cache
    /// goes and the grid is counted again.
    pub fn set_points(&mut self, points: u8) -> bool {
        if points == self.points {
            return false;
        }
        self.points = points;
        self.remeasure()
    }

    /// Measures the cell again and re-counts the grid for the window's size.
    fn remeasure(&mut self) -> bool {
        self.cache.clear();
        self.metrics = CellMetrics::new(
            self.fonts,
            CellMetrics::pixel_size(self.points, self.scale_factor),
            self.options.line_spacing,
        );
        let size = self.window.as_ref().map(|window| window.inner_size());
        match size {
            Some(size) => self.resized(size.width, size.height),
            None => false,
        }
    }

    /// The pixel size of one cell, for turning wheel pixels into rows.
    pub fn cell_size(&self) -> (u32, u32) {
        (self.metrics.width, self.metrics.height)
    }

    /// Where a pointer at these window pixels sits in the grid.
    pub fn cell_at(&self, x: f64, y: f64) -> (u16, u16, f32, f32) {
        let grid = self.grid_geometry();
        let width = f64::from(self.metrics.width.max(1));
        let height = f64::from(self.metrics.height.max(1));
        let across = (x - f64::from(grid.origin.0)) / width;
        let down = (y - f64::from(grid.origin.1)) / height;
        let col = across.floor().clamp(0.0, f64::from(self.grid.0.saturating_sub(1)));
        let row = down.floor().clamp(0.0, f64::from(self.grid.1.saturating_sub(1)));
        (
            col as u16,
            row as u16,
            (across - across.floor()) as f32,
            (down - down.floor()) as f32,
        )
    }

    /// The grid centered in the window, the leftover pixels as margin.
    fn grid_geometry(&self) -> Grid {
        Grid::centered(
            self.metrics,
            (self.canvas.width(), self.canvas.height()),
            self.grid.0,
            self.grid.1,
        )
    }

    /// The text on the system clipboard, for a paste.
    pub fn clipboard_text(&mut self) -> Option<String> {
        self.clipboard.text()
    }

    /// Borderless full screen, on and off.
    pub fn toggle_full_screen(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        self.full_screen = !self.full_screen;
        let mode = self.full_screen.then_some(Fullscreen::Borderless(None));
        window.set_fullscreen(mode);
    }
}

impl Presenter for WindowState<'_> {
    fn capabilities(&self) -> Capabilities {
        Capabilities::window()
    }

    fn present(&mut self, frame: &Frame) {
        let palette = self.options.palette;
        self.canvas.clear(palette.bg);
        let grid = self.grid_geometry();
        let mut context = PaintContext::new(self.fonts, &mut self.cache, grid, palette);
        paint_frame(&mut self.canvas, &mut context, frame);
        if let Some(presenter) = &mut self.presenter
            && let Err(error) = presenter.show(&self.canvas)
        {
            self.failure = Some(error);
        }
    }

    fn set_clipboard(&mut self, text: &str) {
        self.clipboard.set_text(text);
    }

    fn set_title(&mut self, title: &str) {
        if let Some(window) = &self.window {
            window.set_title(title);
        }
    }

    fn set_pointer_shape(&mut self, shape: PointerShape) {
        if let Some(window) = &self.window {
            window.set_cursor(cursor_icon(shape));
        }
    }

    fn set_text_size(&mut self, points: u8, line_spacing: f32) -> Option<(u16, u16)> {
        let before = self.grid;
        // The size the Options screen asked for is the one Ctrl+0 goes back to.
        self.options.points = points;
        self.options.line_spacing = line_spacing;
        self.points = points;
        self.remeasure();
        self.request_redraw();
        (self.grid != before).then_some(self.grid)
    }

    fn set_base_colors(&mut self, text: Color, background: Color) {
        let old = self.options.palette;
        self.options.palette = Palette::new(old.resolve(text, Ink::Foreground), old.resolve(background, Ink::Background));
        self.request_redraw();
    }

    fn request_redraw(&mut self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn waker(&self) -> Waker {
        let proxy = self.proxy.clone();
        Waker::new(move || {
            if let Some(proxy) = &proxy {
                let _ = proxy.send_event(());
            }
        })
    }

    fn run(&mut self, sink: &mut dyn surface::EventSink) -> Result<(), HostError> {
        super::event_loop::drive(self, sink)
    }
}

/// The native cursor for a pointer shape.
fn cursor_icon(shape: PointerShape) -> CursorIcon {
    match shape {
        PointerShape::Arrow => CursorIcon::Default,
        PointerShape::Hand => CursorIcon::Pointer,
        PointerShape::Text => CursorIcon::Text,
        PointerShape::ColumnResize => CursorIcon::ColResize,
        PointerShape::RowResize => CursorIcon::RowResize,
    }
}

#[cfg(test)]
include!("state.tests.rs");
