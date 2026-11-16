//! The one menu widget every context menu is made from: a raised box at the
//! pointer with a list of entries, each with a letter that chooses it. This
//! file only declares and re-exports the modules.

pub mod draw;
pub mod place;
pub mod state;

pub use draw::draw_menu;
pub use place::{entry_at, menu_rect, rows_showing};
pub use state::{Menu, MenuEntry};
