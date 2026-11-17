//! The reading pane's body: styled lines wrapped to the pane, scrolled, with
//! a scrollbar when they overflow. This file only declares and re-exports the
//! modules.

mod cache;
pub mod draw;
pub mod light;
pub mod link_list;
pub mod marks;
pub mod text_map;
pub mod wrap;

pub use draw::{BodyView, LinkHit, body_link_at, body_link_hit, draw_reader_body, visual_len};
pub use light::{LinkLights, LinkState, draw_link_light};
pub use link_list::{LinkSpan, body_links};
pub use marks::{Highlights, Match, draw_highlights};
pub use text_map::{BodyPos, BodyText, body_pos_at, body_text};
