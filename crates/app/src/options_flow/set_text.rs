//! The text size and row height chosen on the Options screen.

use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};
use surface::Presenter;
use ui::AppView;

use super::settings::Settings;
use crate::config_edit::Setting;

impl Settings {
    /// A new text size in points. Returns the grid the host now has, when the
    /// size changed it.
    pub(super) fn set_font_size(&mut self, view: &mut AppView, host: &mut dyn Presenter, points: u8) -> Option<(u16, u16)> {
        let points = points.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
        self.cli.points = None;
        self.text.0 = points;
        self.show_and_save(view, host, &Setting::FontSize(points))
    }

    /// A new row-height multiplier. Returns the grid the host now has, when
    /// it changed it.
    pub(super) fn set_line_spacing(&mut self, view: &mut AppView, host: &mut dyn Presenter, spacing: f32) -> Option<(u16, u16)> {
        let spacing = if spacing.is_nan() { 1.0 } else { spacing.clamp(LINE_SPACING_MIN, LINE_SPACING_MAX) };
        self.cli.line_spacing = None;
        self.text.1 = spacing;
        self.show_and_save(view, host, &Setting::LineSpacing(spacing))
    }

    /// Gives the host the size now in force and saves the choice. A host with
    /// a font of its own to change shows it at once; a terminal's font is the
    /// terminal's, so the setting waits for a window.
    fn show_and_save(&self, view: &mut AppView, host: &mut dyn Presenter, setting: &Setting) -> Option<(u16, u16)> {
        let grid = host.set_text_size(self.text.0, self.text.1);
        let live = host.capabilities().font_zoom;
        self.save(view, setting, live);
        grid
    }
}
