//! An account's saved details, to fill the form with when the user edits it.
//!
//! The program reads the account file and hands these over; the form keeps
//! them as typed text. A prefilled field counts as touched, so the guess made
//! from the address never overwrites it. The password and the client secret
//! are never part of any debug output.

use std::fmt;

use super::fields::{Security, SignIn};
use super::form::SetupForm;
use super::providers::preset_for;

/// What the form starts with when an account is edited.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct AccountPrefill {
    /// The id of the account being edited; empty makes a new account.
    pub id: String,
    pub address: String,
    pub name: String,
    pub username: String,
    pub password: String,
    pub imap_host: String,
    pub imap_port: u16,
    /// `implicit`, `starttls` or `none`, as the account file spells it.
    pub imap_security: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_security: String,
    /// `google` or `microsoft` when the account signs in through a browser.
    pub browser_provider: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub allow_changes: bool,
}

impl fmt::Debug for AccountPrefill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountPrefill")
            .field("id", &self.id)
            .field("address", &self.address)
            .field("password", &"<hidden>")
            .field("client_secret", &"<hidden>")
            .finish_non_exhaustive()
    }
}

/// The provider names the form can sign in through a browser for.
fn browser_provider(name: &str) -> Option<&'static str> {
    match name {
        "google" => Some("google"),
        "microsoft" => Some("microsoft"),
        _ => None,
    }
}

impl SetupForm {
    /// The form filled with an existing account's details.
    pub fn from_prefill(prefill: AccountPrefill) -> Self {
        let mut form = Self::new();
        form.email.set_text(&prefill.address);
        form.name.set_text(&prefill.name);
        form.username.set_text(&prefill.username);
        form.password.set_text(&prefill.password);
        form.client_id.set_text(&prefill.client_id);
        form.client_secret.set_text(&prefill.client_secret);
        form.imap_host.set_text(&prefill.imap_host);
        form.imap_port.set_text(&prefill.imap_port.to_string());
        form.imap_security = Security::from_name(&prefill.imap_security).unwrap_or(Security::Implicit);
        form.smtp_host.set_text(&prefill.smtp_host);
        form.smtp_port.set_text(&prefill.smtp_port.to_string());
        form.smtp_security = Security::from_name(&prefill.smtp_security).unwrap_or(Security::Starttls);
        form.allow_changes = prefill.allow_changes;
        form.signin = if prefill.browser_provider.is_some() { SignIn::Browser } else { SignIn::Password };
        form.oauth_provider = prefill.browser_provider.as_deref().and_then(browser_provider);
        form.edit_id = Some(prefill.id.clone()).filter(|id| !id.is_empty());
        form.servers_touched = true;
        form.username_touched = true;
        form.name_touched = true;
        form.signin_touched = true;
        form.note_provider(&prefill);
        form
    }

    /// The provider's note, and whether the provider files sent mail itself,
    /// when the saved servers are the ones the provider is known by.
    fn note_provider(&mut self, prefill: &AccountPrefill) {
        let Some(preset) = preset_for(prefill.address.trim()) else {
            return;
        };
        self.provider_note = preset.note;
        if preset.imap_host == prefill.imap_host && preset.smtp_host == prefill.smtp_host {
            self.files_sent_itself = preset.files_sent_itself;
        }
    }
}

#[cfg(test)]
include!("prefill.tests.rs");
