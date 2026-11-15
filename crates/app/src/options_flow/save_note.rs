//! Saving a setting and saying, on the Options screen's line, what became
//! of it.

use ui::AppView;

use super::settings::Settings;
use crate::config_edit::{self, Setting};

impl Settings {
    /// Writes `setting` into `config.toml` and tells the user the outcome.
    /// `live` is whether this run already shows the change; when it does not,
    /// the line names a window start, since a terminal keeps its own font and
    /// rows however often it starts. A setting that could not be saved is
    /// still in force for this run, and the line says that it lasts until the
    /// program ends. The words of the problem come whole from `SaveProblem`.
    pub(super) fn save(&self, view: &mut AppView, setting: &Setting, live: bool) {
        match config_edit::save(self.dir.as_deref(), setting) {
            Ok(()) if live => view.options_info("Saved."),
            Ok(()) => view.options_info("Saved. Applies the next time the program starts in a window."),
            Err(problem) => {
                let lasts = if live { "; the change lasts until you quit" } else { "" };
                view.options_problem(&format!("{problem}{lasts}"));
            }
        }
    }
}
