//! The screen in its own window.

use std::path::PathBuf;
use std::time::Instant;

use host::{Palette, WindowOptions, run_window};
use model::config::WindowConfig;
use surface::Capabilities;
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, SizeChoice, window_grid};
use crate::errors::AppError;
use crate::link_opener::opener_for;
use crate::identity::{display_name, slug};
use crate::scene::Scene;
use crate::screen::Screen;
use crate::paths::data_dir;
use crate::stats::{Shown, Stats};
use crate::{fonts, scene, state_file};

/// Opens the window and runs until it is closed.
pub fn run(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let fonts = fonts::embedded().map_err(|error| AppError::Fonts(error.to_string()))?;
    let mut scene = scene::build(options, Capabilities::window())?;
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    // Told before the first events are taken, so the first account and folder
    // shown are the ones that were open last.
    if let Some(state) = &remembered {
        state.restore_into(&mut scene.view);
    }
    scene.prime();
    let remember_size = scene.window.remember_geometry;
    let (cols, rows) = starting_grid(options.size, remembered.as_ref(), remember_size);
    let window = window_options(&scene, options, (cols, rows));
    if let Some(state) = &remembered {
        scene.view.set_prefs(state.layout.clone());
    }
    let effects = scene.view.render(cols, rows).effects.len();
    let mut screen = Screen::new(scene.view, scene.backend, (cols, rows), scene.fixed_now)
        .with_accounts(scene.live.clone())
        .with_settings(scene.settings.clone())
        .with_link_opener(opener_for(data_dir.as_deref()));
    screen = keep_addresses(screen, data_dir.as_ref(), scene.live.is_some(), options.account.iter().cloned().collect());
    if let Some(dir) = data_dir {
        screen = remember(screen, dir, remember_size, remembered);
    }
    if options.smoke {
        screen = screen.stop_after(1);
    }
    run_window(&fonts, window, &mut screen).map_err(|error| AppError::Window(error.to_string()))?;
    screen.save_addresses();
    screen.save_state()?;
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

/// The program's icon, as the bytes of its picture: what the title bar and the
/// task bar show where the system takes an icon from the program.
const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icon/icon-256.png"));

/// What the window is told before it opens.
fn window_options(scene: &Scene, options: &Options, (cols, rows): (u16, u16)) -> WindowOptions {
    WindowOptions {
        title: display_name().to_string(),
        cols,
        rows,
        min_cols: MIN_COLS,
        min_rows: MIN_ROWS,
        maximized: opens_maximized(&scene.window, options.size),
        points: scene.font.0,
        line_spacing: scene.font.1,
        palette: Palette::new(scene.base.0, scene.base.1),
        icon_png: Some(ICON),
        app_id: slug().to_string(),
    }
}

/// The grid the window opens at: one asked for on the command line, else the
/// one it had when it last closed (when the settings remember it), else the
/// reference grid.
fn starting_grid(size: SizeChoice, remembered: Option<&state_file::State>, remember: bool) -> (u16, u16) {
    match (size, remembered.filter(|_| remember).and_then(|state| state.grid)) {
        (SizeChoice::Reference, Some(grid)) => grid,
        _ => window_grid(size),
    }
}

/// What the window leaves in the data folder: the bar sizes always, and its
/// own size when the settings remember it. With the size not remembered the
/// one already in the file is left as it is.
fn remember(screen: Screen, dir: PathBuf, size: bool, loaded: Option<state_file::State>) -> Screen {
    if size {
        screen.remember_in(dir)
    } else {
        screen.remember_layout_in(dir, loaded.unwrap_or_default())
    }
}

/// Keeps the address book, the pages of HTML messages for the browser and the
/// saved attachments in the data folder, for a run with real accounts; mock
/// mail has none. The account file named on the command line, which may be
/// anywhere, is never attached to a message.
fn keep_addresses(screen: Screen, dir: Option<&PathBuf>, real: bool, secrets: Vec<PathBuf>) -> Screen {
    match dir {
        Some(dir) if real => screen.keep_addresses_in(dir.clone()).keep_pages_in(dir).keep_attachments_in(dir, secrets),
        _ => screen,
    }
}

/// Whether the window opens maximized: the settings ask for it and no size
/// was asked for on the command line, which wins.
fn opens_maximized(window: &WindowConfig, size: SizeChoice) -> bool {
    window.start_maximized && matches!(size, SizeChoice::Reference)
}

#[cfg(test)]
include!("window_run.tests.rs");
