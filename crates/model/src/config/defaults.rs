//! The complete default configuration as one `Config`, plus the helper the
//! table defaults use to build string lists.

use serde::{Deserialize, Serialize};

use super::appearance::AppearanceConfig;
use super::banner::BannerConfig;
use super::font::FontConfig;
use super::footer::FooterConfig;
use super::key_bar::KeyBarConfig;
use super::keys::KeysConfig;
use super::list::ListConfig;
use super::notify::NotifyConfig;
use super::reader::ReaderConfig;
use super::sidebar::SidebarConfig;
use super::strip::StripConfig;
use super::window::WindowConfig;

/// Every table this stage reads. Missing tables and missing keys take their
/// defaults; tables not listed here are ignored until a later stage adds them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub font: FontConfig,
    pub window: WindowConfig,
    pub banner: BannerConfig,
    pub strip: StripConfig,
    pub sidebar: SidebarConfig,
    pub list: ListConfig,
    pub reader: ReaderConfig,
    #[serde(rename = "keybar")]
    pub key_bar: KeyBarConfig,
    pub footer: FooterConfig,
    pub notify: NotifyConfig,
    /// What the user rebound; empty when nothing is (so it is not printed).
    #[serde(skip_serializing_if = "KeysConfig::is_empty")]
    pub keys: KeysConfig,
}

/// Builds an owned list of strings from literals.
pub(crate) fn string_list(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}
