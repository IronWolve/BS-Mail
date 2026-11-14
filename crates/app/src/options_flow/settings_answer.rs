//! Each settings request of the Options screen and the one answer it gets.

use surface::Presenter;
use ui::{AppView, OptionsAction};

use super::settings::Settings;

impl Settings {
    /// Answers `actions` in order. Returns the grid the host now has when a
    /// text size changed it, so the screen draws at that size.
    pub fn answer(
        &mut self,
        view: &mut AppView,
        host: &mut dyn Presenter,
        actions: Vec<OptionsAction>,
    ) -> Option<(u16, u16)> {
        let mut grid = None;
        for action in actions {
            let changed = match action {
                OptionsAction::SetTheme(name) => {
                    self.set_theme(view, host, &name);
                    None
                }
                OptionsAction::SetIcons(mode) => {
                    self.set_icons(view, mode);
                    None
                }
                OptionsAction::SetFontSize(points) => self.set_font_size(view, host, points),
                OptionsAction::SetLineSpacing(spacing) => self.set_line_spacing(view, host, spacing),
                OptionsAction::ReloadSettings => self.reload(view, host),
                // The accounts' requests are answered next door.
                OptionsAction::AddAccount | OptionsAction::EditAccount(_) | OptionsAction::RemoveAccount(_) => None,
            };
            grid = changed.or(grid);
        }
        grid
    }
}

#[cfg(test)]
include!("settings_answer.tests.rs");
