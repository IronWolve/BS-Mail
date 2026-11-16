//! One file per config table; this file only declares and re-exports them.

pub mod appearance;
pub mod banner;
pub mod defaults;
pub mod font;
pub mod footer;
pub mod key_bar;
pub mod keys;
pub mod list;
pub mod notify;
pub mod reader;
pub mod sidebar;
pub mod strip;
pub mod window;

pub use appearance::{
    AppearanceConfig, BorderStyle, Density, GaugeStyle, IconMode, PaneBorders,
};
pub use banner::BannerConfig;
pub use defaults::Config;
pub use font::FontConfig;
pub use footer::FooterConfig;
pub use key_bar::KeyBarConfig;
pub use keys::{KeyValue, KeysConfig};
pub use list::{DateStyle, FromStyle, ListConfig, ListWidths, SizeStyle, SortSetting};
pub use notify::NotifyConfig;
pub use reader::{BodyPreference, ReaderConfig, ReaderPosition};
pub use sidebar::SidebarConfig;
pub use strip::StripConfig;
pub use window::WindowConfig;
