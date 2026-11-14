//! Reading the user's settings: `config.toml` and a named theme, from the
//! data folder.
//!
//! A missing file is not a problem: the defaults are the program. A file
//! that cannot be read, a value out of range and a theme that cannot be found
//! each give the default for that one thing and a problem to show the user,
//! so a typo costs one setting and never the program.

use std::fs;
use std::path::Path;

use model::config::Config;
use model::config::font::{FONT_SIZE_MAX, FONT_SIZE_MIN, LINE_SPACING_MAX, LINE_SPACING_MIN};
use ui::theme::{DEFAULT_THEME_NAME, Theme, builtin_theme_text, default_theme, load_theme_file};

use crate::errors::AppError;

pub const CONFIG_NAME: &str = "config.toml";
pub const THEMES_DIR: &str = "themes";

/// What was read, and what was wrong with it.
pub struct Loaded {
    pub config: Config,
    pub theme: Theme,
    /// One line each, for the footer.
    pub problems: Vec<String>,
}

/// Reads the settings in `dir`, or the defaults when there is no folder.
pub fn load(dir: Option<&Path>) -> Result<Loaded, AppError> {
    let mut problems = Vec::new();
    let mut config = dir.map_or_else(Config::default, |dir| read_config(dir, &mut problems));
    clamp(&mut config, &mut problems);
    let theme = pick_theme(&config.appearance.theme, dir, &mut problems)?;
    Ok(Loaded { config, theme, problems })
}

fn read_config(dir: &Path, problems: &mut Vec<String>) -> Config {
    let path = dir.join(CONFIG_NAME);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Config::default(),
        Err(error) => {
            problems.push(format!("{CONFIG_NAME} could not be read: {error}"));
            return Config::default();
        }
    };
    match toml::from_str::<Config>(&text) {
        Ok(config) => config,
        Err(error) => {
            // The message says what; the span says where. The rest of the
            // error quotes the file, which does not belong in a footer.
            let where_ = error.span().map_or(String::new(), |span| format!(" (line {})", line_of(&text, span.start)));
            problems.push(format!("{CONFIG_NAME}{where_}: {}; using the defaults", error.message().trim()));
            Config::default()
        }
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// Brings every ranged value inside its range, saying so for each.
fn clamp(config: &mut Config, problems: &mut Vec<String>) {
    let size = config.font.size.clamp(FONT_SIZE_MIN, FONT_SIZE_MAX);
    if size != config.font.size {
        problems.push(format!("font.size {} is out of range, using {size}", config.font.size));
        config.font.size = size;
    }
    let spacing = config.font.line_spacing.clamp(LINE_SPACING_MIN, LINE_SPACING_MAX);
    let spacing = if spacing.is_nan() { 1.0 } else { spacing };
    if (spacing - config.font.line_spacing).abs() > f32::EPSILON {
        problems.push(format!("font.line_spacing {} is out of range, using {spacing}", config.font.line_spacing));
        config.font.line_spacing = spacing;
    }
    let weight = config.appearance.line_weight.clamp(1, 4);
    if weight != config.appearance.line_weight {
        problems.push(format!("appearance.line_weight {} is out of range, using {weight}", config.appearance.line_weight));
        config.appearance.line_weight = weight;
    }
}

/// The named theme: the built-in one, or `themes/<name>.toml` in the data
/// folder. Another theme it inherits from is found the same way.
fn pick_theme(name: &str, dir: Option<&Path>, problems: &mut Vec<String>) -> Result<Theme, AppError> {
    let default = || default_theme().map_err(|error| AppError::Theme(error.to_string()));
    if name == DEFAULT_THEME_NAME || name.is_empty() {
        return default();
    }
    let Some(text) = find_theme_text(name, dir) else {
        problems.push(format!("theme {name} was not found in {THEMES_DIR}/, using {DEFAULT_THEME_NAME}"));
        return default();
    };
    let found = |wanted: &str| find_theme_text(wanted, dir);
    match load_theme_file(&text, &found).and_then(|file| Theme::from_file(&file)) {
        Ok(theme) => Ok(theme),
        Err(error) => {
            let first = error.to_string().lines().next().unwrap_or_default().to_string();
            problems.push(format!("theme {name}: {first}; using {DEFAULT_THEME_NAME}"));
            default()
        }
    }
}

/// A theme's text by name. A name that could point outside the themes folder
/// is not a theme.
fn find_theme_text(name: &str, dir: Option<&Path>) -> Option<String> {
    if let Some(text) = builtin_theme_text(name) {
        return Some(text.to_string());
    }
    if !is_plain_name(name) {
        return None;
    }
    fs::read_to_string(dir?.join(THEMES_DIR).join(format!("{name}.toml"))).ok()
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

#[cfg(test)]
include!("config_io.tests.rs");
