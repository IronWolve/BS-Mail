//! The screen in its own window.

use std::time::Instant;

use host::{Palette, WindowOptions, run_window};
use surface::Capabilities;
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, SizeChoice, window_grid};
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
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    let (cols, rows) = starting_grid(options.size, remembered.as_ref());
    let window = WindowOptions {
        title: display_name().to_string(),
        cols,
        rows,
        min_cols: MIN_COLS,
        min_rows: MIN_ROWS,
        points: scene.font.0,
        line_spacing: scene.font.1,
        palette: Palette::new(scene.base.0, scene.base.1),
    };
    if let Some(state) = &remembered {
        scene.view.set_prefs(state.layout.clone());
    }
    let effects = scene.view.render(cols, rows).effects.len();
    let mut screen = Screen::new(scene.view, scene.backend, (cols, rows), scene.fixed_now)
        .with_accounts(scene.live.clone())
        .with_settings(scene.settings.clone());
    if let Some(dir) = data_dir {
        screen = screen.remember_in(dir);
    }
    if options.smoke {
        screen = screen.stop_after(1);
    }
    run_window(&fonts, window, &mut screen).map_err(|error| AppError::Window(error.to_string()))?;
    screen.save_state();
    let (cols, rows) = screen.grid();
    Ok(Stats {
        cols,
        rows,
        messages: scene.messages,
        effects,
        source: scene.source,
        label: scene.label,
        accounts: scene.accounts,
        millis: started.elapsed().as_millis(),
        shown: Shown::Window {
            frames: screen.frames(),
            points: scene.font.0,
            font_bytes: fonts::embedded_bytes(),
        },
    })
}

/// The grid the window opens at: one asked for on the command line, else the
/// one it had when it last closed, else the reference grid.
fn starting_grid(size: SizeChoice, remembered: Option<&state_file::State>) -> (u16, u16) {
    match (size, remembered.and_then(|state| state.grid)) {
        (SizeChoice::Reference, Some(grid)) => grid,
        _ => window_grid(size),
    }
}

#[cfg(test)]
include!("window_run.tests.rs");
