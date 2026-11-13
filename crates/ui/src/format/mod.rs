//! Text formatting for the list, the sidebar and the strip: dates, sizes,
//! counts and cell-width truncation. This file only declares and re-exports
//! them.

pub mod civil_time;
pub mod count_format;
pub mod date_format;
pub mod size_format;
pub mod time_pattern;
pub mod truncate;

pub use civil_time::CivilTime;
pub use count_format::{format_count, group_thousands, sidebar_count};
pub use date_format::{DateFormat, format_list_date, is_recent};
pub use size_format::{format_quota, format_size};
pub use time_pattern::format_pattern;
pub use truncate::{ELLIPSIS_CHAR, ELLIPSIS_DOTS, display_width, truncate};
