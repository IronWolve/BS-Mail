//! Remembering what the screen shows between runs: the sizes of the bars and
//! the window, and the account and folder that are open, saved in the data
//! folder when they change.

use std::path::PathBuf;

use super::Screen;
use crate::errors::AppError;
use crate::state_file::{self, State};

impl Screen {
    /// Remembers the sizes of the bars and the window in `dir`.
    pub fn remember_in(mut self, dir: PathBuf) -> Self {
        let current = self.state_now();
        self.remember = Some((dir, current));
        self
    }

    /// Remembers only the bar sizes, starting from what was loaded: for a
    /// terminal, whose grid is not the window's.
    pub fn remember_layout_in(mut self, dir: PathBuf, loaded: State) -> Self {
        self.track_grid = false;
        self.remember = Some((dir, loaded));
        self
    }

    /// What would be remembered now.
    fn state_now(&self) -> State {
        let grid = if self.track_grid {
            Some((self.cols, self.rows))
        } else {
            self.remember.as_ref().and_then(|(_, saved)| saved.grid)
        };
        let saved_place = self.remember.as_ref().map(|(_, saved)| saved.place.clone()).unwrap_or_default();
        State {
            layout: self.view.prefs().clone(),
            grid,
            place: self.view.place().over(&saved_place),
        }
    }

    /// Keeps the account and folder that are open when they are not the ones
    /// kept, so a run that ends without a goodbye still comes back to them.
    pub(super) fn save_place_when_changed(&mut self) {
        let changed = self.remember.as_ref().is_some_and(|(_, saved)| self.state_now().place != saved.place);
        if changed {
            // A failure waits: closing the program tries again and says so.
            let _ = self.save_state();
        }
    }

    /// Writes what is remembered, when it has changed. Called when a drag
    /// ends, when the account or folder open changes and when the window
    /// closes. A write that failed is an error, and
    /// what was saved before is left as it was; the sizes stay marked as not
    /// saved, so the next call tries again.
    pub fn save_state(&mut self) -> Result<(), AppError> {
        let current = self.state_now();
        let Some((dir, saved)) = &mut self.remember else {
            return Ok(());
        };
        if current == *saved {
            return Ok(());
        }
        state_file::save(dir, &current).map_err(|error| AppError::State(dir.clone(), error))?;
        *saved = current;
        Ok(())
    }
}
