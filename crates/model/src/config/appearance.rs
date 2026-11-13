//! The `[appearance]` table: theme, icon set, borders, density and counts.

use serde::{Deserialize, Serialize};

/// Which icon set to use; `auto` picks one from the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconMode {
    #[default]
    Auto,
    Nerd,
    Unicode,
    Ascii,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    #[default]
    Rounded,
    Plain,
    Thick,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaneBorders {
    #[default]
    Shared,
    Boxed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    #[default]
    Auto,
    Roomy,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GaugeStyle {
    #[default]
    Outlined,
    Bar,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme: String,
    pub icons: IconMode,
    /// Frame line thickness in pixels at scale 1.
    pub line_weight: u8,
    /// Rounded corner radius, in cell widths.
    pub corner_radius: f32,
    pub border: BorderStyle,
    pub pane_borders: PaneBorders,
    pub density: Density,
    pub gauge: GaugeStyle,
    /// Rounded fill behind keycaps.
    pub keycap_round: bool,
    /// Sidebar unread count; `{n}` is the number, empty hides counts.
    pub count_format: String,
    pub hide_zero_counts: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "neon-night".to_string(),
            icons: IconMode::Auto,
            line_weight: 1,
            corner_radius: 0.5,
            border: BorderStyle::Rounded,
            pane_borders: PaneBorders::Shared,
            density: Density::Auto,
            gauge: GaugeStyle::Outlined,
            keycap_round: true,
            count_format: "[{n}]".to_string(),
            hide_zero_counts: false,
        }
    }
}
