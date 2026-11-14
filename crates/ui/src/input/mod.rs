//! What the screen does with keys, the pointer and the wheel: movement,
//! focus, selection and the commands those cause. This file only declares
//! and re-exports the modules.

pub mod action;
pub mod changes;
pub mod compose;
pub mod command_queue;
pub mod dispatch;
pub mod drag;
pub mod dragging;
pub mod hit;
pub mod movement;
pub mod tree;
pub mod viewport;

pub use action::{Action, action_for};
pub use command_queue::CommandQueue;
pub use drag::Drag;
pub use hit::{Hit, HitView, hit_at};
