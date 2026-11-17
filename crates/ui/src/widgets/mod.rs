//! One file per widget, plus the helpers they share. This file only declares
//! and re-exports the modules.

pub mod account_list;
pub mod account_strip;
pub mod banner;
pub mod body_highlight;
pub mod cell_text;
pub mod chrome_lines;
pub mod context_menu;
pub mod draw_context;
pub mod find_bar;
pub mod focus_bar;
pub mod focus_mark;
pub mod folder_tree;
pub mod footer;
pub mod gauge;
pub mod help_overlay;
pub mod help_rows;
pub mod key_bar;
pub mod link_hint;
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
pub use context_menu::{Menu, MenuEntry, draw_menu};
pub use draw_context::DrawContext;
pub use focus_bar::bar_role;
pub use focus_mark::{Heading, mark_focused};
pub use folder_tree::{
    FolderRowInfo, FolderTreeView, content_rows, draw_folder_tree, row_offsets, visible_rows,
};
pub use footer::{FooterData, FooterItem, FooterSpan, draw_footer, draw_footer_with_hint, footer_item_at};
pub use gauge::draw_gauge;
pub use help_overlay::{draw_help, help_capacity, help_max_scroll};
pub use key_bar::{button_at, button_span_at, draw_key_bar};
pub use link_hint::{LinkHint, is_web_address};
pub use list_header::{column_align, column_label, draw_list_header};
pub use list_rows::{ListRowsView, column_label_for, draw_list_rows};
pub use list_title::{draw_list_title, heading_extent as list_heading_extent};
pub use reader_body::{
    BodyPos, BodyText, BodyView, Highlights, LinkHit, LinkLights, LinkSpan, LinkState, Match, body_link_at, body_link_hit, body_links,
    body_pos_at, body_text, draw_highlights, draw_link_light, draw_reader_body, visual_len,
};
pub use reader_headers::draw_reader_headers;
pub use reader_title::{ReaderTitleData, draw_reader_title, heading_extent as reader_heading_extent};
pub use scrollbar::{draw_scrollbar, first_for_top, thumb_span};
pub use section_title::draw_section_title;
pub use status_panel::{StatusLine, draw_status_panel};
pub use subject_style::subject_runs;
