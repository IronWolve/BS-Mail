//! What the program remembers between runs: the sizes the user dragged the
//! bars to and the size of the window.
//!
//! One small file in the data folder. A file that is missing, unreadable or
//! not understood gives the defaults and is never an error: the worst result
//! of a damaged state file is bars and a window at their usual sizes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ui::layout::{LayoutPrefs, MIN_COLS, MIN_ROWS};

pub const STATE_NAME: &str = "state.toml";

/// Everything remembered.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub layout: LayoutPrefs,
    /// The window's grid, in cells, as it was when it last closed.
    pub grid: Option<(u16, u16)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            layout: LayoutPrefs::default(),
            grid: None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StateFile {
    #[serde(default)]
    layout: LayoutState,
    #[serde(default)]
    window: WindowState,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LayoutState {
    sidebar_width: Option<u16>,
    list_share: Option<u16>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct WindowState {
    cols: Option<u16>,
    rows: Option<u16>,
}

pub fn path_in(dir: &Path) -> PathBuf {
    dir.join(STATE_NAME)
}

/// The remembered state laid over the defaults.
pub fn load(dir: &Path) -> State {
    let mut state = State::default();
    let Ok(text) = fs::read_to_string(path_in(dir)) else {
        return state;
    };
    let Ok(file) = toml::from_str::<StateFile>(&text) else {
        return state;
    };
    if let Some(width) = file.layout.sidebar_width {
        state.layout.sidebar_width = width;
    }
    if let Some(share) = file.layout.list_share {
        state.layout.list_share = share.min(1000);
    }
    if let (Some(cols), Some(rows)) = (file.window.cols, file.window.rows) {
        // A grid under the screen's own minimum would open a window that
        // cannot show anything, so it is not believed.
        if cols >= MIN_COLS && rows >= MIN_ROWS {
            state.grid = Some((cols, rows));
        }
    }
    state
}

/// Writes the state, through a temporary file and a rename so a crash never
/// leaves half a file. Makes the data folder when it is not there yet.
pub fn save(dir: &Path, state: &State) -> io::Result<()> {
    let file = StateFile {
        layout: LayoutState {
            sidebar_width: Some(state.layout.sidebar_width),
            list_share: Some(state.layout.list_share),
        },
        window: WindowState {
            cols: state.grid.map(|grid| grid.0),
            rows: state.grid.map(|grid| grid.1),
        },
    };
    let text = toml::to_string_pretty(&file)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    crate::private_dirs::make(dir)?;
    let target = path_in(dir);
    let temporary = dir.join(format!("{STATE_NAME}.part"));
    fs::write(&temporary, text)?;
    fs::rename(&temporary, &target)
}

#[cfg(test)]
include!("state_file.tests.rs");
