//! What the program keeps to answer the Options screen's requests about
//! settings: where `config.toml` is, what the command line asked for, and the
//! text size the window is showing.

use std::path::{Path, PathBuf};

use ui::OptionsAction;
use ui::icons::IconKind;

use crate::cli::Options;
use crate::theme_list::theme_names;

/// The command-line flags that win over the file. Each one stays in force
/// until the Options screen changes the same setting, and is never written.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Overrides {
    pub(super) icons: Option<IconKind>,
    pub(super) points: Option<u8>,
    pub(super) line_spacing: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Settings {
    /// The data folder, where `config.toml` and `themes/` are; `None` when
    /// the program found none.
    pub(super) dir: Option<PathBuf>,
    pub(super) cli: Overrides,
    /// The text size in points and the row-height multiplier now in force.
    pub(super) text: (u8, f32),
    /// The themes the Theme choice goes through.
    pub(super) names: Vec<String>,
    /// Whether the note in the footer is about the settings, so a reload that
    /// finds nothing wrong may clear it, and one that finds something may
    /// replace it. A note about an account is left alone.
    pub(super) owns_note: bool,
}

impl Default for Settings {
    /// No data folder, nothing from the command line.
    fn default() -> Self {
        Self::start(&Options::default(), None, (12, 1.0), false)
    }
}

impl Settings {
    /// The settings of a run that started with `options`, showing text at
    /// `text` (points, row height), where `owns_note` says the footer's note
    /// is about the settings.
    pub fn start(options: &Options, dir: Option<&Path>, text: (u8, f32), owns_note: bool) -> Self {
        Self {
            dir: dir.map(Path::to_path_buf),
            cli: Overrides { icons: options.icons, points: options.points, line_spacing: options.line_spacing },
            text,
            names: theme_names(dir),
            owns_note,
        }
    }

    /// The themes the Theme choice goes through.
    pub fn theme_names(&self) -> &[String] {
        &self.names
    }
}

/// Whether the request is about settings rather than accounts.
pub fn is_setting(action: &OptionsAction) -> bool {
    matches!(
        action,
        OptionsAction::SetTheme(_)
            | OptionsAction::SetIcons(_)
            | OptionsAction::SetFontSize(_)
            | OptionsAction::SetLineSpacing(_)
            | OptionsAction::SetLayout(_)
            | OptionsAction::ReloadSettings
    )
}
