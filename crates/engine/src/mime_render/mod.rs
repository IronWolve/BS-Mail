//! Turns a raw message into what the reading pane shows. This file only
//! declares and re-exports the modules.

mod attachments;
mod body_select;
mod flowed;
mod guarded;
mod html_to_text;
mod limits;
mod render;
mod reply_to;
mod styled_lines;

#[cfg(test)]
pub use render::render_body;
pub use render::render_message;
