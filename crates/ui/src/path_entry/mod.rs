//! The box that asks for the path of a file, with the folders and files that
//! match what is typed listed under it. It is drawn in the cell grid like every
//! other box, so it works the same in the window and in the terminal, with the
//! keyboard and with the mouse; there is no native file dialog. It is used to
//! attach a file to the message being written and to say where to save an
//! attachment. This file only declares and re-exports the modules.

pub mod complete;
pub mod draw;
pub mod input;
pub mod prompt;

pub use complete::{Match, home_dir, matches};
pub use draw::draw_path_prompt;
pub use input::PathOutcome;
pub use prompt::{PathFor, PathPrompt};
