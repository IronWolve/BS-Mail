//! The account form: what has been typed, which field has focus, and the
//! account file text it makes.
//!
//! Typing an address fills in the servers for that provider until the user
//! changes a server field themselves; after that their choice stands. The
//! password is kept as typed and only ever drawn as dots.

use crate::composer::EditorBuffer;

use super::providers::{Preset, preset_for};

/// How a connection is protected, as the account file spells it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    Implicit,
    Starttls,
    /// No encryption: for a server on this same machine only.
    None,
}

impl Security {
    pub fn label(self) -> &'static str {
        match self {
            Self::Implicit => "encrypted from the start (TLS)",
            Self::Starttls => "encrypted after hello (STARTTLS)",
            Self::None => "not encrypted (this machine only)",
        }
    }

    pub fn toml_name(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Starttls => "starttls",
            Self::None => "none",
        }
    }

    /// The next choice, wrapping.
    pub fn next(self) -> Self {
        match self {
            Self::Implicit => Self::Starttls,
            Self::Starttls => Self::None,
            Self::None => Self::Implicit,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Implicit => Self::None,
            Self::Starttls => Self::Implicit,
            Self::None => Self::Starttls,
        }
    }
}

/// The rows of the form, in the order Tab visits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Email,
    Password,
    Name,
    ImapHost,
    ImapPort,
    ImapSecurity,
    SmtpHost,
    SmtpPort,
    SmtpSecurity,
    Username,
    AllowChanges,
    Save,
}

impl SetupField {
    pub const ALL: [SetupField; 12] = [
        Self::Email,
        Self::Password,
        Self::Name,
        Self::ImapHost,
        Self::ImapPort,
        Self::ImapSecurity,
        Self::SmtpHost,
        Self::SmtpPort,
        Self::SmtpSecurity,
        Self::Username,
        Self::AllowChanges,
        Self::Save,
    ];

    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|field| *field == self).unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Email => "Email address",
            Self::Password => "Password",
            Self::Name => "Your name",
            Self::ImapHost => "Incoming server",
            Self::ImapPort | Self::SmtpPort => "port",
            Self::ImapSecurity | Self::SmtpSecurity => "  security",
            Self::SmtpHost => "Outgoing server",
            Self::Username => "User name",
            Self::AllowChanges => "Let me change mail",
            Self::Save => "",
        }
    }

    /// A field that is a choice and not text.
    pub fn is_choice(self) -> bool {
        matches!(self, Self::ImapSecurity | Self::SmtpSecurity | Self::AllowChanges | Self::Save)
    }

    fn is_server(self) -> bool {
        matches!(
            self,
            Self::ImapHost | Self::ImapPort | Self::ImapSecurity | Self::SmtpHost | Self::SmtpPort | Self::SmtpSecurity
        )
    }
}

