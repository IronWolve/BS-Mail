//! The safe copy of an HTML message, for opening in the browser: the part's
//! HTML is parsed, and a new page is written from the parts of it that an
//! allow-list names. Scripts, frames, forms, embedded objects, `base`, `meta`,
//! event attributes, styles that name a file and everything that would load
//! from the network are left out, and what was left out is counted. This file
//! only declares and re-exports the modules.

mod allow;
mod css;
mod dom;
mod limits;
mod page;
mod sanitize;
mod urls;
mod write;

pub(crate) use page::{Outcome, build_page};
