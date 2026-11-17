//! Taking the secrets out of account file text. The form, and an account file
//! an older version wrote, hold `password` (and sometimes `smtp_password` and
//! `oauth_client_secret`) in plain text; the file the program writes now holds
//! none of them, and says `secrets = "vault"` instead. The secrets go to the
//! vault (`vault_store.rs`).

use engine::{MARKER_KEY, MARKER_VALUE};
use model::Secret;
use toml_edit::{DocumentMut, Item, Table, value};

const PASSWORD: &str = "password";
const SMTP_PASSWORD: &str = "smtp_password";
const CLIENT_SECRET: &str = "oauth_client_secret";

/// Account file text without its secrets, and the secrets.
pub struct Taken {
    /// The text with the `password`, `smtp_password` and `oauth_client_secret`
    /// lines gone and every other line, comment and key as it was.
    pub text: String,
    pub password: Option<Secret>,
    pub smtp_password: Option<Secret>,
    pub client_secret: Option<Secret>,
}

impl std::fmt::Debug for Taken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Taken")
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("smtp_password", &self.smtp_password.as_ref().map(|_| "<redacted>"))
            .field("client_secret", &self.client_secret.as_ref().map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

impl Taken {
    /// Whether there was a secret of any kind.
    pub fn any(&self) -> bool {
        self.count() > 0
    }

    /// How many secrets there were.
    pub fn count(&self) -> usize {
        [&self.password, &self.smtp_password, &self.client_secret].iter().filter(|secret| secret.is_some()).count()
    }
}

/// Takes the top-level `password`, `smtp_password` and `oauth_client_secret`
/// out of `text`. Only a non-empty text value is taken; anything else is left
/// where it is, for the engine to refuse as it always has. `Err` when the text
/// is not TOML.
pub fn take_passwords(text: &str) -> Result<Taken, String> {
    let mut document = parse(text)?;
    let table = document.as_table_mut();
    let password = take(table, PASSWORD);
    let smtp_password = take(table, SMTP_PASSWORD);
    let client_secret = take(table, CLIENT_SECRET);
    Ok(Taken { text: document.to_string(), password, smtp_password, client_secret })
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
