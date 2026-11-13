//! One file per widget, plus the helpers they share. This file only declares
//! and re-exports the modules.

pub mod account_list;
pub mod account_strip;
pub mod banner;
pub mod body_highlight;
pub mod cell_text;
pub mod chrome_lines;
pub mod draw_context;
pub mod folder_tree;
pub mod footer;
pub mod gauge;
pub mod key_bar;
pub mod list_header;
pub mod list_rows;
pub mod list_title;
pub mod reader_body;
pub mod reader_headers;
pub mod reader_title;
pub mod scrollbar;
pub mod section_title;
pub mod segment_row;
pub mod status_panel;
pub mod subject_style;

pub use account_list::draw_account_list;
pub use account_strip::{MailboxSummary, StripData, draw_account_strip};
pub use banner::draw_banner;
pub use body_highlight::{HighlightRules, highlight_line, highlight_text};
pub use cell_text::{
    Align, Run, cut_runs, fill_cells, put_aligned, put_cut, put_runs, put_runs_cut, put_text,
};
pub use chrome_lines::draw_chrome_lines;
pub use draw_context::DrawContext;
pub use folder_tree::{
    FolderRowInfo, FolderTreeView, content_rows, draw_folder_tree, row_offsets, visible_rows,
};
pub use footer::{FooterData, draw_footer};
pub use gauge::draw_gauge;
pub use key_bar::draw_key_bar;
pub use list_header::{column_align, column_label, draw_list_header};
pub use list_rows::{ListRowsView, draw_list_rows};
pub use list_title::draw_list_title;
pub use reader_body::{BodyView, draw_reader_body, visual_len};
pub use reader_headers::draw_reader_headers;
pub use reader_title::{ReaderTitleData, draw_reader_title};
pub use scrollbar::draw_scrollbar;
pub use section_title::draw_section_title;
pub use status_panel::{StatusLine, draw_status_panel};
pub use subject_style::subject_runs;
