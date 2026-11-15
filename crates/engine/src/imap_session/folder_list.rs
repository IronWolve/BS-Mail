//! Reads the mailbox list: LIST for the names and special-use attributes,
//! and each mailbox's unread and total counts. A server that offers
//! LIST-STATUS (RFC 5819) gives the counts in the same answer; for any other,
//! or for a mailbox the answer left out, STATUS asks one mailbox at a time.

use std::collections::HashMap;

use async_imap::imap_proto::{NameAttribute, StatusAttribute};
use async_imap::types::{Name, UnsolicitedResponse};
use futures_util::StreamExt;

use super::error::SessionError;
use super::raw_folder::RawFolder;
use super::session::ImapSession;
use super::timeouts::bounded;

/// Unread and total counts by mailbox name, as LIST-STATUS gave them.
type Counts = HashMap<String, (u32, u32)>;

impl ImapSession {
    /// Every mailbox with its counts. A mailbox whose STATUS the server
    /// refuses is listed with zero counts; a lost connection is an error.
    pub async fn list_folders(&mut self) -> Result<Vec<RawFolder>, SessionError> {
        let (mut folders, mut given) = self.list_names().await?;
        for folder in &mut folders {
            if folder.has_attribute("\\noselect") || folder.has_attribute("\\nonexistent") {
                continue;
            }
            if let Some((unread, total)) = given.remove(&folder.path) {
                folder.unread = unread;
                folder.total = total;
                continue;
            }
            match self.status_counts(&folder.path).await {
                Ok((unread, total)) => {
                    folder.unread = unread;
                    folder.total = total;
                }
                Err(error) if error.is_lost() => return Err(error),
                Err(_) => {}
            }
        }
        Ok(folders)
    }

    /// The names, and the counts the server gave with them, if it did. A
    /// server that refuses the return options is asked again without them.
    /// The library ends the answer at the tagged reply without looking at a
    /// refusal in it, so a refusal reads as a list with no names; a mailbox
    /// always has at least an Inbox, so an empty answer is taken for one.
    async fn list_names(&mut self) -> Result<(Vec<RawFolder>, Counts), SessionError> {
        if let Some(options) = self.return_options() {
            match self.run_list(&format!("* RETURN ({options})")).await {
                Err(SessionError::Refused(_)) => {}
                Ok((names, _)) if names.is_empty() => {}
                other => return other,
            }
        }
        self.run_list("*").await
    }

    /// What to ask LIST to return along with the names, when it can.
    fn return_options(&self) -> Option<String> {
        if !self.caps.list_extended {
            return None;
        }
        let mut options = Vec::new();
        if self.caps.special_use {
            options.push("SPECIAL-USE");
        }
        if self.caps.list_status {
            options.push("STATUS (MESSAGES UNSEEN)");
        }
        (!options.is_empty()).then(|| options.join(" "))
    }

    /// Runs LIST. The counts of LIST-STATUS come as STATUS answers between
    /// the names; the library sets them aside in a queue that holds only a
    /// hundred, so the queue is emptied after every name.
    async fn run_list(&mut self, pattern: &str) -> Result<(Vec<RawFolder>, Counts), SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let tray = session.unsolicited_responses.clone();
        let take = |counts: &mut Counts| {
            while let Ok(item) = tray.try_recv() {
                if let Some((name, found)) = counts_of(item) {
                    counts.insert(name, found);
                }
            }
        };
        take(&mut Counts::new());
        let result = bounded(limit, async {
            let mut stream = session.list(Some(""), Some(pattern)).await?;
            let (mut names, mut counts) = (Vec::new(), Counts::new());
            while let Some(name) = stream.next().await {
                names.push(raw_from_name(&name?));
                take(&mut counts);
            }
            take(&mut counts);
            Ok((names, counts))
        })
        .await;
        self.settle(result)
    }

    /// Unread and total counts of one mailbox, from STATUS.
    pub async fn status_counts(&mut self, path: &str) -> Result<(u32, u32), SessionError> {
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.status(path, "(MESSAGES UNSEEN)")).await;
        let found = self.settle(result)?;
        Ok((found.unseen.unwrap_or(0), found.exists))
    }
}

/// The counts in a STATUS answer, with the name of the mailbox they are for.
fn counts_of(item: UnsolicitedResponse) -> Option<(String, (u32, u32))> {
    let UnsolicitedResponse::Status { mailbox, attributes } = item else {
        return None;
    };
    let (mut unread, mut total) = (None, None);
    for attribute in attributes {
        match attribute {
            StatusAttribute::Unseen(n) => unread = Some(n),
            StatusAttribute::Messages(n) => total = Some(n),
            _ => {}
        }
    }
    Some((mailbox, (unread.unwrap_or(0), total?)))
}

fn raw_from_name(name: &Name) -> RawFolder {
    RawFolder {
        path: name.name().to_owned(),
        delimiter: name.delimiter().map(str::to_owned),
        attributes: name.attributes().iter().map(attribute_text).collect(),
        unread: 0,
        total: 0,
    }
}

/// An attribute as lower-case text with its backslash.
fn attribute_text(attribute: &NameAttribute<'_>) -> String {
    let text = match attribute {
        NameAttribute::NoInferiors => "\\noinferiors",
        NameAttribute::NoSelect => "\\noselect",
        NameAttribute::Marked => "\\marked",
        NameAttribute::Unmarked => "\\unmarked",
        NameAttribute::All => "\\all",
        NameAttribute::Archive => "\\archive",
        NameAttribute::Drafts => "\\drafts",
        NameAttribute::Flagged => "\\flagged",
        NameAttribute::Junk => "\\junk",
        NameAttribute::Sent => "\\sent",
        NameAttribute::Trash => "\\trash",
        NameAttribute::Extension(other) => return other.to_lowercase(),
        _ => "\\unknown",
    };
    text.to_owned()
}

#[cfg(test)]
include!("folder_list.tests.rs");
