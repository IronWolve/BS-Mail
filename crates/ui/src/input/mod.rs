//! What the screen does with keys, the pointer and the wheel: movement,
//! focus, selection and the commands those cause. This file only declares
//! and re-exports the modules.

pub mod account_cursor;
pub mod accounts;
pub mod action;
pub mod addresses;
pub mod changes;
pub mod compose;
mod compose_start;
pub mod command_queue;
pub mod delete_prompt;
pub mod dispatch;
pub mod drag;
pub mod dragging;
pub mod folder_change;
pub mod help_scroll;
pub mod footer_clicks;
pub mod goto_keys;
pub mod hidden;
pub mod hover;
pub mod hit;
pub mod links;
pub mod mail_keys;
pub mod marks;
pub mod memory;
pub mod menu_run;
pub mod menu_tables;
pub mod menus;
pub mod message_drag;
pub mod motion;
pub mod movement;
pub mod new_mail;
pub mod options;
pub mod options_press;
pub mod paired_text;
pub mod pointer_events;
pub mod pointer_shape;
pub mod picker;
pub mod picker_mouse;
pub mod read_next;
pub mod restore;
pub mod row_clicks;
pub mod role_moves;
pub mod scrollbars;
pub mod search;
pub mod search_mouse;
pub mod setup;
pub mod tree;
pub mod viewport;
pub mod wheel;

pub use action::{Action, action_for};
pub use command_queue::CommandQueue;
pub use drag::Drag;
pub use hit::{Hit, HitView, account_at, hit_at};
