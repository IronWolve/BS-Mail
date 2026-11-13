//! The contract between the screen code and any host: frames, effects, input
//! events, pointer shapes, capabilities and the presenter trait.

pub mod capabilities;
pub mod effect;
pub mod frame;
pub mod input_event;
pub mod pointer_shape;
pub mod presenter;

pub use capabilities::{Capabilities, ClipboardKind, ColorDepth, EffectSupport};
pub use effect::{Effect, GaugeOutline, Glow, HalfRowShift, RoundedFill, ScaledText, TextFill};
pub use frame::{Buffer, Color, CursorRequest, CursorShape, Frame, Modifier, Rect, Style};
pub use input_event::{
    ButtonState, CellPosition, CompositionEvent, InputEvent, KeyEvent, LogicalKey, Modifiers,
    NamedKey, PhysicalKey, PointerButton,
};
pub use pointer_shape::PointerShape;
pub use presenter::{EventSink, Flow, HostError, Presenter, Waker};
