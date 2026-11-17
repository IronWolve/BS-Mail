//! The window: how it is made, what it does with native events, and the
//! state it keeps between frames.

pub mod create;
pub mod event_loop;
pub mod geometry;
pub mod icon;
pub mod options;
pub mod shortcuts;
pub mod state;

pub use event_loop::run_window;
pub use options::WindowOptions;
pub use state::WindowState;
