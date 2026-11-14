//! Draws one frame from the mock fixture into the terminal it was started
//! from, and writes it to standard output.
//!
//! This is the screen without a window: the terminal host paints the cells,
//! so the layout, the colors and the text read the same, minus the effects a
//! terminal cannot draw.

use std::io::{self, Write};
use std::time::Instant;

use surface::{Capabilities, ColorDepth};
use tty_host::{PaintOptions, paint_frame, terminal_size};
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, REF_COLS, REF_ROWS, SizeChoice};
use crate::errors::AppError;
use crate::scene::{self, SETTLE_LIMIT};
use crate::stats::{Shown, Stats};

/// Rows kept free below the screen when the terminal's own size is used, so
/// the shell prompt does not push the first row out of view.
const PROMPT_ROWS: u16 = 1;

pub fn draw(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let mut scene = scene::build(options, Capabilities::terminal(depth(options.color)))?;
    scene.settle(SETTLE_LIMIT);
    let (cols, rows) = grid_size(options.size);
    let frame = scene.view.render(cols, rows);
    let text = paint_frame(&frame, &PaintOptions {
        color: options.color,
    });
    write_out(&text)?;
    Ok(Stats {
        cols,
        rows,
        messages: scene.messages,
        effects: frame.effects.len(),
        source: scene.source,
        real: scene.real,
        setup: scene.setup.is_some(),
        millis: started.elapsed().as_millis(),
        shown: Shown::Terminal,
    })
}

fn depth(color: bool) -> ColorDepth {
    if color { ColorDepth::TrueColor } else { ColorDepth::None }
}

fn write_out(text: &str) -> Result<(), AppError> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes()).map_err(AppError::Output)?;
    out.flush().map_err(AppError::Output)
}

/// The grid to draw, never smaller than the screen's own minimum.
fn grid_size(choice: SizeChoice) -> (u16, u16) {
    let (cols, rows) = match choice {
        SizeChoice::Reference => (REF_COLS, REF_ROWS),
        SizeChoice::Fixed(cols, rows) => (cols, rows),
        SizeChoice::Fit => terminal_size()
            .map(|(cols, rows)| (cols, rows.saturating_sub(PROMPT_ROWS)))
            .unwrap_or((REF_COLS, REF_ROWS)),
    };
    (cols.max(MIN_COLS), rows.max(MIN_ROWS))
}

#[cfg(test)]
include!("run.tests.rs");
