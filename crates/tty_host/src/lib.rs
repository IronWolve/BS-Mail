//! The terminal host: writes frames into a terminal as escape sequences and
//! turns terminal input into input events. This file only declares and
//! re-exports the modules.

pub mod ansi;
pub mod draw;
pub mod event_loop;
pub mod host;
pub mod input;
pub mod osc;
pub mod probe;
pub mod screen_size;
pub mod session;

pub use ansi::{PaintOptions, paint_frame};
pub use draw::Painter;
pub use host::TtyHost;
pub use probe::{color_depth_from_env, parse_depth};
pub use screen_size::terminal_size;
