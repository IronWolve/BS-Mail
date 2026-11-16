//! The screen inside the terminal it was started in.

use std::time::Instant;

use surface::{Capabilities, ColorDepth, Presenter};
use tty_host::{TtyHost, color_depth_from_env, terminal_size};
use ui::layout::{MIN_COLS, MIN_ROWS};

use crate::cli::{Options, REF_COLS, REF_ROWS};
use crate::errors::AppError;
use crate::link_opener::opener_for;
use crate::paths::data_dir;
use crate::screen::Screen;
use crate::stats::{Shown, Stats};
use crate::{scene, state_file};

/// Runs until the user quits.
pub fn run(options: &Options) -> Result<Stats, AppError> {
    let started = Instant::now();
    let depth = depth_for(options);
    let mut scene = scene::build(options, Capabilities::terminal(depth))?;
    let data_dir = data_dir(options.data_dir.as_ref());
    let remembered = data_dir.as_ref().map(|dir| state_file::load(dir));
    if let Some(state) = &remembered {
        scene.view.set_prefs(state.layout.clone());
        // Told before the first events are taken, so the first account and
        // folder shown are the ones that were open last.
        scene.view.restore_place(state.place.clone());
    }
    scene.prime();
    let (cols, rows) = terminal_size().unwrap_or((REF_COLS, REF_ROWS));
    let grid = (cols.max(MIN_COLS), rows.max(MIN_ROWS));
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
    let mut host = TtyHost::new(depth);
    host.run(&mut screen).map_err(|error| AppError::Window(error.to_string()))?;
    screen.save_addresses();
    screen.save_state()?;
    let (cols, rows) = screen.grid();
    Ok(Stats {
        cols,
        rows,
        messages: scene.messages,
        effects: 0,
        source: scene.source,
        label: scene.label,
        accounts: scene.accounts,
        millis: started.elapsed().as_millis(),
        shown: Shown::Tty {
            frames: screen.frames(),
            depth: depth_name(depth),
        },
    })
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
