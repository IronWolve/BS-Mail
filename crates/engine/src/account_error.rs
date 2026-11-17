//! Why an account file was refused.

use std::fmt;

/// Why an account file was refused.
#[derive(Debug)]
pub enum ConfigError {
    /// The file could not be read.
    Read(String),
    /// The text is not valid TOML or has a wrong or missing field.
    Parse(String),
    /// The password is meant to be in the vault and cannot be had from it;
    /// holds the sentence to show. It never holds a secret.
    Vault(String),
    /// A value is present but not allowed.
    Invalid(String),
    /// Other users can read the file (Unix mode bits).
    Permissions { path: String, mode: u32 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(text) => write!(f, "cannot read the account file: {text}"),
            Self::Parse(text) => write!(f, "the account file is not valid: {text}"),
            Self::Vault(text) => f.write_str(text),
            Self::Invalid(text) => write!(f, "the account file is not allowed: {text}"),
            Self::Permissions { path, mode } => write!(
                f,
                "the account file {path} holds a password or client secret and is readable by \
                 other users (mode {mode:o}); run: chmod 600 {path}"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}
