//! Changing flags on named messages of the mailbox open for changes, and
//! reading them back when the server's answer did not carry them.

use async_imap::types::Flag;
use futures_util::TryStreamExt;
use model::Flags;

use super::error::SessionError;
use super::flag_map::map_flags;
use super::session::ImapSession;
use super::timeouts::bounded;
use super::uid_set::UidSet;
use super::write_command::{Keyword, StoreFlag, StoreMode, WriteCommand};

impl ImapSession {
    /// Adds or removes `flags` on exactly the messages in `set`. Returns the
    /// flags the server reports for each of them; a message the server did
    /// not report on is missing from the list.
    pub async fn store_flags(
        &mut self,
        set: &UidSet,
        mode: StoreMode,
        flags: &[StoreFlag],
    ) -> Result<Vec<(u32, Flags)>, SessionError> {
        let rows = self
            .send_change(&WriteCommand::Store { set, mode, flags })
            .await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| row.uid.map(|uid| (uid, row.flags)))
            .filter(|(uid, _)| set.contains(*uid))
            .collect())
    }

    /// Adds or removes one tag on exactly the messages in `set`. Returns the
    /// tags the server reports for each of them; a message the server did not
    /// report on is missing from the list.
    pub async fn store_keyword(
        &mut self,
        set: &UidSet,
        mode: StoreMode,
        keyword: &Keyword,
    ) -> Result<Vec<(u32, Vec<String>)>, SessionError> {
        let rows = self.send_change(&WriteCommand::StoreKeyword { set, mode, keyword }).await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| row.uid.map(|uid| (uid, row.tags)))
            .filter(|(uid, _)| set.contains(*uid))
            .collect())
    }

    /// The tags of exactly the messages in `set`, as the server holds them
    /// now. A read: it changes nothing.
    pub async fn read_tags(&mut self, set: &UidSet) -> Result<Vec<(u32, Vec<String>)>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session.uid_fetch(set.to_string(), "(UID FLAGS)").await?.try_collect::<Vec<_>>().await
        })
        .await;
        let rows = self.settle(result)?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let flags: Vec<Flag<'_>> = row.flags().collect();
                row.uid.map(|uid| (uid, map_flags(&flags).1))
            })
            .filter(|(uid, _)| set.contains(*uid))
            .collect())
    }

    /// The flags of exactly the messages in `set`, as the server holds them
    /// now. A read: it changes nothing.
    pub async fn read_flags(&mut self, set: &UidSet) -> Result<Vec<(u32, Flags)>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            session
                .uid_fetch(set.to_string(), "(UID FLAGS)")
                .await?
                .try_collect::<Vec<_>>()
                .await
        })
        .await;
        let rows = self.settle(result)?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let flags: Vec<Flag<'_>> = row.flags().collect();
                row.uid.map(|uid| (uid, map_flags(&flags).0))
            })
            .filter(|(uid, _)| set.contains(*uid))
            .collect())
    }
}

#[cfg(test)]
include!("store_flags.tests.rs");
