//! One frame straight into a picture file.
//!
//! The same painting the window shows, drawn on a machine that may have no
//! screen at all: a build check, a golden picture, or a look at the screen
//! from the far end of a remote shell.

use std::path::{Path, PathBuf};
use std::time::Instant;

use host::{Canvas, CellMetrics, GlyphCache, Grid, PaintContext, Palette, paint_frame, write_png};
use surface::Capabilities;

use crate::cli::{Options, window_grid};
use crate::errors::AppError;
use crate::stats::{Shown, Stats};
use crate::{fonts, scene};

/// Draws the screen once and writes it to `path`.
pub fn render(options: &Options, path: &Path) -> Result<Stats, AppError> {
    let started = Instant::now();
    let fonts = fonts::embedded().map_err(|error| AppError::Fonts(error.to_string()))?;
    let mut scene = scene::build(options, Capabilities::window())?;
    scene.prime();
    let (cols, rows) = window_grid(options.size);
    let metrics = CellMetrics::new(
        &fonts,
        CellMetrics::pixel_size(options.points, 1.0),
        options.line_spacing,
    );
    let (width, height) = metrics.grid_pixels(cols, rows);
    let palette = Palette::new(scene.base.0, scene.base.1);
    let mut canvas = Canvas::new(width, height, palette.bg);
    let mut cache = GlyphCache::new();
    let mut context = PaintContext::new(&fonts, &mut cache, Grid::new(metrics, (0, 0)), palette);
    let frame = scene.view.render(cols, rows);
    paint_frame(&mut canvas, &mut context, &frame);
    write_png(&canvas, path).map_err(|error| AppError::Picture(error.to_string()))?;
    Ok(Stats {
        cols,
        rows,
        messages: scene.messages,
        effects: frame.effects.len(),
        fixture: scene.source,
        millis: started.elapsed().as_millis(),
        shown: Shown::Picture {
            path: PathBuf::from(path),
            width,
            height,
            bytes: file_bytes(path),
        },
    })
}

/// The size of the file just written, or zero when it cannot be read back.
fn file_bytes(path: &Path) -> u64 {
    std::fs::metadata(path).map(|data| data.len()).unwrap_or(0)
}
