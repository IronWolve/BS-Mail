//! What every widget needs to draw: the theme, the icon set, the options and
//! the clock. Widgets ask it for styles by role and glyphs by icon id.

use model::MailDate;
use model::config::Config;
use surface::{Color, Style};

use crate::borders::BorderSet;
use crate::format::{CivilTime, format_pattern};
use crate::icons::{IconId, IconSet};
use crate::theme::{Theme, surface_color};

pub struct DrawContext<'a> {
    pub theme: &'a Theme,
    pub icons: &'a IconSet,
    pub config: &'a Config,
    pub borders: &'static BorderSet,
    pub now: MailDate,
}

impl<'a> DrawContext<'a> {
    pub fn new(theme: &'a Theme, icons: &'a IconSet, config: &'a Config, now: MailDate) -> Self {
        Self {
            theme,
            icons,
            config,
            borders: BorderSet::for_style(config.appearance.border),
            now,
        }
    }

    /// Foreground and weight of a role, for text over any background.
    pub fn fg(&self, role: &str) -> Style {
        self.theme.role(role).to_fg_style()
    }

    /// Foreground, background and weight, for a bar that brings its own color.
    pub fn full(&self, role: &str) -> Style {
        self.theme.role(role).to_style()
    }

    /// Background of a role alone.
    pub fn bg(&self, role: &str) -> Style {
        self.theme.role(role).to_bg_style()
    }

    /// A role's foreground as a surface color.
    pub fn fg_color(&self, role: &str) -> Color {
        surface_color(self.theme.role(role).fg)
    }

    /// A role's background as a surface color, when it has one.
    pub fn bg_color(&self, role: &str) -> Option<Color> {
        self.theme.role(role).bg.map(surface_color)
    }

    /// The text of an icon: always two cells wide.
    pub fn icon(&self, id: IconId) -> &str {
        self.icons.text(id)
    }

    /// The icon without the padding after a one-cell glyph, for inline use.
    pub fn icon_inline(&self, id: IconId) -> &str {
        self.icons.text(id).trim_end()
    }

    /// The time of day of UTC second `unix` in the viewer's zone, as
    /// `22:40:31`.
    pub fn clock(&self, unix: i64) -> String {
        let time = CivilTime::from_unix(unix, self.now.offset_minutes);
        format_pattern("%H:%M:%S", &time)
    }
}

#[cfg(test)]
include!("draw_context.tests.rs");
