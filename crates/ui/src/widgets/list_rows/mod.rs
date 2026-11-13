//! The message list's rows: one envelope per row with a cell per column, and
//! a bar across the full width on the selected row. This file only declares
//! and re-exports the modules.

pub mod cells;
pub mod draw;
pub mod row_style;

pub use draw::{ListRowsView, draw_list_rows};
