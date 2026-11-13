//! The `[banner]` table: the title banner region.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BannerConfig {
    pub show: bool,
    pub slashes: bool,
    pub slash_count: u8,
    /// 1 is the one-row banner, 2 the double-size title.
    pub title_scale: u8,
    pub gradient: bool,
    pub glow: bool,
    /// Empty means the manifest tagline.
    pub tagline: String,
    pub show_version: bool,
}

impl Default for BannerConfig {
    fn default() -> Self {
        Self {
            show: true,
            slashes: true,
            slash_count: 6,
            title_scale: 2,
            gradient: true,
            glow: true,
            tagline: String::new(),
            show_version: true,
        }
    }
}
