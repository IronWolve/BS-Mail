//! Native window events turned into the normalized events the screen reads.

pub mod keyboard;
pub mod pointer;

pub use keyboard::{key_event, modifiers, modifiers_for, typed_text};
pub use pointer::{WheelCarry, middle_paste, pointer_button};
