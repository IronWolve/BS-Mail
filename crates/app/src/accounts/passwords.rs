//! Taking the passwords out of account file text. The form, and an account
//! file an older version wrote, hold `password` (and sometimes
//! `smtp_password`) in plain text; the file the program writes now holds
//! neither, and says `secrets = "vault"` instead. The passwords go to the
//! vault (`vault_store.rs`).

use engine::{MARKER_KEY, MARKER_VALUE};
use model::Secret;
use toml_edit::{DocumentMut, Item, Table, value};

const PASSWORD: &str = "password";
const SMTP_PASSWORD: &str = "smtp_password";

/// Account file text without its passwords, and the passwords.
pub struct Taken {
    /// The text with the `password` and `smtp_password` lines gone and every
    /// other line, comment and key as it was.
    pub text: String,
    pub password: Option<Secret>,
    pub smtp_password: Option<Secret>,
}

impl std::fmt::Debug for Taken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Taken")
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("smtp_password", &self.smtp_password.as_ref().map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

impl Taken {
    /// Whether there was a password of either kind.
    pub fn any(&self) -> bool {
        self.password.is_some() || self.smtp_password.is_some()
    }

    /// How many passwords there were.
    pub fn count(&self) -> usize {
        usize::from(self.password.is_some()) + usize::from(self.smtp_password.is_some())
    }
}

/// Takes the top-level `password` and `smtp_password` out of `text`. Only a
/// non-empty text value is taken; anything else is left where it is, for the
/// engine to refuse as it always has. `Err` when the text is not TOML.
pub fn take_passwords(text: &str) -> Result<Taken, String> {
    let mut document = parse(text)?;
    let table = document.as_table_mut();
    let password = take(table, PASSWORD);
    let smtp_password = take(table, SMTP_PASSWORD);
    Ok(Taken { text: document.to_string(), password, smtp_password })
}

/// `text` with `secrets = "vault"` set (`on`) or taken out again. A `secrets`
/// key that says something else is not the program's and is never removed.
pub fn with_marker(text: &str, on: bool) -> Result<String, String> {
    let mut document = parse(text)?;
    let table = document.as_table_mut();
    if on {
        table.insert(MARKER_KEY, value(MARKER_VALUE));
    } else if table.get(MARKER_KEY).and_then(Item::as_str) == Some(MARKER_VALUE) {
        table.remove(MARKER_KEY);
    }
    Ok(document.to_string())
}

fn parse(text: &str) -> Result<DocumentMut, String> {
    // The parser's own message quotes the line it stopped at.
    text.parse::<DocumentMut>().map_err(|_| "the account text could not be read".to_string())
}

fn take(table: &mut Table, key: &str) -> Option<Secret> {
    let text = table.get(key)?.as_str().filter(|text| !text.is_empty())?.to_owned();
    table.remove(key);
    Some(Secret::new(text))
}

#[cfg(test)]
include!("passwords.tests.rs");
