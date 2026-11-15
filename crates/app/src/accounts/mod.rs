//! The accounts of this run and where they are kept: one folder per account
//! inside the data folder, each holding its own `account.toml` and, later, its
//! sign-in tokens and mail cache. This file only declares and re-exports the
//! modules.

mod id;
mod live;
mod members;
mod merge;
mod place;
mod plan;
mod reason;
mod recover;
mod remove;
mod save;
mod scan;
mod summary;

pub use live::Live;
pub use members::members_of;
pub use place::{MOCK_RUN, Place};
pub use plan::{Plan, plan};
pub use reason::reason_of;
pub use recover::recover;
pub use remove::{checked_folder, remove, remove_unsaved};
pub use save::save;
pub use scan::{Scan, scan};
pub use summary::prefill_of;
