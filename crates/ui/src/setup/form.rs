//! The account form: what has been typed, which field has focus, and how the
//! answers to a few fields change the others.
//!
//! Typing an address fills in the servers for that provider until the user
//! changes a server field themselves; after that their choice stands. The
//! password and the client secret are kept as typed and only ever drawn as
//! dots, and `form_debug.rs` keeps them out of any debug output. The account
//! file text the form makes is in `form_toml.rs`.

use crate::composer::EditorBuffer;

use super::fields::{Security, SetupField, SignIn};
use super::providers::{Preset, preset_for};
use super::saved_client::SavedClient;

#[derive(Clone)]
pub struct SetupForm {
    pub email: EditorBuffer,
    pub password: EditorBuffer,
    pub client_id: EditorBuffer,
    pub client_secret: EditorBuffer,
    pub name: EditorBuffer,
    pub imap_host: EditorBuffer,
    pub imap_port: EditorBuffer,
    pub smtp_host: EditorBuffer,
    pub smtp_port: EditorBuffer,
    pub username: EditorBuffer,
    pub imap_security: Security,
    pub smtp_security: Security,
    /// A password, or the provider's page in a browser.
    pub signin: SignIn,
    /// The provider the browser sign-in is set up for, from the address.
    pub oauth_provider: Option<&'static str>,
    /// Off keeps the account read-only: nothing is ever changed or sent.
    pub allow_changes: bool,
    pub focus: SetupField,
    /// A problem with what was typed, or the server's answer.
    pub notice: Option<String>,
    /// Something to tell the user that is not a problem.
    pub info: Option<String>,
    /// What the guess for this provider says, shown with the form.
    pub provider_note: &'static str,
    /// The text was handed over and the answer has not come back.
    pub waiting: bool,
    /// The link the user is to open in a browser, while the program waits for
    /// them to finish there.
    pub link: Option<String>,
    /// The link, copied, until the host has put it on the clipboard.
    pub copied: Option<String>,
    /// Escape was pressed once on a form with something typed in it.
    pub quit_armed: bool,
    /// The id of the account being edited; `None` for a new account, whose id
    /// is made from the address when the file text is made.
    pub(super) edit_id: Option<String>,
    /// The client ids the program already holds, by provider.
    pub(crate) saved_clients: Vec<SavedClient>,
    /// The provider files a copy of sent mail itself, so the program must not.
    pub(super) files_sent_itself: bool,
    pub(super) servers_touched: bool,
    pub(super) username_touched: bool,
    pub(super) name_touched: bool,
    pub(super) signin_touched: bool,
    /// The user, or the account being edited, set the client id or secret, so
    /// a saved one is not put over it.
    pub(super) client_touched: bool,
}

impl Default for SetupForm {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupForm {
    pub fn new() -> Self {
        let single = || EditorBuffer::single_line("");
        Self {
            email: single(),
            password: single(),
            client_id: single(),
            client_secret: single(),
            name: single(),
            imap_host: single(),
            imap_port: EditorBuffer::single_line("993"),
            smtp_host: single(),
            smtp_port: EditorBuffer::single_line("587"),
            username: single(),
            imap_security: Security::Implicit,
            smtp_security: Security::Starttls,
            signin: SignIn::Password,
            oauth_provider: None,
            allow_changes: false,
            focus: SetupField::Email,
            notice: None,
            info: None,
            provider_note: "",
            waiting: false,
            link: None,
            copied: None,
            quit_armed: false,
            edit_id: None,
            saved_clients: Vec::new(),
            files_sent_itself: false,
            servers_touched: false,
            username_touched: false,
            name_touched: false,
            signin_touched: false,
            client_touched: false,
        }
    }

    pub fn buffer(&self, field: SetupField) -> Option<&EditorBuffer> {
        Some(match field {
            SetupField::Email => &self.email,
            SetupField::Password => &self.password,
            SetupField::ClientId => &self.client_id,
            SetupField::ClientSecret => &self.client_secret,
            SetupField::Name => &self.name,
            SetupField::ImapHost => &self.imap_host,
            SetupField::ImapPort => &self.imap_port,
            SetupField::SmtpHost => &self.smtp_host,
            SetupField::SmtpPort => &self.smtp_port,
            SetupField::Username => &self.username,
            _ => return None,
        })
    }

    pub fn focused_buffer(&mut self) -> Option<&mut EditorBuffer> {
        Some(match self.focus {
            SetupField::Email => &mut self.email,
            SetupField::Password => &mut self.password,
            SetupField::ClientId => &mut self.client_id,
            SetupField::ClientSecret => &mut self.client_secret,
            SetupField::Name => &mut self.name,
            SetupField::ImapHost => &mut self.imap_host,
            SetupField::ImapPort => &mut self.imap_port,
            SetupField::SmtpHost => &mut self.smtp_host,
            SetupField::SmtpPort => &mut self.smtp_port,
            SetupField::Username => &mut self.username,
            _ => return None,
        })
    }

