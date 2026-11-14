//! What a run did, for the block printed after it.

use std::path::PathBuf;

pub struct Stats {
    pub cols: u16,
    pub rows: u16,
    pub messages: usize,
    pub effects: usize,
    /// The account file, the accounts folder or the mock file the mail came
    /// from.
    pub source: PathBuf,
    /// What to call `source`.
    pub label: &'static str,
    /// How many accounts the run started with; `None` for mock mail.
    pub accounts: Option<usize>,
    pub millis: u128,
    pub shown: Shown,
}

/// Where the screen went, and what only that way has to report.
pub enum Shown {
    /// One draw into the terminal, then exit.
    Terminal,
    /// A run inside the terminal until the user quits.
    Tty {
        frames: u32,
        depth: &'static str,
    },
    Window {
        frames: u32,
        points: u8,
        font_bytes: usize,
    },
    Picture {
        path: PathBuf,
        width: u32,
        height: u32,
        bytes: u64,
    },
}
