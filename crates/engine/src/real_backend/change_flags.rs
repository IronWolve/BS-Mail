//! Setting and clearing flags on messages.

use std::collections::HashMap;

use model::{FlagChange, Flags, MessageId};

use super::change_rules::{FlagPlan, plan_flags};
use super::changes::Tally;
use super::resolve::{Group, resolve};
use super::worker::Worker;
use crate::change_refusals::DELETED_NOT_HERE;
use crate::imap_session::{ImapSession, SessionError, UidSet};

impl Worker {
    /// Sends the flag change for every folder the messages are in.
    pub(super) async fn set_flags(
        &mut self,
        session: &mut ImapSession,
        messages: &[MessageId],
        change: FlagChange,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let plan = plan_flags(change);
        if plan.deleted_ignored {
            tally.notice(DELETED_NOT_HERE);
        }
        if plan.is_empty() {
            return Ok(());
        }
        let resolved = resolve(&self.state, messages);
        tally.unknown(&resolved.unknown);
        for group in &resolved.groups {
            if let Err(error) = self.open_for_change(session, group.folder, &group.path).await {
                tally.open_failed(error)?;
                continue;
            }
            self.store_group(session, group, &plan, tally).await?;
        }
        Ok(())
    }

    /// Changes the flags of one folder's messages, a command's worth at a
    /// time, and notes what the server reports.
    async fn store_group(
        &mut self,
        session: &mut ImapSession,
        group: &Group,
        plan: &FlagPlan,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        for (ids, set) in group.batches() {
            let mut known: HashMap<u32, Flags> = HashMap::new();
            let mut failed = false;
            for (mode, flags) in plan.steps() {
                match session.store_flags(&set, mode, flags).await {
                    Ok(rows) => known.extend(rows),
                    Err(error) if error.is_lost() => return Err(error),
                    Err(error) => {
                        tally.notice(error.text());
                        failed = true;
                        break;
                    }
                }
            }
            if !failed {
                self.read_back(session, &set, &mut known, tally).await?;
            }
            let uids: HashMap<MessageId, u32> = group
                .items
                .iter()
                .map(|(id, uid)| (*id, uid.0))
                .collect();
            let (mut unmet, mut gone) = (0, 0);
            for id in ids {
                match uids.get(&id).and_then(|uid| known.get(uid)) {
                    Some(flags) => {
                        tally.updates.push((id, *flags));
                        unmet += usize::from(!failed && !plan.satisfied_by(*flags));
                    }
                    None => gone += usize::from(!failed),
                }
            }
            if !known.is_empty() {
                tally.touch(group.folder);
            }
            if unmet > 0 {
                tally.notice(format!("the server did not keep the change on {unmet} message(s)"));
            }
            if gone > 0 {
                tally.notice(format!("{gone} message(s) are no longer on the server"));
            }
        }
        Ok(())
    }

    /// Asks for the flags of messages the STORE answer said nothing about.
    async fn read_back(
        &mut self,
        session: &mut ImapSession,
        set: &UidSet,
        known: &mut HashMap<u32, Flags>,
        tally: &mut Tally,
    ) -> Result<(), SessionError> {
        let silent = UidSet::new(set.uids().iter().copied().filter(|uid| !known.contains_key(uid)));
        let Some(silent) = silent else {
            return Ok(());
        };
        match session.read_flags(&silent).await {
            Ok(rows) => known.extend(rows),
            Err(error) if error.is_lost() => return Err(error),
            Err(error) => tally.notice(error.text()),
        }
        Ok(())
    }
}
