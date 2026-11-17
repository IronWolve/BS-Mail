//! The screen in its own window.

use std::path::PathBuf;
use std::time::Instant;

use host::{Palette, WindowOptions, run_window};
use model::config::WindowConfig;
use surface::Capabilities;
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, SizeChoice, window_grid};
use crate::data_lock::DataLock;
use crate::errors::AppError;
use crate::link_opener::opener_for;
use crate::identity::{display_name, slug};
use crate::scene::Look;
use crate::screen::Screen;
use crate::paths::data_dir;
use crate::stats::{Shown, Stats};
use crate::unlock::{Built, Facts, Gate, Gated, LockOnExit, vault_folder};
use crate::{fonts, scene, state_file};

/// Opens the window and runs until it is closed. When the vault is protected
/// by a passphrase the prompt comes first, in the same window, and nothing else
/// starts until it is answered.
pub fn run(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let fonts = fonts::embedded().map_err(|error| AppError::Fonts(error.to_string()))?;
    // What the window needs to open comes from the settings, not from the accounts.
    let look = scene::look(options)?;
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    let remember_size = look.window.remember_geometry;
    let grid = starting_grid(options.size, remembered.as_ref(), remember_size);
    let window = window_options(&look, options, grid);
    // Wipes the passphrase key held in memory when the run ends, however it ends.
    let _wipe = LockOnExit::new(vault_folder(options));
    let gate = Gate::for_run(options)?;
    let asked = options.clone();
    let mut gated = Gated::new(gate, grid, Box::new(move |held, grid| build_screen(&asked, held, grid, remember_size)))?;
    run_window(&fonts, window, &mut gated).map_err(|error| AppError::Window(error.to_string()))?;
    let Some(Built { mut screen, facts, lock: _held }) = gated.finish()? else {
        return Ok(quit_stats(options, grid, look.font.0, started));
    };
    screen.save_addresses();
    screen.save_state()?;
    let (cols, rows) = screen.grid();
    Ok(Stats {
        cols,
        rows,
        messages: facts.messages,
        effects: facts.effects,
        source: facts.source,
        label: facts.label,
        accounts: facts.accounts,
        millis: started.elapsed().as_millis(),
        shown: Shown::Window {
            frames: screen.frames(),
            points: look.font.0,
            font_bytes: fonts::embedded_bytes(),
        },
    })
}

/// What the numbers say for a run that ended at the passphrase prompt.
fn quit_stats(options: &Options, (cols, rows): (u16, u16), points: u8, started: Instant) -> Stats {
    Stats {
        cols,
        rows,
        messages: 0,
        effects: 0,
        source: vault_folder(options).unwrap_or_default(),
        label: "passphrase not given for",
        accounts: None,
        millis: started.elapsed().as_millis(),
        shown: Shown::Window { frames: 0, points, font_bytes: fonts::embedded_bytes() },
    }
}

/// The screen with the mail on it, built once the vault is open (at once when
/// it needs no passphrase), for a window of `grid` cells.
pub(crate) fn build_screen(options: &Options, held: Option<DataLock>, (cols, rows): (u16, u16), remember_size: bool) -> Result<Built, AppError> {
    let mut scene = scene::build_holding(options, Capabilities::window(), held)?;
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    // Told before the first events are taken, so the first account and folder
    // shown are the ones that were open last.
    if let Some(state) = &remembered {
        state.restore_into(&mut scene.view);
    }
    scene.prime();
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
    let facts = Facts { messages: scene.messages, source: scene.source, label: scene.label, accounts: scene.accounts, effects };
    Ok(Built { screen, facts, lock: scene.lock })
}

/// The program's icon, as the bytes of its picture: what the title bar and the
/// task bar show where the system takes an icon from the program.
const ICON: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icon/icon-256.png"));

/// What the window is told before it opens.
fn window_options(look: &Look, options: &Options, (cols, rows): (u16, u16)) -> WindowOptions {
    WindowOptions {
        title: display_name().to_string(),
        cols,
        rows,
        min_cols: MIN_COLS,
        min_rows: MIN_ROWS,
        maximized: opens_maximized(&look.window, options.size),
        points: look.font.0,
        line_spacing: look.font.1,
        palette: Palette::new(look.base.0, look.base.1),
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
