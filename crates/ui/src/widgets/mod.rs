//! One file per widget, plus the helpers they share. This file only declares
//! and re-exports the modules.

pub mod account_list;
pub mod account_strip;
pub mod banner;
pub mod cell_text;
pub mod chrome_lines;
pub mod draw_context;
pub mod folder_tree;
pub mod footer;
pub mod gauge;
pub mod key_bar;
pub mod section_title;
pub mod segment_row;
pub mod status_panel;

pub use account_list::draw_account_list;
pub use account_strip::{MailboxSummary, StripData, draw_account_strip};
pub use banner::draw_banner;
pub use cell_text::{Run, fill_cells, put_cut, put_runs, put_text};
pub use chrome_lines::draw_chrome_lines;
pub use draw_context::DrawContext;
pub use folder_tree::{
    FolderRowInfo, FolderTreeView, content_rows, draw_folder_tree, row_offsets, visible_rows,
};
pub use footer::{FooterData, draw_footer};
pub use gauge::draw_gauge;
pub use key_bar::draw_key_bar;
pub use section_title::draw_section_title;
pub use status_panel::{StatusLine, draw_status_panel};
