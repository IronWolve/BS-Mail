//! Turns a raw message into what the reading pane shows. This file only
//! declares and re-exports the modules.

mod attachments;
mod body_select;
mod flowed;
mod html_to_text;
mod render;
mod reply_to;
mod styled_lines;

pub use render::render_body;
