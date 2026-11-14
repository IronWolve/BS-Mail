//! Separating `folder:` terms from the rest of a query. They say where to
//! look, not what to look for, so they only make sense joined to the other
//! terms by and: never inside an `or` or after a `not`.

use super::ast::{Expr, Query, Term};
use super::error::SearchError;

/// Splits the folder terms off the top-level and of `expr`.
pub fn split_scope(expr: Expr) -> Result<Query, SearchError> {
    let items = match expr {
        Expr::And(items) => items,
        other => vec![other],
    };
    let mut folders = Vec::new();
    let mut rest = Vec::new();
    for item in items {
        match item {
            Expr::Term(Term::Folder(text)) => folders.push(text),
            other if mentions_folder(&other) => {
                return Err(SearchError::new(
                    "folder: can only be joined to other terms with and; it cannot be inside or, not or a minus",
                ));
            }
            other => rest.push(other),
        }
    }
    let expr = match rest.len() {
        0 => None,
        1 => rest.pop(),
        _ => Some(Expr::And(rest)),
    };
    Ok(Query { folders, expr })
}

fn mentions_folder(expr: &Expr) -> bool {
    match expr {
        Expr::Term(term) => matches!(term, Term::Folder(_)),
        Expr::Not(inner) => mentions_folder(inner),
        Expr::And(items) | Expr::Or(items) => items.iter().any(mentions_folder),
    }
}

#[cfg(test)]
include!("scope.tests.rs");
