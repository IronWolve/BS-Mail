//! What the screen does with keys, the pointer and the wheel: movement,
//! focus, selection and the commands those cause. This file only declares
//! and re-exports the modules.

pub mod accounts;
pub mod action;
pub mod changes;
pub mod compose;
pub mod command_queue;
pub mod delete_prompt;
pub mod dispatch;
pub mod drag;
pub mod dragging;
pub mod folder_change;
pub mod hidden;
pub mod hit;
pub mod marks;
pub mod movement;
pub mod options;
pub mod options_press;
pub mod paired_text;
pub mod picker;
pub mod search;
pub mod setup;
pub mod tree;
pub mod viewport;

pub use action::{Action, action_for};
pub use command_queue::CommandQueue;
pub use drag::Drag;
pub use hit::{Hit, HitView, account_at, hit_at};
