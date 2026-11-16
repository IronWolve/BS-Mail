//! The second page of the account form: the settings of one account that
//! the first page has no row for. How often to look for mail, whether a copy
//! of sent mail is kept on the server, how the servers' certificates are
//! judged, the signature, the five folders that are named by hand, and the
//! switch that lets the program change mail.
//!
//! The page keeps what was typed and the choices made; the account file text
//! comes from `settings_toml.rs`, and the engine's own checks run when the
//! program reads that text, so a value the engine refuses comes back with its
//! reason and stays in its box. Nothing on this page is a secret: the
//! password and the client secret are on the first page, drawn as dots.

use crate::composer::EditorBuffer;

use super::form::SetupForm;

/// Which page of the form shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormPage {
    #[default]
    Details,
    Settings,
}

/// The rows of the second page, in the order Tab visits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Poll,
    SentCopy,
    ImapTrust,
    SmtpTrust,
    AllowChanges,
    Signature,
    FolderSent,
    FolderDrafts,
    FolderTrash,
    FolderArchive,
    FolderSpam,
    Back,
    Save,
}

impl SettingsField {
    pub const ALL: [SettingsField; 13] = [
        Self::Poll,
        Self::SentCopy,
        Self::ImapTrust,
        Self::SmtpTrust,
        Self::AllowChanges,
        Self::Signature,
        Self::FolderSent,
        Self::FolderDrafts,
        Self::FolderTrash,
        Self::FolderArchive,
        Self::FolderSpam,
        Self::Back,
        Self::Save,
    ];

    /// The five folder rows, in the order of the `folders` table.
    pub const FOLDERS: [SettingsField; 5] =
        [Self::FolderSent, Self::FolderDrafts, Self::FolderTrash, Self::FolderArchive, Self::FolderSpam];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|field| *field == self).unwrap_or(0)
    }

    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Poll => "Look for mail every",
            Self::SentCopy => "Copy of sent mail",
            Self::ImapTrust => "Incoming check",
            Self::SmtpTrust => "Outgoing check",
            Self::AllowChanges => "Let me change mail",
            Self::Signature => "Signature",
            Self::FolderSent => "Sent folder",
            Self::FolderDrafts => "Drafts folder",
            Self::FolderTrash => "Trash folder",
            Self::FolderArchive => "Archive folder",
            Self::FolderSpam => "Spam folder",
            Self::Back | Self::Save => "",
        }
    }

    /// The key of the `folders` table a folder row is for.
    pub fn folder_key(self) -> Option<&'static str> {
        Some(match self {
            Self::FolderSent => "sent",
            Self::FolderDrafts => "drafts",
            Self::FolderTrash => "trash",
            Self::FolderArchive => "archive",
            Self::FolderSpam => "spam",
            _ => return None,
        })
    }

    /// One of the two buttons at the bottom.
    pub fn is_button(self) -> bool {
        matches!(self, Self::Back | Self::Save)
    }

    /// A row that is a choice or a button, not text.
    pub fn is_choice(self) -> bool {
        matches!(self, Self::SentCopy | Self::ImapTrust | Self::SmtpTrust | Self::AllowChanges | Self::Back | Self::Save)
    }

    /// What an empty text row says it will do, drawn dimly until it has focus.
    pub fn placeholder(self) -> &'static str {
        match self {
            Self::Poll => "the program's own choice",
            Self::Signature => "nothing under a new message",
            _ if self.folder_key().is_some() => "guess from the server",
            _ => "",
        }
    }
}

/// What has been typed and chosen on the second page.
#[derive(Debug, Clone)]
pub struct AccountSettings {
    /// Seconds between checks for new mail; empty is the program's own
    /// choice.
    pub poll: EditorBuffer,
    /// The text put under a new message, in lines.
    pub signature: EditorBuffer,
    /// The five folders, in the order of `SettingsField::FOLDERS`.
    pub folders: [EditorBuffer; 5],
    /// Whether a copy of sent mail is kept on the server, as chosen here;
    /// `None` follows the provider (one that files a copy itself gets none).
    pub sent_copy: Option<bool>,
    /// Whether the incoming server's certificate is left unchecked, as chosen
    /// here; `None` follows the server (only one on this computer is).
    pub imap_trust_any: Option<bool>,
    /// The same for the outgoing server.
    pub smtp_trust_any: Option<bool>,
    pub focus: SettingsField,
}

impl Default for AccountSettings {
    fn default() -> Self {
        let single = || EditorBuffer::single_line("");
        Self {
            poll: single(),
            signature: EditorBuffer::new(""),
            folders: [single(), single(), single(), single(), single()],
            sent_copy: None,
            imap_trust_any: None,
            smtp_trust_any: None,
            focus: SettingsField::Poll,
        }
    }
}

