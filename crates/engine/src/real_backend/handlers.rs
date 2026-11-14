//! What each command does over a live session. Every handler returns the
//! events to send; nothing here writes to the server.

use async_imap::types::Fetch;
use model::{AccountId, Envelope, Event, Folder, FolderId, MessageId, SyncState};

use super::lifecycle::{now_unix, progress, quota_event};
use super::worker::Worker;
use crate::imap_session::{ImapSession, SessionError, build_envelope, build_tree, fetched_row};
use crate::mime_render::render_body;

/// How many messages one page of a folder holds.
const PAGE_SIZE: u32 = 50;

impl Worker {
    /// Reads the folder list and quota again: the folder tree, then the sync
    /// status, then the quota.
    pub(super) async fn refresh(
        &mut self,
        session: &mut ImapSession,
    ) -> Result<Vec<Event>, SessionError> {
        let folders = self.read_tree(session).await?;
        self.state.set_folders(folders.clone());
        let quota = session.storage_quota().await?;
        let now = now_unix();
        self.last_sync = Some(now);
        let arrived = self.poll.arrived();
        Ok(vec![
            Event::FolderList {
                account: self.account.clone(),
                folders,
            },
            progress(&self.sink, SyncState::Idle, Some(now), quota, arrived),
            quota_event(&self.sink, quota),
        ])
    }

    /// The folder tree as the server lists it now, with fresh counts. It
    /// does not replace the list held.
    pub(super) async fn read_tree(
        &mut self,
        session: &mut ImapSession,
    ) -> Result<Vec<Folder>, SessionError> {
        let listed = session.list_folders().await?;
        let state = &mut self.state;
        Ok(build_tree(&self.account, listed, &mut |path| state.folder_id(path)))
    }

    /// The newest page of a folder.
    pub(super) async fn open_folder(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
    ) -> Result<Vec<Event>, SessionError> {
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            let reason = format!("unknown folder id: {}", folder.0);
            return Ok(vec![self.sink.failure("open folder", reason)]);
        };
        self.wanted.folder = Some(folder);
        let page = self.read_first_page(session, folder, &path).await;
        if matches!(&page, Err(error) if !error.is_lost()) {
            self.wanted.folder = None;
        }
        page
    }

    /// Opens the mailbox and reads its newest page. If the server refuses
    /// the range (messages were removed in between), it looks once more.
    pub(super) async fn read_first_page(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        path: &str,
    ) -> Result<Vec<Event>, SessionError> {
        let mut info = session.examine(path).await?;
        let fetched = match session.fetch_page(info.exists, 0, PAGE_SIZE).await {
            Err(error) if !error.is_lost() => {
                info = session.examine(path).await?;
                session.fetch_page(info.exists, 0, PAGE_SIZE).await?
            }
            other => other?,
        };
        self.state.note_validity(folder, info.uid_validity);
        // The screen drops a search it is showing when a page arrives.
        self.search_shown = false;
        Ok(vec![Event::EnvelopePage {
            folder,
            offset: 0,
            total: info.exists,
            envelopes: self.envelopes_of(folder, &fetched),
        }])
    }

    /// The page of older messages that follows the `have` newest ones: the
    /// mailbox is opened read-only, so nothing changes on the server. Nothing
    /// older, or a folder that was rebuilt in between, says so instead.
    pub(super) async fn load_more(
        &mut self,
        session: &mut ImapSession,
        folder: FolderId,
        have: u32,
    ) -> Result<Vec<Event>, SessionError> {
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            let reason = format!("unknown folder id: {}", folder.0);
            return Ok(vec![self.sink.failure("load older mail", reason)]);
        };
        let info = session.examine(&path).await?;
        if self.state.note_validity(folder, info.uid_validity) {
            let reason = "the folder was rebuilt on the server; refresh it";
            return Ok(vec![self.sink.failure("load older mail", reason)]);
        }
        let fetched = session.fetch_page(info.exists, have, PAGE_SIZE).await?;
        if fetched.is_empty() {
            return Ok(Vec::new());
        }
        Ok(vec![Event::OlderEnvelopes {
            folder,
            offset: have,
            total: info.exists,
            envelopes: self.envelopes_of(folder, &fetched),
        }])
    }

    /// One message's body, rendered for the reading pane.
    pub(super) async fn fetch_body(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
    ) -> Result<Vec<Event>, SessionError> {
        let Some((folder, uid)) = self.state.message_key(message) else {
            let reason = format!("unknown message id: {}", message.0);
            return Ok(vec![self.sink.failure("fetch body", reason)]);
        };
        let Some(path) = self.state.folder_path(folder).map(str::to_owned) else {
            return Ok(vec![self.sink.failure("fetch body", "the folder is gone")]);
        };
        self.wanted.body = Some(message);
        let body = self.read_body(session, message, folder, &path, uid.0).await;
        if !matches!(&body, Err(error) if error.is_lost()) {
            self.wanted.body = None;
        }
        body
    }

    async fn read_body(
        &mut self,
        session: &mut ImapSession,
        message: MessageId,
        folder: FolderId,
        path: &str,
        uid: u32,
    ) -> Result<Vec<Event>, SessionError> {
        let reopened = session.ensure_examined(path).await?;
        if reopened.is_some_and(|info| self.state.note_validity(folder, info.uid_validity)) {
            let reason = "the folder was rebuilt on the server; refresh it";
            return Ok(vec![self.sink.failure("fetch body", reason)]);
        }
        let Some(raw) = session.fetch_body(uid).await? else {
            let reason = "the message is no longer on the server";
            return Ok(vec![self.sink.failure("fetch body", reason)]);
        };
        Ok(vec![Event::BodyReady {
            message,
            body: render_body(&raw),
        }])
    }

    /// Re-reads the folder tree and counts, and the open folder's page.
    pub(super) async fn sync_now(
        &mut self,
        session: &mut ImapSession,
        account: &AccountId,
    ) -> Result<Vec<Event>, SessionError> {
        if *account != self.account {
            let reason = format!("unknown account: {}", account.0);
            return Ok(vec![self.sink.failure("sync", reason)]);
        }
        let arrived = self.poll.arrived();
        self.sink
            .send(progress(&self.sink, SyncState::Syncing, self.last_sync, None, arrived));
        let mut events = match self.refresh(session).await {
            Ok(events) => events,
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => {
                let failed = progress(&self.sink, SyncState::Failed, self.last_sync, None, arrived);
                return Ok(vec![failed, self.sink.failure("sync", error.text())]);
            }
        };
        if let Some(folder) = self.wanted.folder {
            match self.open_folder(session, folder).await {
                Ok(page) => events.extend(page),
                Err(error) if error.is_lost() => return Err(error),
                Err(error) => events.push(self.sink.failure("open folder", error.text())),
            }
        }
        Ok(events)
    }

    fn envelopes_of(&mut self, folder: FolderId, fetched: &[Fetch]) -> Vec<Envelope> {
        fetched
            .iter()
            .filter_map(fetched_row)
            .map(|row| {
                let id = self.state.message_id(folder, model::Uid(row.uid));
                build_envelope(id, folder, &row)
            })
            .collect()
    }
}
