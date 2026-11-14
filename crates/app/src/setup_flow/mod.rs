//! What the program does for the account form: tries the account the user
//! typed on a backend of its own, and only once the server has let it in
//! saves the file and joins the account to the ones already running.
//!
//! The form asks; this answers. A bad password never reaches the disk, and
//! an old attempt never keeps running behind a new one. This file only
//! declares and re-exports the modules.

mod actions;
mod attempt;
mod browser;
mod finish;
mod flow;
mod remember;
mod starter;
mod trial;

pub use flow::SetupFlow;