impl AccountSettings {
    /// Whether nothing has been typed or chosen on the page.
    pub fn is_blank(&self) -> bool {
        self.poll.is_empty()
            && self.signature.is_empty()
            && self.folders.iter().all(EditorBuffer::is_empty)
            && self.sent_copy.is_none()
            && self.imap_trust_any.is_none()
            && self.smtp_trust_any.is_none()
    }
}

impl SetupForm {
    /// Shows the second page, with the keyboard on its first row.
    pub fn show_settings(&mut self) {
        self.page = FormPage::Settings;
        self.settings.focus = SettingsField::Poll;
        self.notice = None;
        self.quit_armed = false;
    }

    /// Shows the first page again; what was typed on the second stays.
    pub fn show_details(&mut self) {
        self.page = FormPage::Details;
        self.quit_armed = false;
    }

    /// The text buffer of a row of the second page, when it has one.
    pub fn settings_buffer(&self, field: SettingsField) -> Option<&EditorBuffer> {
        match field {
            SettingsField::Poll => Some(&self.settings.poll),
            SettingsField::Signature => Some(&self.settings.signature),
            other => SettingsField::FOLDERS.iter().position(|folder| *folder == other).map(|at| &self.settings.folders[at]),
        }
    }

    pub fn settings_buffer_mut(&mut self, field: SettingsField) -> Option<&mut EditorBuffer> {
        match field {
            SettingsField::Poll => Some(&mut self.settings.poll),
            SettingsField::Signature => Some(&mut self.settings.signature),
            other => SettingsField::FOLDERS.iter().position(|folder| *folder == other).map(|at| &mut self.settings.folders[at]),
        }
    }

    /// Whether a copy of sent mail is kept on the server: what was chosen,
    /// else what the provider's servers want.
    pub fn sent_copy_on(&self) -> bool {
        self.settings.sent_copy.unwrap_or(!self.files_sent_itself)
    }

    /// Changes the choice the keyboard is on.
    pub fn change_settings_choice(&mut self) {
        match self.settings.focus {
            SettingsField::SentCopy => self.settings.sent_copy = Some(!self.sent_copy_on()),
            SettingsField::ImapTrust => self.settings.imap_trust_any = Some(!self.imap_trust_any()),
            SettingsField::SmtpTrust if self.has_outgoing_server() => self.settings.smtp_trust_any = Some(!self.smtp_trust_any()),
            SettingsField::AllowChanges => self.allow_changes = !self.allow_changes,
            _ => return,
        }
        self.notice = None;
        self.info = None;
        self.quit_armed = false;
    }

    /// Whether there is an outgoing server for the outgoing choices to be
    /// about.
    pub fn has_outgoing_server(&self) -> bool {
        !self.smtp_host.text().trim().is_empty()
    }

    /// The words a choice row shows.
    pub fn settings_choice_text(&self, field: SettingsField) -> String {
        let checked = |any: bool| if any { "< certificate not checked >" } else { "< certificate checked >" };
        match field {
            SettingsField::SentCopy if self.sent_copy_on() => "< yes: kept on the server >".to_string(),
            SettingsField::SentCopy => "< no: the server files its own >".to_string(),
            SettingsField::ImapTrust => checked(self.imap_trust_any()).to_string(),
            SettingsField::SmtpTrust if self.has_outgoing_server() => checked(self.smtp_trust_any()).to_string(),
            SettingsField::SmtpTrust => "no outgoing server".to_string(),
            SettingsField::AllowChanges if self.allow_changes => "Yes: mail can be changed and sent".to_string(),
            SettingsField::AllowChanges => "No: read-only, nothing is changed or sent".to_string(),
            _ => String::new(),
        }
    }

    /// After the engine refused the account file with `reason`: shows the page
    /// of the setting the reason names, with the keyboard on its row, so the
    /// bad value is in front of the reason. A reason that names none of them
    /// changes nothing.
    pub fn show_problem_row(&mut self, reason: &str) {
        let named = |key: &str| reason.contains(&format!("`{key}")) || reason.contains(&format!("{key}`"));
        let field = if let Some(folder) = SettingsField::FOLDERS.iter().find(|folder| folder.folder_key().is_some_and(|key| named(&format!("folders.{key}")))) {
            Some(*folder)
        } else if named("smtp_trust") {
            Some(SettingsField::SmtpTrust)
        } else if named("trust") {
            Some(SettingsField::ImapTrust)
        } else if named("poll_seconds") {
            Some(SettingsField::Poll)
        } else if named("signature") {
            Some(SettingsField::Signature)
        } else if named("save_sent") {
            Some(SettingsField::SentCopy)
        } else {
            None
        };
        if let Some(field) = field {
            self.page = FormPage::Settings;
            self.settings.focus = field;
        }
    }
}

#[cfg(test)]
include!("settings_page.tests.rs");
