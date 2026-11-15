//! The search query language: a lexer, a parser and a compiler to IMAP
//! search keys. Pure text in, text out; nothing here talks to a server.
//!
//! A query is words separated by spaces (all must match), `or`, `not` or a
//! leading `-`, parentheses, `"quoted phrases"` and fields: `from:`, `to:`,
//! `cc:`, `subject:`, `body:`, `tag:`, `is:`, `before:`, `after:`, `size:`
//! and `folder:`.

mod ast;
mod compile;
mod dates;
mod error;
mod lexer;
mod parser;
mod scope;
mod sizes;
mod terms;
mod wire;

pub use ast::{Date, Expr, Status, Term};
pub use compile::compile;
pub use parser::parse;
pub use wire::{Keys, Part};
