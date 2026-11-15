//! What the layout reads from the configuration and from the data: which
//! regions are shown, the section order, and how tall the content is.

use model::config::{Config, Density, ReaderPosition};
use surface::EffectSupport;

use super::columns::ColumnSpec;
use super::sidebar_layout::{SectionKind, SidebarContent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutOptions {
    /// Rows of the banner; 0 hides it.
    pub banner_rows: u16,
    pub strip: bool,
    pub key_bar: bool,
    pub footer: bool,
    pub sidebar: bool,
    pub sections: Vec<SectionKind>,
    pub density: Density,
    pub content: SidebarContent,
    pub reader: ReaderPosition,
    /// Header lines in the reading pane.
    pub reader_header_rows: u16,
    /// The message is being read: where the screen has no room for the
    /// reading pane beside the list (or it is hidden), the message fills the
    /// main area in place of the list.
    pub reader_full: bool,
    pub columns: ColumnSpec,
}

impl LayoutOptions {
    /// Reads the region switches, orders and counts from `config`.
    pub fn from_config(config: &Config, content: SidebarContent) -> Self {
        let banner = &config.banner;
        Self {
            banner_rows: if banner.show {
                u16::from(banner.title_scale).clamp(1, 2)
            } else {
                0
            },
            strip: config.strip.show,
            key_bar: config.key_bar.show,
            footer: config.footer.show,
            sidebar: config.sidebar.show,
            sections: SectionKind::list_from_names(&config.sidebar.sections),
            density: config.appearance.density,
            content,
            reader: config.reader.position,
            reader_header_rows: u16::try_from(config.reader.headers.len()).unwrap_or(0),
            reader_full: false,
            columns: ColumnSpec::from_names(&config.list.columns, &config.list.flex_column),
        }
    }

    /// Leaves out what the host cannot draw. Without scaled text the banner
    /// is one row: its second row carries nothing of its own, it is only the
    /// lower half of the double-size title.
    pub fn fit_to_host(mut self, effects: &EffectSupport) -> Self {
        if !effects.scaled_text {
            self.banner_rows = self.banner_rows.min(1);
        }
        self
    }
}

#[cfg(test)]
include!("options.tests.rs");