#[derive(Debug, Clone)]
pub struct SetupForm {
    pub email: EditorBuffer,
    pub password: EditorBuffer,
    pub name: EditorBuffer,
    pub imap_host: EditorBuffer,
    pub imap_port: EditorBuffer,
    pub smtp_host: EditorBuffer,
    pub smtp_port: EditorBuffer,
    pub username: EditorBuffer,
    pub imap_security: Security,
    pub smtp_security: Security,
    /// Off keeps the account read-only: nothing is ever changed or sent.
    pub allow_changes: bool,
    pub focus: SetupField,
    /// A problem with what was typed, or the server's answer.
    pub notice: Option<String>,
    /// What the guess for this provider says, shown with the form.
    pub provider_note: &'static str,
    /// The text was handed over and the answer has not come back.
    pub waiting: bool,
    /// Escape was pressed once on a form with something typed in it.
    pub quit_armed: bool,
    servers_touched: bool,
    username_touched: bool,
    name_touched: bool,
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
            name: single(),
            imap_host: single(),
            imap_port: EditorBuffer::single_line("993"),
            smtp_host: single(),
            smtp_port: EditorBuffer::single_line("587"),
            username: single(),
            imap_security: Security::Implicit,
            smtp_security: Security::Starttls,
            allow_changes: false,
            focus: SetupField::Email,
            notice: None,
            provider_note: "",
            waiting: false,
            quit_armed: false,
            servers_touched: false,
            username_touched: false,
            name_touched: false,
        }
    }

    pub fn buffer(&self, field: SetupField) -> Option<&EditorBuffer> {
        Some(match field {
            SetupField::Email => &self.email,
            SetupField::Password => &self.password,
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
        self.email.is_empty() && self.password.is_empty() && !self.name_touched && !self.servers_touched
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

    /// Changes the servers and the user name to match the address, unless
    /// the user has set them.
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
        let Some(preset) = preset_for(email) else {
            self.provider_note = "";
            return;
        };
        self.provider_note = preset.note;
        if !self.servers_touched {
            self.fill_servers(&preset);
        }
    }

    fn fill_servers(&mut self, preset: &Preset) {
        self.imap_host.set_text(&preset.imap_host);
        self.imap_port.set_text(&preset.imap_port.to_string());
        self.imap_security = preset.imap_security;
        self.smtp_host.set_text(&preset.smtp_host);
        self.smtp_port.set_text(&preset.smtp_port.to_string());
        self.smtp_security = preset.smtp_security;
    }

    /// Records that the user, not the guess, set this field.
    pub fn note_edit(&mut self) {
        self.notice = None;
        self.quit_armed = false;
        if self.focus.is_server() {
            self.servers_touched = true;
        }
        if self.focus == SetupField::Username {
            self.username_touched = true;
        }
        if self.focus == SetupField::Name {
            self.name_touched = true;
        }
    }

    /// Changes a choice field: security forward or back, the checkbox.
    pub fn change_choice(&mut self, forward: bool) {
        match self.focus {
            SetupField::ImapSecurity => {
                self.imap_security = if forward { self.imap_security.next() } else { self.imap_security.previous() };
            }
            SetupField::SmtpSecurity => {
                self.smtp_security = if forward { self.smtp_security.next() } else { self.smtp_security.previous() };
            }
            SetupField::AllowChanges => self.allow_changes = !self.allow_changes,
            _ => return,
        }
        self.note_edit();
    }

    /// The account file this form makes, or what is wrong with it.
    pub fn to_account_toml(&self) -> Result<String, String> {
        let email = self.email.text().trim().to_string();
        if !valid_address(&email) {
            return Err("type your full email address, such as you@example.org".to_string());
        }
        let password = self.password.text();
        if password.is_empty() {
            return Err("type the password (an app password, where your provider offers them)".to_string());
        }
        let imap_host = host_of(&self.imap_host, "the incoming server")?;
        let imap_port = port_of(&self.imap_port, "the incoming server's port")?;
        let username = nonempty(&self.username).unwrap_or_else(|| email.clone());
        let name = nonempty(&self.name).unwrap_or_else(|| email.split('@').next().unwrap_or("").to_string());

        let mut table = toml::Table::new();
        let mut put = |key: &str, value: toml::Value| {
            table.insert(key.to_string(), value);
        };
        put("id", "main".into());
        put("name", name.into());
        put("address", email.into());
        put("imap_host", imap_host.clone().into());
        put("imap_port", i64::from(imap_port).into());
        put("security", self.imap_security.toml_name().into());
        put("username", username.into());
        put("password", password.into());
        if is_local(&imap_host) {
            put("trust", "any".into());
        }
        put("read_only", (!self.allow_changes).into());
        if let Some(smtp_host) = nonempty(&self.smtp_host) {
            let smtp_host = host_of_text(&smtp_host, "the outgoing server")?;
            let smtp_port = port_of(&self.smtp_port, "the outgoing server's port")?;
            put("smtp_host", smtp_host.clone().into());
            put("smtp_port", i64::from(smtp_port).into());
            put("smtp_security", self.smtp_security.toml_name().into());
            if is_local(&smtp_host) {
                put("smtp_trust", "any".into());
            }
        }
        toml::to_string(&table).map_err(|error| format!("the settings could not be written: {error}"))
    }
}

fn nonempty(buffer: &EditorBuffer) -> Option<String> {
    let text = buffer.text().trim().to_string();
    (!text.is_empty()).then_some(text)
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

/// A server on this machine, whose own certificate is its own.
fn is_local(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

#[cfg(test)]
include!("form.tests.rs");
