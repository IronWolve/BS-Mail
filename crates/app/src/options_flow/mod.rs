//! What the program does when the Options screen asks for something to be
//! done to the accounts: open the form to add or edit one, or remove one. The
//! screen queues requests and answers nothing itself; each is answered here
//! through the view's own setters. This file only declares and re-exports the
//! modules.

mod answer;
mod edit;
mod remove;

pub use answer::{Opened, answer};
