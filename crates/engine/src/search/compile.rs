//! Turning an expression into IMAP search keys.
//!
//! Terms side by side become a plain list of keys; `or` becomes the
//! two-operand `OR` key, chained to the right; `not` becomes `NOT`. A part
//! that is more than one key is put in parentheses wherever it could be
//! misread. A bare word is `OR SUBJECT x OR FROM x TEXT x`: TEXT alone is
//! the same set on a server that reads encoded header words, and the
//! longer form also finds those hits on one that does not.

use super::ast::{Expr, Status, Term};
use super::error::SearchError;
use super::wire::{KeyWriter, Keys};

/// The keys for `expr`; `None` means every message, `ALL`.
pub fn compile(expr: Option<&Expr>) -> Result<Keys, SearchError> {
    let mut out = KeyWriter::default();
    match expr {
        None => out.text("ALL"),
        Some(expr) => write_expr(&mut out, expr, false),
    }
    out.finish().map_err(SearchError::new)
}

/// True for a part that is more than one key.
fn is_compound(expr: &Expr) -> bool {
    matches!(expr, Expr::And(_) | Expr::Or(_) | Expr::Term(Term::Any(_)))
}

fn write_expr(out: &mut KeyWriter, expr: &Expr, wrap: bool) {
    let bracket = wrap && is_compound(expr);
    if bracket {
        out.text("(");
    }
    match expr {
        Expr::Term(term) => write_term(out, term),
        Expr::And(items) => {
            for (at, item) in items.iter().enumerate() {
                if at > 0 {
                    out.text(" ");
                }
                write_expr(out, item, true);
            }
        }
        Expr::Or(items) => write_or(out, items),
        Expr::Not(inner) => {
            out.text("NOT ");
            write_expr(out, inner, true);
        }
    }
    if bracket {
        out.text(")");
    }
}

/// `OR a OR b c`: the two-operand key chained to the right.
fn write_or(out: &mut KeyWriter, items: &[Expr]) {
    for (at, item) in items.iter().enumerate() {
        if at + 1 < items.len() {
            out.text("OR ");
        }
        write_expr(out, item, true);
        if at + 1 < items.len() {
            out.text(" ");
        }
    }
}

fn write_term(out: &mut KeyWriter, term: &Term) {
    match term {
        Term::Any(text) => {
            out.text("OR SUBJECT ");
            out.string(text);
            out.text(" OR FROM ");
            out.string(text);
            out.text(" TEXT ");
            out.string(text);
        }
        Term::From(text) => field(out, "FROM ", text),
        Term::To(text) => field(out, "TO ", text),
        Term::Cc(text) => field(out, "CC ", text),
        Term::Subject(text) => field(out, "SUBJECT ", text),
        Term::Body(text) => field(out, "BODY ", text),
        Term::Tag(word) => out.text(&format!("KEYWORD {word}")),
        Term::Is(status) => out.text(status_key(*status)),
        Term::Before(day) => out.text(&format!("BEFORE {}", day.imap())),
        Term::After(day) => out.text(&format!("SINCE {}", day.imap())),
        Term::Larger(bytes) => out.text(&format!("LARGER {bytes}")),
        Term::Smaller(bytes) => out.text(&format!("SMALLER {bytes}")),
        Term::Folder(_) => out.fail("folder: cannot be sent to the server as a search key"),
    }
}

fn field(out: &mut KeyWriter, key: &str, text: &str) {
    out.text(key);
    out.string(text);
}

const fn status_key(status: Status) -> &'static str {
    match status {
        Status::Unread => "UNSEEN",
        Status::Read => "SEEN",
        Status::Flagged => "FLAGGED",
        Status::Answered => "ANSWERED",
        Status::Draft => "DRAFT",
    }
}

#[cfg(test)]
include!("compile.tests.rs");
