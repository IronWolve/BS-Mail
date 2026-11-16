//! Setting and clearing a tag, which is an IMAP keyword, on messages.
//!
//! The tag is checked first (see `check_tag`): only a plain word goes to the
//! server, alone in its command. What the server reports for each message is
//! sent back as the message's tags; a message it reports nothing for is read
//! again, and a server that did not keep the change says so.

use std::collections::HashMap;

use model::MessageId;

use super::changes::Tally;
use super::resolve::{Group, resolve};
use super::worker::Worker;
use crate::imap_session::{ImapSession, Keyword, SessionError, StoreMode, UidSet};

impl Worker {
    /// Adds or removes `tag` on the messages, in every folder they are in.
    pub(super) async fn set_tag(
        &mut self,
        session: &mut ImapSession,
        messages: &[MessageId],
        (tag, on): (&str, bool),
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let keyword = match Keyword::new(tag) {
            Ok(keyword) => keyword,
            Err(refused) => {
                tally.notice(refused.text());
                return Ok(());
            }
        };
        let resolved = resolve(&self.state, messages);
        tally.unknown(&resolved.unknown);
        for group in &resolved.groups {
            if let Err(error) = self.open_for_change(session, group.folder, &group.path).await {
                tally.open_failed(error)?;
                continue;
            }
            self.tag_group(session, group, (&keyword, on), tally).await?;
        }
        Ok(())
    }

    /// One folder's messages, a command's worth at a time.
    async fn tag_group(
        &mut self,
        session: &mut ImapSession,
        group: &Group,
        (keyword, on): (&Keyword, bool),
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let mode = if on { StoreMode::Add } else { StoreMode::Remove };
        for (ids, set) in group.batches() {
            let mut known: HashMap<u32, Vec<String>> = HashMap::new();
            match session.store_keyword(&set, mode, keyword).await {
                Ok(rows) => known.extend(rows),
                Err(error) if error.is_lost() => return Err(error),
                Err(error) => {
                    tally.notice(error.text());
                    break;
                }
            }
            self.read_back_tags(session, &set, &mut known, tally).await?;
            let uids: HashMap<MessageId, u32> = group.items.iter().map(|(id, uid)| (*id, uid.0)).collect();
            let (mut unmet, mut gone) = (0, 0);
            for id in ids {
                match uids.get(&id).and_then(|uid| known.get(uid)) {
                    Some(tags) => {
                        unmet += usize::from(tags.iter().any(|held| held.eq_ignore_ascii_case(keyword.as_str())) != on);
                        tally.tags.push((id, tags.clone()));
                    }
                    None => gone += 1,
                }
            }
            if unmet > 0 {
                tally.notice(format!("the server did not keep the tag on {unmet} message(s)"));
            }
            if gone > 0 {
                tally.notice(format!("{gone} message(s) are no longer on the server"));
            }
        }
        Ok(())
    }

    /// Asks for the tags of messages the STORE answer said nothing about.
    async fn read_back_tags(
        &mut self,
        session: &mut ImapSession,
        set: &UidSet,
        known: &mut HashMap<u32, Vec<String>>,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let silent = UidSet::new(set.uids().iter().copied().filter(|uid| !known.contains_key(uid)));
        let Some(silent) = silent else {
            return Ok(());
        };
        match session.read_tags(&silent).await {
            Ok(rows) => known.extend(rows),
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => tally.notice(error.text()),
        }
        Ok(())
    }
}

#[cfg(test)]
include!("change_tags.tests.rs");
