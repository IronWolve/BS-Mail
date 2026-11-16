//! Reads the fixture file and converts it into model values the mock backend
//! serves. Every failure is a typed error naming the file and the field.

use std::path::Path;

use model::{
    Account, AccountId, Body, Envelope, Folder, FolderId, FolderRole, MailDate, MessageId, SyncStatus,
};
use toml::de::Error as ParseFailure;
use toml::from_str as parse_fixture_text;

use super::convert_body::body_from_rows;
use super::convert_date::mail_date;
use super::convert_mail::{
    accounts_from_rows, envelopes_from_rows, folders_from_rows, status_from_row,
};
use super::fixture_error::{FieldProblem, FixtureError};
use super::fixture_schema::FixtureFile;

/// All the mock data, as model types. The mock backend keeps one and changes
/// it as commands arrive.
#[derive(Debug, Clone)]
pub struct Fixture {
    /// The moment the UI treats as the present.
    pub now: MailDate,
    pub host_name: String,
    pub system: String,
    pub accounts: Vec<Account>,
    /// The selected account; the folders, messages and status belong to it.
    pub account: AccountId,
    /// Depth-first: each parent is followed by its children.
    pub folders: Vec<Folder>,
    /// Newest first.
    pub messages: Vec<Envelope>,
    pub open_message: MessageId,
    /// The body of the open message; no other message has one.
    pub open_body: Body,
    pub status: SyncStatus,
}

impl Fixture {
    /// Reads and converts the fixture at `path`.
    pub fn from_file(path: &Path) -> Result<Self, FixtureError> {
        let file = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|error| FixtureError::Read {
            file: file.clone(),
            reason: error.to_string(),
        })?;
        Self::load(&file, &text)
    }

    /// Converts fixture text; errors name the file as `<inline>`.
    pub fn from_text(text: &str) -> Result<Self, FixtureError> {
        Self::load("<inline>", text)
    }

    /// The folder the open message lives in, which the UI shows at start-up.
    pub fn open_folder(&self) -> Option<FolderId> {
        self.messages
            .iter()
            .find(|envelope| envelope.id == self.open_message)
            .map(|envelope| envelope.folder)
    }

    fn load(file: &str, text: &str) -> Result<Self, FixtureError> {
        let parsed: FixtureFile =
            parse_fixture_text(text).map_err(|error| parse_error(file, text, &error))?;
        convert(&parsed).map_err(|problem| problem.in_file(file))
    }
}

fn convert(parsed: &FixtureFile) -> Result<Fixture, FieldProblem> {
    let mut accounts = accounts_from_rows(&parsed.accounts)?;
    let account = accounts
        .iter()
        .find(|account| account.selected)
        .map(|account| account.id.clone())
        .ok_or_else(|| FieldProblem::new("accounts.selected", "no account is selected"))?;
    let folders = folders_from_rows(&parsed.folders, &account)?;
    // The account the folders belong to, when the fixture gives it a count,
    // counts its own Inbox.
    let inbox = folders.iter().find(|folder| folder.role == FolderRole::Inbox);
    let counting = accounts.iter_mut().find(|listed| listed.id == account && listed.unread.is_some());
    if let (Some(inbox), Some(listed)) = (inbox, counting) {
        listed.unread = Some(inbox.unread);
    }
    let messages = envelopes_from_rows(&parsed.messages, &folders)?;
    let open_message = MessageId(parsed.open.message);
    if !messages.iter().any(|envelope| envelope.id == open_message) {
        return Err(FieldProblem::new(
            "open.message",
            format!("message {} is not in the fixture", parsed.open.message),
        ));
    }
    Ok(Fixture {
        now: mail_date(&parsed.now, "now")?,
        host_name: parsed.host.name.clone(),
        system: parsed.host.system.clone(),
        accounts,
        account,
        folders,
        messages,
        open_message,
        open_body: body_from_rows(&parsed.open.body, "open.body")?,
        status: status_from_row(&parsed.status)?,
    })
}

/// Wraps a TOML error with the file, the position and the offending line.
fn parse_error(file: &str, text: &str, error: &ParseFailure) -> FixtureError {
    let offset = error.span().map_or(0, |span| span.start).min(text.len());
    let before = &text[..text.floor_char_boundary(offset)];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    FixtureError::Parse {
        file: file.to_owned(),
        line,
        column: before[line_start..].chars().count() + 1,
        message: error.message().to_owned(),
        near: text[line_start..]
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}

#[cfg(test)]
include!("fixture_load.tests.rs");
