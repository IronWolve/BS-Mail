//! Switching the theme from the Options screen.

use surface::{Color, Presenter};
use ui::AppView;
use ui::theme::Theme;

use super::settings::Settings;
use crate::config_edit::Setting;
use crate::config_io::theme_named;
use crate::scene::plain_colors;

impl Settings {
    /// Loads the theme `name` and shows it at once, then saves the choice. A
    /// theme that cannot be loaded changes nothing, saves nothing and says
    /// why; it is taken out of the list of themes until the settings are
    /// reloaded, so the choice goes on past it.
    pub(super) fn set_theme(&mut self, view: &mut AppView, host: &mut dyn Presenter, name: &str) {
        let theme = match theme_named(name, self.dir.as_deref()) {
            Ok(theme) => theme,
            Err(reason) => {
                self.names.retain(|listed| listed != name);
                view.set_theme_names(self.names.clone());
                return view.options_problem(&format!("{reason}; the theme in use was kept"));
            }
        };
        tell_the_host(host, &theme);
        view.set_theme(theme);
        self.save(view, &Setting::Theme(name.to_string()), true);
    }
}

/// Gives the host the theme's plain text and background.
pub(super) fn tell_the_host(host: &mut dyn Presenter, theme: &Theme) {
    let (text, background) = plain_colors(theme);
    host.set_base_colors(Color::Rgb(text.r, text.g, text.b), Color::Rgb(background.r, background.g, background.b));
}
