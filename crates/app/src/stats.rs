//! What a run did, for the block printed after it.

use std::path::PathBuf;

pub struct Stats {
    pub cols: u16,
    pub rows: u16,
    pub messages: usize,
    pub effects: usize,
    pub fixture: PathBuf,
    pub millis: u128,
    pub shown: Shown,
}

/// Where the screen went, and what only that way has to report.
pub enum Shown {
    Terminal,
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
