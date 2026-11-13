//! The layout engine: preferred sizes, clamps, the fixed chrome rows, the
//! sidebar sections, the main split, list columns and `compute_layout`.
//! This file only declares and re-exports the modules.

pub mod chrome_rows;
pub mod clamp;
pub mod columns;
pub mod compute;
pub mod main_split;
pub mod options;
pub mod prefs;
pub mod regions;
pub mod sidebar_layout;
pub mod spread;

pub use chrome_rows::{ChromeRows, ChromeShape, RowSpan, chrome_rows};
pub use columns::{ColumnId, ColumnLayout, ColumnSlot, ColumnSpec, layout_columns};
pub use compute::compute_layout;
pub use main_split::{Divider, MainSplit, list_region, reader_region, split_main};
pub use options::LayoutOptions;
pub use prefs::LayoutPrefs;
pub use regions::{
    GridSize, Layout, LineKind, LineSpec, ListRegion, MIN_COLS, MIN_ROWS, Orientation,
    ReaderRegion, inset_h,
};
pub use sidebar_layout::{
    FolderSpacing, SectionKind, SidebarContent, SidebarInput, SidebarLayout, SidebarSection,
    layout_sidebar,
};
pub use spread::{Spread, bar_offset, spread_items};
