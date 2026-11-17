//! The screen inside the terminal it was started in.

use std::time::Instant;

use surface::{Capabilities, ColorDepth, Presenter};
use tty_host::{TtyHost, color_depth_from_env, terminal_size};
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, REF_COLS, REF_ROWS};
use crate::data_lock::DataLock;
use crate::errors::AppError;
use crate::link_opener::opener_for;
use crate::paths::data_dir;
use crate::screen::Screen;
use crate::stats::{Shown, Stats};
use crate::unlock::{Built, Facts, Gate, Gated, LockOnExit, vault_folder};
use crate::{scene, state_file};

/// Runs until the user quits. When the vault is protected by a passphrase the
/// prompt comes first, in the same terminal screen, and nothing else starts
/// until it is answered.
pub fn run(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let depth = depth_for(options);
    // Wipes the passphrase key held in memory when the run ends, however it ends.
    let _wipe = LockOnExit::new(vault_folder(options));
    let gate = Gate::for_run(options)?;
    let (cols, rows) = terminal_size().unwrap_or((REF_COLS, REF_ROWS));
    let grid = (cols.max(MIN_COLS), rows.max(MIN_ROWS));
    let asked = options.clone();
    let mut gated = Gated::new(gate, grid, Box::new(move |held, grid| build_screen(&asked, depth, held, grid)))?;
    let mut host = TtyHost::new(depth);
    host.run(&mut gated).map_err(|error| AppError::Window(error.to_string()))?;
    let Some(Built { mut screen, facts, lock: _held }) = gated.finish()? else {
        return Ok(quit_stats(options, depth, grid, started));
    };
    screen.save_addresses();
    screen.save_state()?;
    let (cols, rows) = screen.grid();
    Ok(Stats {
        cols,
        rows,
        messages: facts.messages,
        effects: 0,
        source: facts.source,
        label: facts.label,
        accounts: facts.accounts,
        millis: started.elapsed().as_millis(),
        shown: Shown::Tty {
            frames: screen.frames(),
            depth: depth_name(depth),
        },
    })
}

/// What the numbers say for a run that ended at the passphrase prompt.
fn quit_stats(options: &Options, depth: ColorDepth, (cols, rows): (u16, u16), started: Instant) -> Stats {
    Stats {
        cols,
        rows,
        messages: 0,
        effects: 0,
        source: vault_folder(options).unwrap_or_default(),
        label: "passphrase not given for",
        accounts: None,
        millis: started.elapsed().as_millis(),
        shown: Shown::Tty { frames: 0, depth: depth_name(depth) },
    }
}

/// The screen with the mail on it, built once the vault is open (at once when
/// it needs no passphrase).
pub(crate) fn build_screen(options: &Options, depth: ColorDepth, held: Option<DataLock>, grid: (u16, u16)) -> Result<Built, AppError> {
    let mut scene = scene::build_holding(options, Capabilities::terminal(depth), held)?;
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    if let Some(state) = &remembered {
        scene.view.set_prefs(state.layout.clone());
        // Told before the first events are taken, so the first account and
        // folder shown are the ones that were open last.
        state.restore_into(&mut scene.view);
    }
    scene.prime();
    let mut screen = Screen::new(scene.view, scene.backend, grid, scene.fixed_now)
        .with_accounts(scene.live.clone())
        .with_settings(scene.settings.clone())
        .with_link_opener(opener_for(data_dir.as_deref()));
    if let (Some(dir), true) = (&data_dir, scene.live.is_some()) {
        let secrets = options.account.iter().cloned().collect();
        screen = screen.keep_addresses_in(dir.clone()).keep_pages_in(dir).keep_attachments_in(dir, secrets);
    }
    if let (Some(dir), Some(state)) = (data_dir, remembered) {
        screen = screen.remember_layout_in(dir, state);
    }
    let facts = Facts { messages: scene.messages, source: scene.source, label: scene.label, accounts: scene.accounts, effects: 0 };
    Ok(Built { screen, facts, lock: scene.lock })
}

/// The color depth: named, else none when colors are off, else the terminal's.
fn depth_for(options: &Options) -> ColorDepth {
    if !options.color {
        return ColorDepth::None;
    }
    options.color_depth.unwrap_or_else(color_depth_from_env)
}

fn depth_name(depth: ColorDepth) -> &'static str {
    match depth {
        ColorDepth::TrueColor => "true color",
        ColorDepth::Indexed256 => "256 colors",
        ColorDepth::Ansi16 => "16 colors",
        ColorDepth::None => "no color",
    }
}

#[cfg(test)]
include!("tty_run.tests.rs");
