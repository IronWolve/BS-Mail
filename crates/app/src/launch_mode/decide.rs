//! Which way the screen is shown this run.
//!
//! The program is a terminal program. Unless a window is asked for with `--window`
//! (or a picture file with `--render-once`), the screen is drawn in the terminal
//! the program was started from, on every system.

use std::path::PathBuf;

use crate::cli::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Terminal,
    Window,
    /// Draw once into a picture file and exit, with no window at all.
    Picture(PathBuf),
}

/// The way to show the screen, for a mode.
pub fn decide(mode: &Mode) -> Launch {
    match mode {
        Mode::Auto | Mode::Terminal => Launch::Terminal,
        Mode::Window => Launch::Window,
        Mode::Picture(path) => Launch::Picture(path.clone()),
    }
}

#[cfg(test)]
include!("decide.tests.rs");
