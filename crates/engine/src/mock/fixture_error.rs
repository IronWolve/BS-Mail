//! The typed error for a fixture that cannot be read, parsed or converted.
//! Every variant names the file and, when it can, the field at fault.

use std::fmt;

/// Why a fixture could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureError {
    /// The file could not be read.
    Read { file: String, reason: String },
    /// The text is not valid TOML or does not fit the fixture's shape.
    Parse {
        file: String,
        /// One-based position of the offending text.
        line: usize,
        column: usize,
        message: String,
        /// The offending source line, so the field name is visible.
        near: String,
    },
    /// The text parsed, but a value is not usable.
    Field {
        file: String,
        /// The field's path, such as `messages[3].date`.
        field: String,
        problem: String,
    },
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { file, reason } => write!(f, "fixture {file}: cannot read: {reason}"),
            Self::Parse {
                file,
                line,
                column,
                message,
                near,
            } => write!(
                f,
                "fixture {file}: line {line}, column {column}: {message} (near `{near}`)"
            ),
            Self::Field {
                file,
                field,
                problem,
            } => write!(f, "fixture {file}: field `{field}`: {problem}"),
        }
    }
}

impl std::error::Error for FixtureError {}

/// A bad value found while converting, before the file name is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FieldProblem {
    pub field: String,
    pub problem: String,
}

impl FieldProblem {
    pub fn new(field: impl Into<String>, problem: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            problem: problem.into(),
        }
    }

    /// Adds the file name, making the public error.
    pub fn in_file(self, file: &str) -> FixtureError {
        FixtureError::Field {
            file: file.to_owned(),
            field: self.field,
            problem: self.problem,
        }
    }
}
