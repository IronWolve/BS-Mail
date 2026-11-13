//! Reads the mailbox list: LIST for the names and special-use attributes,
//! STATUS for each mailbox's unread and total counts.

use async_imap::imap_proto::NameAttribute;
use async_imap::types::Name;
use futures_util::TryStreamExt;

use super::error::SessionError;
use super::raw_folder::RawFolder;
use super::session::ImapSession;
use super::timeouts::bounded;

impl ImapSession {
    /// Every mailbox with its counts. A mailbox whose STATUS the server
    /// refuses is listed with zero counts; a lost connection is an error.
    pub async fn list_folders(&mut self) -> Result<Vec<RawFolder>, SessionError> {
        let mut folders = self.list_names().await?;
        for folder in &mut folders {
            if folder.has_attribute("\\noselect") || folder.has_attribute("\\nonexistent") {
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

    async fn list_names(&mut self) -> Result<Vec<RawFolder>, SessionError> {
        if self.caps.special_use && self.caps.list_extended {
            match self.run_list("* RETURN (SPECIAL-USE)").await {
                Err(SessionError::Refused(_)) => {}
                other => return other,
            }
        }
        self.run_list("*").await
    }

    async fn run_list(&mut self, pattern: &str) -> Result<Vec<RawFolder>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result: Result<Vec<Name>, SessionError> = bounded(limit, async {
            session.list(Some(""), Some(pattern)).await?.try_collect().await
        })
        .await;
        let names = self.settle(result)?;
        Ok(names.iter().map(raw_from_name).collect())
    }

    async fn status_counts(&mut self, path: &str) -> Result<(u32, u32), SessionError> {
        let limit = self.limits.command;
        let result = bounded(limit, self.inner.status(path, "(MESSAGES UNSEEN)")).await;
        let found = self.settle(result)?;
        Ok((found.unseen.unwrap_or(0), found.exists))
    }
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
