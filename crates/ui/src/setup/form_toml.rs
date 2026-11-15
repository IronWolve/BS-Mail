//! The account file text the form makes, and the checks that come first.
//!
//! The form never writes the file: it checks what was typed and hands this
//! text to the program, which parses it with the engine's own rules.

use crate::composer::EditorBuffer;

use super::account_id::account_id_for;
use super::fields::SignIn;
use super::form::SetupForm;

/// The longest client id or secret taken; real ones are far shorter.
const MAX_CLIENT_TEXT: usize = 512;

const NO_BROWSER_FOR_PROVIDER: &str = "signing in through the browser is built in for some providers only; for this \
     one, set the oauth keys in the account file by hand (see docs/oauth-setup.md)";
const NO_CLIENT_ID: &str = "type the client id from your provider's developer console (see docs/oauth-setup.md)";

impl SetupForm {
    /// The id the account gets: the one being edited, or one made from the
    /// address. The program makes sure it is not taken.
    pub fn account_id(&self) -> String {
        match &self.edit_id {
            Some(id) => id.clone(),
            None => account_id_for(&self.email.text()),
        }
    }

    /// A sign-in through the browser for a provider the program knows, with no
    /// client id typed or saved yet: the one thing the user must get first.
    pub fn needs_client_id(&self) -> bool {
        self.signin == SignIn::Browser && self.oauth_provider.is_some() && self.client_id.text().trim().is_empty()
    }

    /// The account file this form makes, or what is wrong with it.
    pub fn to_account_toml(&self) -> Result<String, String> {
        let email = self.email.text().trim().to_string();
        if !valid_address(&email) {
            return Err("type your full email address, such as you@example.org".to_string());
        }
        let imap_host = host_of(&self.imap_host, "the incoming server")?;
        let imap_port = port_of(&self.imap_port, "the incoming server's port")?;
        let username = nonempty(&self.username).unwrap_or_else(|| email.clone());
        let name = nonempty(&self.name).unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string());

        let mut table = toml::Table::new();
        let mut put = |key: &str, value: toml::Value| {
            table.insert(key.to_string(), value);
        };
        put("id", self.account_id().into());
        put("name", name.into());
        put("address", email.into());
        put("imap_host", imap_host.clone().into());
        put("imap_port", i64::from(imap_port).into());
        put("security", self.imap_security.toml_name().into());
        put("username", username.into());
        match self.signin {
            SignIn::Password => {
                let password = self.password.text();
                if password.is_empty() {
                    return Err("type the password (an app password, where your provider offers them)".to_string());
                }
                put("password", password.into());
            }
            SignIn::Browser => {
                let provider = self.oauth_provider.ok_or(NO_BROWSER_FOR_PROVIDER)?;
                put("auth", "oauth2".into());
                put("oauth_provider", provider.into());
                put("oauth_client_id", client_text(&self.client_id, "client id")?.ok_or(NO_CLIENT_ID)?.into());
                if let Some(secret) = client_text(&self.client_secret, "client secret")? {
                    put("oauth_client_secret", secret.into());
                }
            }
        }
        if self.imap_trust_any() {
            put("trust", "any".into());
        }
        put("read_only", (!self.allow_changes).into());
        if let Some(smtp_host) = nonempty(&self.smtp_host) {
            let smtp_host = host_of_text(&smtp_host, "the outgoing server")?;
            let smtp_port = port_of(&self.smtp_port, "the outgoing server's port")?;
            put("smtp_host", smtp_host.clone().into());
            put("smtp_port", i64::from(smtp_port).into());
            put("smtp_security", self.smtp_security.toml_name().into());
            if self.smtp_trust_any() {
                put("smtp_trust", "any".into());
            }
            if self.files_sent_itself {
                put("save_sent", false.into());
            }
        }
        toml::to_string(&table).map_err(|error| format!("the settings could not be written: {error}"))
    }
}

fn nonempty(buffer: &EditorBuffer) -> Option<String> {
    let text = buffer.text().trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// A client id or secret: one run of printable characters, or nothing.
fn client_text(buffer: &EditorBuffer, what: &str) -> Result<Option<String>, String> {
    let Some(text) = nonempty(buffer) else {
        return Ok(None);
    };
    let bad = text.chars().any(|ch| ch.is_whitespace() || ch.is_control()) || text.len() > MAX_CLIENT_TEXT;
    if bad {
        return Err(format!("check the {what}: it is one piece of text with no spaces"));
    }
    Ok(Some(text))
}

fn host_of(buffer: &EditorBuffer, what: &str) -> Result<String, String> {
    host_of_text(buffer.text().trim(), what)
}

fn host_of_text(text: &str, what: &str) -> Result<String, String> {
    let ok = !text.is_empty()
        && text.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | ':' | '[' | ']' | '_'));
    if ok { Ok(text.to_string()) } else { Err(format!("check {what}: a name like imap.example.org")) }
}

fn port_of(buffer: &EditorBuffer, what: &str) -> Result<u16, String> {
    match buffer.text().trim().parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err(format!("check {what}: a number from 1 to 65535")),
    }
}

/// One `@`, text on both sides, nothing that could not be in an address.
fn valid_address(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.contains('@')
        && !text.chars().any(|ch| ch.is_whitespace() || ch.is_control() || matches!(ch, '<' | '>' | ',' | ';' | '"'))
}

#[cfg(test)]
include!("form_toml.tests.rs");