    /// Whether anything has been typed.
    pub fn is_empty(&self) -> bool {
        self.email.is_empty()
            && self.password.is_empty()
            && (self.client_id.is_empty() || !self.client_touched)
            && !self.name_touched
            && !self.servers_touched
    }

    /// Whether the row shows with the way of signing in chosen now.
    pub fn is_shown(&self, field: SetupField) -> bool {
        field.is_shown(self.signin)
    }

    /// Moves focus one row, over the rows that are not shown.
    pub fn step_focus(&mut self, forward: bool) {
        let mut to = self.focus;
        loop {
            to = if forward { to.next() } else { to.previous() };
            if self.is_shown(to) {
                break;
            }
        }
        self.move_focus(to);
    }

    /// Moves focus. Leaving the address field fills in the servers for its
    /// provider, as long as the user has not chosen their own.
    pub fn move_focus(&mut self, to: SetupField) {
        let leaving_email = self.focus == SetupField::Email && to != SetupField::Email;
        self.focus = to;
        if leaving_email {
            self.apply_provider();
        }
    }

    /// Changes the servers, the user name and the way of signing in to match
    /// the address, unless the user has set them.
    pub fn apply_provider(&mut self) {
        let email = self.email.text();
        let email = email.trim();
        if !self.username_touched {
            self.username.set_text(email);
        }
        if !self.name_touched {
            let local = email.split('@').next().unwrap_or("");
            self.name.set_text(local);
        }
        let preset = preset_for(email);
        self.provider_note = preset.as_ref().map_or("", |preset| preset.note);
        self.oauth_provider = preset.as_ref().and_then(|preset| preset.oauth);
        self.fill_saved_client();
        if !self.signin_touched {
            let browser = preset.as_ref().is_some_and(|preset| preset.browser_first);
            self.signin = if browser { SignIn::Browser } else { SignIn::Password };
        }
        if let (Some(preset), false) = (preset, self.servers_touched) {
            self.fill_servers(&preset);
        }
    }

    /// Puts the client id the program holds for this provider into the form,
    /// unless the user has typed their own; a provider with none leaves the
    /// rows empty for the user to fill once.
    fn fill_saved_client(&mut self) {
        if self.client_touched {
            return;
        }
        let saved = self
            .oauth_provider
            .and_then(|provider| self.saved_clients.iter().find(|client| client.provider == provider))
            .cloned();
        self.client_id.set_text(saved.as_ref().map_or("", |client| client.client_id.as_str()));
        self.client_secret.set_text(saved.as_ref().map_or("", |client| client.client_secret.as_str()));
        if saved.is_some() && self.signin == SignIn::Browser {
            self.info = Some("Signing in uses the client id this program already has for this provider.".to_string());
        }
    }

    fn fill_servers(&mut self, preset: &Preset) {
        self.imap_host.set_text(&preset.imap_host);
        self.imap_port.set_text(&preset.imap_port.to_string());
        self.imap_security = preset.imap_security;
        self.smtp_host.set_text(&preset.smtp_host);
        self.smtp_port.set_text(&preset.smtp_port.to_string());
        self.smtp_security = preset.smtp_security;
        self.files_sent_itself = preset.files_sent_itself;
    }

    /// Records that the user, not the guess, set this field.
    pub fn note_edit(&mut self) {
        self.notice = None;
        self.info = None;
        self.quit_armed = false;
        if self.focus.is_server() {
            self.servers_touched = true;
            self.files_sent_itself = false;
        }
        match self.focus {
            SetupField::Username => self.username_touched = true,
            SetupField::Name => self.name_touched = true,
            SetupField::SignIn => self.signin_touched = true,
            SetupField::ClientId | SetupField::ClientSecret => self.client_touched = true,
            _ => {}
        }
    }

    /// Changes a choice field: security forward or back, the way of signing
    /// in, the checkbox.
    pub fn change_choice(&mut self, forward: bool) {
        match self.focus {
            SetupField::ImapSecurity => {
                self.imap_security = if forward { self.imap_security.next() } else { self.imap_security.previous() };
            }
            SetupField::SmtpSecurity => {
                self.smtp_security = if forward { self.smtp_security.next() } else { self.smtp_security.previous() };
            }
            SetupField::SignIn => self.signin = self.signin.other(),
            SetupField::AllowChanges => self.allow_changes = !self.allow_changes,
            _ => return,
        }
        self.note_edit();
    }
}

#[cfg(test)]
include!("form.tests.rs");
