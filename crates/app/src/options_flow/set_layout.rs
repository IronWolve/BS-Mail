//! The layout settings chosen on the Options screen: where the reader sits,
//! the spacing, the columns of the list and the panels.

use ui::AppView;
use ui::options::LayoutChange;

use super::settings::Settings;
use crate::config_edit::Setting;

impl Settings {
    /// The screen has already switched to `change`, and it applies at once;
    /// this saves it, the way the icon set is saved.
    pub(super) fn set_layout(&self, view: &mut AppView, change: LayoutChange) {
        self.save(view, &Setting::Layout(change), true);
    }
}
