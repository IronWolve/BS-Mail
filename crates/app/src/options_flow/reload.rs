//! Reading the settings files again, from the Options screen.

use surface::Presenter;
use ui::AppView;
use ui::icons::IconSet;

use super::set_theme::tell_the_host;
use super::settings::Settings;
use crate::config_io;
use crate::scene::icon_kind;
use crate::theme_list::theme_names;

impl Settings {
    /// Reads `config.toml` and the theme again and takes them as a restart
    /// would, with the command line's flags still winning. Returns the grid
    /// the host now has, when the text size changed it.
    ///
    /// With no data folder there is nothing to read and nothing changes. A
    /// file that cannot be read or understood changes nothing either: the
    /// settings in force are kept and the problem is shown, in the footer
    /// too. Anything else wrong (a value out of range, a theme that is not
    /// there) is brought in as it is at start, and the first such problem
    /// is shown.
    pub(super) fn reload(&mut self, view: &mut AppView, host: &mut dyn Presenter) -> Option<(u16, u16)> {
        if self.dir.is_none() {
            // Nothing to read: taking the defaults would undo what the user chose.
            view.options_problem("There is no data folder to read settings from.");
            return None;
        }
        let loaded = match config_io::load(self.dir.as_deref()) {
            Ok(loaded) => loaded,
            Err(error) => {
                view.options_problem(&error.to_string());
                return None;
            }
        };
        if let Some(reason) = loaded.file_problem {
            let text = format!("{reason}; the settings in force were kept");
            view.set_config_problem(Some(text.clone()));
            self.owns_note = true;
            view.options_problem(&text);
            return None;
        }
        let kind = self.cli.icons.unwrap_or_else(|| icon_kind(loaded.config.appearance.icons));
        let points = self.cli.points.unwrap_or(loaded.config.font.size);
        let spacing = self.cli.line_spacing.unwrap_or(loaded.config.font.line_spacing);
        self.text = (points, spacing);
        tell_the_host(host, &loaded.theme);
        view.apply_settings(loaded.config, loaded.theme, IconSet::new(kind));
        self.names = theme_names(self.dir.as_deref());
        view.set_theme_names(self.names.clone());
        let grid = host.set_text_size(points, spacing);
        match loaded.problems.first() {
            Some(first) => {
                view.set_config_problem(Some(first.clone()));
                self.owns_note = true;
                view.options_problem(first);
            }
            None => {
                if self.owns_note {
                    view.set_config_problem(None);
                    self.owns_note = false;
                }
                view.options_info("Settings reloaded.");
            }
        }
        grid
    }
}
