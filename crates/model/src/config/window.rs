//! The `[window]` table: geometry memory, start state and title (window mode only).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    /// Remember size, position, maximized and full-screen state.
    pub remember_geometry: bool,
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
