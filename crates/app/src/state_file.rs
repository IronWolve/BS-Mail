//! What the program remembers between runs: for now, the sizes the user
//! dragged the bars to.
//!
//! One small file in the data folder. A file that is missing, unreadable or
//! not understood gives the defaults and is never an error: the worst result
//! of a damaged state file is bars at their usual sizes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ui::layout::LayoutPrefs;

pub const STATE_NAME: &str = "state.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
struct StateFile {
    #[serde(default)]
    layout: LayoutState,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct LayoutState {
    sidebar_width: Option<u16>,
    list_share: Option<u16>,
}

pub fn path_in(dir: &Path) -> PathBuf {
    dir.join(STATE_NAME)
}

/// The remembered sizes laid over the defaults.
pub fn load(dir: &Path) -> LayoutPrefs {
    let mut prefs = LayoutPrefs::default();
    let Ok(text) = fs::read_to_string(path_in(dir)) else {
        return prefs;
    };
    let Ok(state) = toml::from_str::<StateFile>(&text) else {
        return prefs;
    };
    if let Some(width) = state.layout.sidebar_width {
        prefs.sidebar_width = width;
    }
    if let Some(share) = state.layout.list_share {
        prefs.list_share = share.min(1000);
    }
    prefs
}

/// Writes the sizes, through a temporary file and a rename so a crash never
/// leaves half a file. Makes the data folder when it is not there yet.
pub fn save(dir: &Path, prefs: &LayoutPrefs) -> io::Result<()> {
    let state = StateFile {
        layout: LayoutState {
            sidebar_width: Some(prefs.sidebar_width),
            list_share: Some(prefs.list_share),
        },
    };
    let text = toml::to_string_pretty(&state)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    fs::create_dir_all(dir)?;
    let target = path_in(dir);
    let temporary = dir.join(format!("{STATE_NAME}.part"));
    fs::write(&temporary, text)?;
    fs::rename(&temporary, &target)
}

#[cfg(test)]
include!("state_file.tests.rs");
