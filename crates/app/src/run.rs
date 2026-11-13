//! Draws one frame from the mock fixture and writes it to standard output.
//!
//! This is the screen without a window: the terminal host paints the cells,
//! so the layout, the colors and the text can be judged before the window
//! host exists. Effects, such as the double-size banner name, are not drawn.

use std::fmt;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

use engine::MockBackend;
use model::config::Config;
use surface::{Capabilities, ColorDepth};
use tty_host::{PaintOptions, paint_frame, terminal_size};
use ui::AppView;
use ui::icons::IconSet;
use ui::layout::{MIN_COLS, MIN_ROWS};
use ui::theme::default_theme;

use crate::cli::{Options, REF_COLS, REF_ROWS, SizeChoice};
use crate::identity::identity;
use crate::paths::{FIXTURE_NAME, installed_fixture};

/// Rows kept free below the screen when the terminal's own size is used, so
/// the shell prompt does not push the first row out of view.
const PROMPT_ROWS: u16 = 1;

/// What the run did, for the stats line.
pub struct Stats {
    pub cols: u16,
    pub rows: u16,
    pub messages: usize,
    pub effects: usize,
    pub fixture: PathBuf,
    pub millis: u128,
}

pub enum RunError {
    NoFixture,
    Fixture(PathBuf, String),
    Theme(String),
    Output(io::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFixture => write!(
                f,
                "no mock data: expected {FIXTURE_NAME} beside the program, or --mock <file>"
            ),
            Self::Fixture(path, reason) => {
                write!(f, "the mock file {} did not load: {reason}", path.display())
            }
            Self::Theme(reason) => write!(f, "the built-in theme did not load: {reason}"),
            Self::Output(error) => write!(f, "writing the screen failed: {error}"),
        }
    }
}

pub fn draw(options: &Options) -> Result<Stats, RunError> {
    let started = Instant::now();
    let fixture = match &options.mock {
        Some(path) => path.clone(),
        None => installed_fixture().ok_or(RunError::NoFixture)?,
    };
    let backend = MockBackend::from_file(&fixture)
        .map_err(|error| RunError::Fixture(fixture.clone(), error.to_string()))?;
    let theme = default_theme().map_err(|error| RunError::Theme(error.to_string()))?;
    let mut view = AppView::new(
        identity(),
        theme,
        Config::default(),
        IconSet::new(options.icons),
    );
    view.set_capabilities(Capabilities::terminal(depth(options.color)));
    view.set_now(backend.now());
    for event in backend.startup_events() {
        view.apply_event(event);
    }
    let (cols, rows) = grid_size(options.size);
    let frame = view.render(cols, rows);
    let text = paint_frame(&frame, &PaintOptions {
        color: options.color,
    });
    write_out(&text)?;
    Ok(Stats {
        cols,
        rows,
        messages: backend.fixture().messages.len(),
        effects: frame.effects.len(),
        fixture,
        millis: started.elapsed().as_millis(),
    })
}

fn depth(color: bool) -> ColorDepth {
    if color { ColorDepth::TrueColor } else { ColorDepth::None }
}

fn write_out(text: &str) -> Result<(), RunError> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes()).map_err(RunError::Output)?;
    out.flush().map_err(RunError::Output)
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
