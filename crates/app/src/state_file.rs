//! What the program remembers between runs: the sizes the user dragged the
//! bars to, the size of the window, and the account and folder that were open.
//!
//! One small file in the data folder. A file that is missing, unreadable or
//! not understood gives the defaults and is never an error: the worst result
//! of a damaged state file is bars and a window at their usual sizes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ui::layout::{LayoutPrefs, MIN_COLS, MIN_ROWS};
use ui::view_state::Place;

pub const STATE_NAME: &str = "state.toml";

/// Everything remembered.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub layout: LayoutPrefs,
    /// The window's grid, in cells, as it was when it last closed.
    pub grid: Option<(u16, u16)>,
    /// The account (by id) and the folder (by path) that were open.
    pub place: Place,
}

impl Default for State {
    fn default() -> Self {
        Self {
            layout: LayoutPrefs::default(),
            grid: None,
            place: Place::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct StateFile {
    #[serde(default)]
    layout: LayoutState,
    #[serde(default)]
    window: WindowState,
    #[serde(default)]
    view: ViewState,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ViewState {
    account: Option<String>,
    folder: Option<String>,
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
    // A name that is empty names nothing.
    let named = |text: Option<String>| text.filter(|text| !text.is_empty());
    state.place = Place { account: named(file.view.account), folder: named(file.view.folder) };
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
        view: ViewState {
            account: state.place.account.clone(),
            folder: state.place.folder.clone(),
        },
    };
    let text = toml::to_string_pretty(&file)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    crate::private_dirs::make(dir)?;
    let target = path_in(dir);
    let temporary = dir.join(format!("{STATE_NAME}.part"));
    // A leftover of an earlier try is removed (a link is removed itself, never
    // followed) and the new one is made fresh, so nothing a link points at is
    // ever written.
    if fs::symlink_metadata(&temporary).is_ok() {
        fs::remove_file(&temporary)?;
    }
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    io::Write::write_all(&mut file, text.as_bytes())?;
    // The bytes are on disk before the name points at them, so a power loss
    // after the rename cannot leave an empty or torn file in place of the
    // last good one.
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, &target)
}

#[cfg(test)]
include!("state_file.tests.rs");
