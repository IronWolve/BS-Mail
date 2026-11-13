//! Was the program started from a console?
//!
//! A terminal on either output means the user is already looking at one, and
//! the program draws there rather than opening a window in front of them.
//! Both are asked, so sending the screen to a file from a terminal does not
//! turn into a window.

use std::io::IsTerminal;

pub fn from_console() -> bool {
    std::io::stdout().is_terminal() || std::io::stderr().is_terminal()
}
