//! A backend that serves several mail accounts as one.
//!
//! `MultiBackend` owns one member backend per account and presents them to
//! the screen as a single `MailBackend`. The screen shows one account at a
//! time; every member is polled all the time so each connection keeps
//! working, and what the screen has no use for is remembered, not forwarded.
//! Ids are kept apart by giving each member a slot (see `slots`). This file
//! only declares and re-exports the modules.

mod account_list;
mod cache;
mod composite;
mod decode_command;
mod entry;
mod failed_member;
mod handle;
mod intake;
mod member;
mod membership;
mod pending;
mod policy;
mod polling;
mod remap_event;
mod retire;
mod router;
pub mod slots;

pub use composite::MultiBackend;
pub use handle::MembersHandle;
pub use member::Member;
pub use slots::Slot;
