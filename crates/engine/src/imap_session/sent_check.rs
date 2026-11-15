//! Looking in the Sent folder for a message by its Message-ID: before a copy
//! that may already have been saved (the connection was lost while the server
//! was answering the APPEND) is saved again, so the folder does not end up
//! with two. It only reads: the folder is opened with EXAMINE and searched.

use super::error::SessionError;
use super::session::ImapSession;
use super::write_command::SentFolder;
use crate::search::Keys;

impl ImapSession {
    /// Whether the Sent folder holds a message whose Message-ID is
    /// `message_id` (without its angle brackets). `Ok(false)` also when the
    /// id cannot be searched for, since nothing can then be concluded and the
    /// copy is saved as before.
    pub async fn sent_holds(&mut self, target: &SentFolder, message_id: &str) -> Result<bool, SessionError> {
        let Some(keys) = Keys::header_holds("Message-ID", &format!("<{message_id}>")) else {
            return Ok(false);
        };
        self.examine(target.path()).await?;
        Ok(!self.uid_search(&keys).await?.is_empty())
    }
}
