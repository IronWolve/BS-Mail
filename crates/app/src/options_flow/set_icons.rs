//! The icon set chosen on the Options screen.

use model::config::IconMode;
use ui::AppView;

use super::settings::Settings;
use crate::config_edit::Setting;

impl Settings {
    /// The screen has already switched to `mode`. This saves it, and drops
    /// the command line's `--icons`, which the user has just overruled.
    pub(super) fn set_icons(&mut self, view: &mut AppView, mode: IconMode) {
        self.cli.icons = None;
        self.save(view, &Setting::Icons(mode), true);
    }
}
