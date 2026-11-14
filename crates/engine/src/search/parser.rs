//! Reads tokens into an expression. From loosest to tightest binding:
//! `or`, then terms side by side (an implicit and), then `not` or a leading
//! minus, then a term or a parenthesised group.

use super::ast::{Expr, Query, Term};
use super::error::SearchError;
use super::lexer::{Token, lex};
use super::scope::split_scope;
use super::terms::{check_text, field_term};

/// The most terms one query may hold.
pub const MAX_TERMS: usize = 30;

/// Reads a query. A query with nothing in it, or that cannot be understood,
/// is refused with a sentence that names the problem.
pub fn parse(text: &str) -> Result<Query, SearchError> {
    let tokens = lex(text)?;
    if tokens.is_empty() {
        return Err(SearchError::new(
            "the search is empty; type a word or a field such as from:alex",
        ));
    }
    let mut parser = Parser { tokens, at: 0, terms: 0, depth: 0 };
    let expr = parser.or_expr()?;
    if parser.at < parser.tokens.len() {
        return Err(SearchError::new("a ) has no matching ("));
    }
    split_scope(expr)
}

/// Adds one side of an `or`; an `or` inside a group joins its parent.
fn push_or(items: &mut Vec<Expr>, side: Expr) {
    match side {
        Expr::Or(inner) => items.extend(inner),
        other => items.push(other),
    }
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
    terms: usize,
    /// How many groups are open at this point.
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    /// `and_expr ( or and_expr )*`
    fn or_expr(&mut self) -> Result<Expr, SearchError> {
        let mut items = Vec::new();
        push_or(&mut items, self.and_expr()?);
        let mut joined = false;
        while self.peek() == Some(&Token::Or) {
            self.at += 1;
            joined = true;
            push_or(&mut items, self.and_expr()?);
        }
        Ok(if joined { Expr::Or(items) } else { items.remove(0) })
    }

    /// One or more unary expressions side by side.
    fn and_expr(&mut self) -> Result<Expr, SearchError> {
        let mut items: Vec<Expr> = Vec::new();
        while !matches!(self.peek(), None | Some(Token::Or | Token::Close)) {
            match self.unary()? {
                Expr::And(inner) => items.extend(inner),
                other => items.push(other),
            }
        }
        match items.len() {
            0 => Err(self.nothing_here()),
            1 => Ok(items.remove(0)),
            _ => Ok(Expr::And(items)),
        }
    }

    /// The problem when a part of the query has no terms at all.
    fn nothing_here(&self) -> SearchError {
        let after_or = self.at > 0 && self.tokens.get(self.at - 1) == Some(&Token::Or);
        if after_or || self.peek() == Some(&Token::Or) {
            SearchError::new("or needs a search on both sides of it")
        } else if self.peek().is_none() {
            SearchError::new("a ( was opened but never closed")
        } else if self.depth == 0 {
            SearchError::new("a ) has no matching (")
        } else {
            SearchError::new("the parentheses have nothing inside them")
        }
    }

    /// `( not | - )* atom`
    fn unary(&mut self) -> Result<Expr, SearchError> {
        if self.peek() != Some(&Token::Not) {
            return self.atom();
        }
        self.at += 1;
        if matches!(self.peek(), None | Some(Token::Or | Token::Close)) {
            return Err(SearchError::new("not needs a search after it to turn around"));
        }
        Ok(match self.unary()? {
            Expr::Not(inner) => *inner,
            other => Expr::Not(Box::new(other)),
        })
    }

    fn atom(&mut self) -> Result<Expr, SearchError> {
        let Some(token) = self.peek().cloned() else {
            return Err(SearchError::new("the search ends where a term was expected"));
        };
        self.at += 1;
        match token {
            Token::Open => self.group(),
            Token::Word(text) => self.term(Term::Any(check_text(text)?)),
            Token::Field { name, value } => self.term(field_term(&name, &value)?),
            Token::Close | Token::Or | Token::Not => {
                Err(SearchError::new("the search has a stray word where a term was expected"))
            }
        }
    }

    /// The inside of a parenthesised group; the opening bracket is read.
    fn group(&mut self) -> Result<Expr, SearchError> {
        self.depth += 1;
        let inside = self.or_expr()?;
        self.depth -= 1;
        if self.peek() == Some(&Token::Close) {
            self.at += 1;
            Ok(inside)
        } else {
            Err(SearchError::new("a ( was opened but never closed"))
        }
    }

    fn term(&mut self, term: Term) -> Result<Expr, SearchError> {
        self.terms += 1;
        if self.terms > MAX_TERMS {
            return Err(SearchError::new(format!(
                "the search has more than {MAX_TERMS} terms; make it shorter"
            )));
        }
        Ok(Expr::Term(term))
    }
}

#[cfg(test)]
include!("parser.tests.rs");
