//! Icon system: the icon ids, the three glyph sets and the set that fits each
//! glyph to the two-cell slot. This file only declares and re-exports them.

pub mod ascii_set;
pub mod icon_id;
pub mod icon_set;
pub mod nerd_set;
pub mod unicode_set;

pub use icon_id::IconId;
pub use icon_set::{IconKind, IconSet, IconSlot, SLOT_CELLS};
