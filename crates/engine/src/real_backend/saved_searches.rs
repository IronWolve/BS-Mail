//! Opening the Starred and Alerts rows: they are not mailboxes but saved
//! searches over the whole account (the starred messages, the messages with
//! the alert marker), so opening one runs that search and answers with the
//! hits as the folder's page, as a list of messages from many folders. A
//! hit carries its own folder, so reading it and changing it work as in the
//! folder it is filed in. The newest hits only: what the search leaves out
//! is said in the footer.

use model::{Event, FolderId, FolderRole, RequestId};

use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError};

/// The query a saved search stands for.
fn query_of(role: FolderRole) -> Option<&'static str> {
    match role {
        FolderRole::Starred => Some("is:starred"),
        FolderRole::Alerts => Some("is:alert"),
        _ => None,
    }
}

impl Worker {
    /// The page of a saved search. Only a lost connection is an error.
    pub(super) async fn open_saved_search(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
    ) -> Result<Vec<Event>, SessionError> {
        let Some(query) = self.state.folder(folder).and_then(|known| query_of(known.role)) else {
            let reason = format!("unknown folder id: {}", folder.0);
            return Ok(vec![self.sink.failure("open folder", reason)]);
        };
        self.wanted.folder = Some(folder);
        let account = self.account.clone();
        let answered = self.search(session, RequestId(0), &account, query).await?;
        Ok(answered.into_iter().flat_map(|event| page_of(folder, event, &self.account)).collect())
    }
}

/// An answer to the search as the answers to opening a folder: the hits as
/// the folder's page (with a note when the list is not the whole of it), and
/// a failure under the name of opening a folder.
fn page_of(folder: FolderId, event: Event, account: &model::AccountId) -> Vec<Event> {
    match event {
        Event::SearchResults { envelopes, incomplete, .. } => {
            let total = u32::try_from(envelopes.len()).unwrap_or(u32::MAX);
            let page = Event::EnvelopePage { folder, offset: 0, total, envelopes };
            let note = incomplete.map(|reason| Event::OpFailed {
                account: Some(account.clone()),
                operation: "open folder".to_owned(),
                reason,
            });
            std::iter::once(page).chain(note).collect()
        }
        Event::OpFailed { account, reason, .. } => {
            vec![Event::OpFailed { account, operation: "open folder".to_owned(), reason }]
        }
        other => vec![other],
    }
}

#[cfg(test)]
include!("saved_searches.tests.rs");
