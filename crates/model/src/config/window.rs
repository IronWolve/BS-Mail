//! The `[window]` table: geometry memory, start state and title (window mode only).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    /// Reopen at the size the window had when it closed. Position, maximized
    /// and full-screen state are not remembered yet.
    pub remember_geometry: bool,
    /// Open the window maximized, unless a size was asked for on the command
    /// line.
    pub start_maximized: bool,
    pub title_shows_folder: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            remember_geometry: true,
            start_maximized: false,
            title_shows_folder: true,
        }
    }
}
