//! The screen in its own window.

use std::time::Instant;

use host::{Palette, WindowOptions, run_window};
use surface::Capabilities;
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, window_grid};
use crate::errors::AppError;
use crate::identity::display_name;
use crate::screen::Screen;
use crate::paths::data_dir;
use crate::stats::{Shown, Stats};
use crate::{fonts, scene, state_file};

/// Opens the window and runs until it is closed.
pub fn run(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let fonts = fonts::embedded().map_err(|error| AppError::Fonts(error.to_string()))?;
    let mut scene = scene::build(options, Capabilities::window())?;
    scene.prime();
    let (cols, rows) = window_grid(options.size);
    let window = WindowOptions {
        title: display_name().to_string(),
        cols,
        rows,
        min_cols: MIN_COLS,
        min_rows: MIN_ROWS,
        points: options.points,
        line_spacing: options.line_spacing,
        palette: Palette::new(scene.base.0, scene.base.1),
    };
    let data_dir = data_dir(options.data_dir.as_ref());
    if let Some(dir) = &data_dir {
        scene.view.set_prefs(state_file::load(dir));
    }
    let effects = scene.view.render(cols, rows).effects.len();
    let mut screen = Screen::new(scene.view, scene.backend, (cols, rows), scene.fixed_now);
    if let Some(dir) = data_dir {
        screen = screen.remember_in(dir);
    }
    if options.smoke {
        screen = screen.stop_after(1);
    }
    run_window(&fonts, window, &mut screen).map_err(|error| AppError::Window(error.to_string()))?;
    let (cols, rows) = screen.grid();
    Ok(Stats {
        cols,
        rows,
        messages: scene.messages,
        effects,
        source: scene.source,
        real: scene.real,
        millis: started.elapsed().as_millis(),
        shown: Shown::Window {
            frames: screen.frames(),
            points: options.points,
            font_bytes: fonts::embedded_bytes(),
        },
    })
}
