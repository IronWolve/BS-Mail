//! The message list's rows: one envelope per row with a cell per column, and
//! a bar across the full width on the selected row. This file only declares
//! and re-exports the modules.

pub mod cells;
pub mod draw;
mod flag_cell;
pub mod party;
pub mod row_style;
pub mod thread_cell;

pub use draw::{ListRowsView, draw_list_rows};
pub use party::column_label_for;
