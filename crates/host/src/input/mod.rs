//! Native window events turned into the normalized events the screen reads.

pub mod keyboard;
pub mod pointer;

pub use keyboard::{key_event, modifiers, typed_text};
pub use pointer::{WheelCarry, pointer_button};
