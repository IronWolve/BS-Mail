//! Which way the screen is shown this run.
//!
//! Asked for plainly on the command line, the answer is that. Otherwise:
//! started from a console it stays in that console, and started on its own,
//! by a launcher or a double click, it opens a window.

use std::path::PathBuf;

use crate::cli::Mode;

use super::console_check::from_console;
use super::display_check::has_display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Terminal,
    Window,
    /// Draw once into a picture file and exit, with no window at all.
    Picture(PathBuf),
}

/// The way to show the screen, for a mode and what the run looks like.
pub fn decide(mode: &Mode) -> Launch {
    decide_for(mode, cfg!(windows))
}

/// The same, with the system given: Windows has no terminal mode, so a request
/// for one opens the window.
fn decide_for(mode: &Mode, windows: bool) -> Launch {
    match mode {
        Mode::Terminal if windows => Launch::Window,
        Mode::Terminal => Launch::Terminal,
        Mode::Window => Launch::Window,
        Mode::Picture(path) => Launch::Picture(path.clone()),
        Mode::Auto => auto(from_console(), has_display()),
    }
}

/// The rule, with both answers given, so it can be checked without a console
/// and without a display.
pub fn auto(console: bool, display: bool) -> Launch {
    if console || !display {
        Launch::Terminal
    } else {
        Launch::Window
    }
}

#[cfg(test)]
include!("decide.tests.rs");
