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
use super::local_host::is_local_host;
use super::provider_menu::{ProviderChoice, ProviderMenu};
use super::providers::Preset;
use super::saved_client::SavedClient;

const NO_BROWSER_SIGN_IN: &str =
    "This provider has no sign-in through the browser for mail programs: use a password, or an app password where it offers them.";
const NO_ADDRESS_YET: &str = "Type your email address first: whether the browser can be used depends on your provider.";

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
    /// Which provider the servers are for: the one the address says, or one
    /// picked from the list.
    pub provider: ProviderChoice,
    /// The provider list, while it is open.
    pub provider_menu: Option<ProviderMenu>,
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
    /// The incoming server of the account being edited, when its file says
    /// `trust = "verify"`: that choice is kept for that host, not replaced by
    /// the local-server rule.
    pub(super) verified_imap_host: Option<String>,
    /// The same for the outgoing server and `smtp_trust`.
    pub(super) verified_smtp_host: Option<String>,
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
            provider: ProviderChoice::Auto,
            provider_menu: None,
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
            verified_imap_host: None,
            verified_smtp_host: None,
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

    /// Whether anything has been typed. A client id or secret and a user name
    /// count once the user wrote them; ones the program filled in do not.
    pub fn is_empty(&self) -> bool {
        let client_typed = self.client_touched && !(self.client_id.is_empty() && self.client_secret.is_empty());
        let username_typed = self.username_touched && !self.username.is_empty();
        self.email.is_empty()
            && self.password.is_empty()
            && !client_typed
            && !username_typed
            && !self.name_touched
            && !self.servers_touched
            && self.provider == ProviderChoice::Auto
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
        let preset = self.preset_in_use(email);
        self.provider_note = preset.as_ref().map_or("", |preset| preset.note);
        self.oauth_provider = preset.as_ref().and_then(|preset| preset.oauth);
        self.fill_saved_client();
        if !self.signin_touched {
            let browser = preset.as_ref().is_some_and(|preset| preset.browser_first);
            self.signin = if browser { SignIn::Browser } else { SignIn::Password };
        } else if self.signin == SignIn::Browser && self.oauth_provider.is_none() && self.edit_id.is_none() {
            // The address changed to one with no browser sign-in: a new
            // account cannot stay on a way in that does not exist for it.
            self.signin = SignIn::Password;
            self.info = Some(NO_BROWSER_SIGN_IN.to_string());
        }
        if !self.servers_touched {
            // An address with no known provider must not keep the servers of
            // the one the address was before: the password would go there.
            match preset {
                Some(preset) => self.fill_servers(&preset),
                None => self.clear_servers(),
            }
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

    /// The servers of a form nobody has filled: no host, the usual ports.
    fn clear_servers(&mut self) {
        self.imap_host.set_text("");
        self.imap_port.set_text("993");
        self.imap_security = Security::Implicit;
        self.smtp_host.set_text("");
        self.smtp_port.set_text("587");
        self.smtp_security = Security::Starttls;
        self.files_sent_itself = false;
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

    /// Whether the servers in the form are those of a provider that files a
    /// copy of sent mail itself, so the program must not add another.
    fn servers_file_sent_mail(&self) -> bool {
        let email = self.email.text();
        let Some(preset) = self.preset_in_use(email.trim()) else {
            return false;
        };
        let same = |buffer: &EditorBuffer, host: &str| buffer.text().trim().eq_ignore_ascii_case(host);
        preset.files_sent_itself && same(&self.imap_host, &preset.imap_host) && same(&self.smtp_host, &preset.smtp_host)
    }

    /// Records that the user, not the guess, set this field.
    pub fn note_edit(&mut self) {
        self.notice = None;
        self.info = None;
        self.quit_armed = false;
        if self.focus.is_server() {
            self.servers_touched = true;
            // Only leaving the provider's own servers ends the rule; a
            // security or port change, or a host put back, keeps it.
            self.files_sent_itself = self.servers_file_sent_mail();
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
                let local = is_local_host(self.imap_host.text().trim());
                self.imap_security = self.imap_security.step(forward, local);
            }
            SetupField::SmtpSecurity => {
                let local = is_local_host(self.smtp_host.text().trim());
                self.smtp_security = self.smtp_security.step(forward, local);
            }
            SetupField::Provider => {
                self.open_provider_menu();
                return;
            }
            SetupField::SignIn => {
                if self.signin.other() == SignIn::Browser && self.oauth_provider.is_none() {
                    self.notice = Some(self.no_browser_here().to_string());
                    return;
                }
                self.signin = self.signin.other();
            }
            SetupField::AllowChanges => self.allow_changes = !self.allow_changes,
            _ => return,
        }
        self.note_edit();
    }

    /// Why the browser cannot be chosen: the provider has no browser sign-in
    /// the program knows, or there is no address yet to tell.
    fn no_browser_here(&self) -> &'static str {
        if self.email.text().trim().is_empty() {
            NO_ADDRESS_YET
        } else {
            NO_BROWSER_SIGN_IN
        }
    }
}

#[cfg(test)]
include!("form.tests.rs");
