//! What the program does when the Options screen asks for something to be
//! done: open the form to add or edit an account, remove one, or change or
//! reload a setting. The screen queues requests and answers nothing itself;
//! each is answered here through the view's own setters. This file only
//! declares and re-exports the modules.

mod answer;
mod edit;
mod reload;
mod remove;
mod save_note;
mod set_icons;
mod set_layout;
mod set_text;
mod set_theme;
mod settings;
mod settings_answer;

pub use answer::{Opened, answer};
pub use settings::{Settings, is_setting};
