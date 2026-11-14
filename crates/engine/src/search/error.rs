//! Why a search could not be understood, as one plain sentence.

use std::fmt;

/// A query the program cannot search for. The text names the problem and is
/// shown to the person as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchError(String);

impl SearchError {
    pub(super) fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SearchError {}
