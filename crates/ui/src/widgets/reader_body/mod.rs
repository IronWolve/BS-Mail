//! The reading pane's body: styled lines wrapped to the pane, scrolled, with
//! a scrollbar when they overflow. This file only declares and re-exports the
//! modules.

mod cache;
pub mod draw;
pub mod marks;
pub mod text_map;
pub mod wrap;

pub use draw::{BodyView, body_link_at, draw_reader_body, visual_len};
pub use marks::{Highlights, Match, draw_highlights};
pub use text_map::{BodyPos, BodyText, body_pos_at, body_text};
