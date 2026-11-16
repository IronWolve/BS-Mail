//! Turns a raw message into what the reading pane shows. This file only
//! declares and re-exports the modules.

mod attachment_sections;
mod attachments;
mod body_select;
mod flowed;
mod guarded;
mod html_page;
mod html_to_text;
mod limits;
mod prefer;
mod raw_text;
mod render;
mod reply_to;
mod styled_lines;

#[cfg(test)]
pub use render::render_body;
pub use prefer::{body_preference, set_body_preference};
pub use html_page::render_page;
pub use html_to_text::too_nested;
pub use raw_text::raw_text;
pub use render::render_message;
