//! Plain data about what is selected, scrolled, focused and open, plus the
//! mail data the regions draw. This file only declares and re-exports them.

pub mod focus;
pub mod mail_snapshot;
pub mod overlays;
pub mod page_merge;
pub mod place;
pub mod scroll;
pub mod selection;

pub use focus::Focus;
pub use mail_snapshot::{HostSummary, ListPage, MailSnapshot};
pub use overlays::{OverlayKind, Overlays};
pub use place::Place;
pub use scroll::{ScrollState, first_row_at_offsets, first_row_showing};
pub use selection::Selection;
