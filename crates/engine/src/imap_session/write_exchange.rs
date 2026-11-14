//! The one place a changing command is sent and its answer read.
//!
//! Before anything is written the session checks that the account allows
//! changes, that a mailbox is open for changes, and that the server offers
//! the extension the command needs. The answer is read to the command's own
//! tagged line and its status is checked: a refusal is an error, never an
//! empty success. Flags the server reports back for a STORE are collected.

use std::borrow::Cow;

use async_imap::error::Error as ImapError;
use async_imap::imap_proto::{AttributeValue, Response, Status};
use async_imap::types::Flag;
use model::Flags;

use super::error::SessionError;
use super::flag_map::map_flags;
use super::session::ImapSession;
use super::timeouts::bounded;
use super::write_command::{Needs, WriteCommand};

/// One `* n FETCH` line that came back with a STORE answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FlagRow {
    /// Absent when the server left the UID out; such a row is not used.
    pub uid: Option<u32>,
    pub flags: Flags,
}

impl ImapSession {
    /// Refuses unless the server offers what `needs` asks for.
    pub(super) fn require_extension(&self, needs: Needs) -> Result<(), SessionError> {
        let (offered, name) = match needs {
            Needs::Nothing => return Ok(()),
            Needs::Move => (self.caps.move_ext, "MOVE"),
            Needs::UidPlus => (self.caps.uidplus, "UIDPLUS"),
        };
        if offered {
            Ok(())
        } else {
            Err(SessionError::Refused(format!("the server does not offer {name}")))
        }
    }

    /// Sends one changing command and returns the flag rows of its answer.
    pub(super) async fn send_change(
        &mut self,
        command: &WriteCommand<'_>,
    ) -> Result<Vec<FlagRow>, SessionError> {
        self.require_writes()?;
        self.require_writable_mailbox()?;
        self.require_extension(command.needs())?;
        let line = command.line()?;
        self.exchange(&line).await
    }

    async fn exchange(&mut self, line: &str) -> Result<Vec<FlagRow>, SessionError> {
        let limit = self.limits.command;
        let session = &mut self.inner;
        let result = bounded(limit, async {
            let tag = session.run_command(line).await?;
            let mut rows = Vec::new();
            loop {
                let next = session.read_response().await.map_err(ImapError::Io)?;
                let Some(response) = next else {
                    return Err(ImapError::ConnectionLost);
                };
                match response.parsed() {
                    Response::Done { tag: done, status, outcome } if *done == tag => {
                        return outcome_of(status, outcome.information.as_deref()).map(|()| rows);
                    }
                    Response::Fetch(_, attributes) => rows.push(flag_row(attributes)),
                    Response::Data { status: Status::Bye, .. } => {
                        return Err(ImapError::ConnectionLost);
                    }
                    _ => {}
                }
            }
        })
        .await;
        self.settle(result)
    }
}

/// The tagged status as a result: only OK is success.
fn outcome_of(status: &Status, text: Option<&str>) -> Result<(), ImapError> {
    let text = text.unwrap_or("no reason given").to_owned();
    match status {
        Status::Ok => Ok(()),
        Status::No => Err(ImapError::No(text)),
        Status::Bad => Err(ImapError::Bad(text)),
        _ => Err(ImapError::ConnectionLost),
    }
}

fn flag_row(attributes: &[AttributeValue<'_>]) -> FlagRow {
    let mut uid = None;
    let mut names: Vec<&str> = Vec::new();
    for attribute in attributes {
        match attribute {
            AttributeValue::Uid(found) => uid = Some(*found),
            AttributeValue::Flags(list) => names = list.iter().map(Cow::as_ref).collect(),
            _ => {}
        }
    }
    let flags: Vec<Flag<'_>> = names.into_iter().map(Flag::from).collect();
    FlagRow { uid, flags: map_flags(&flags).0 }
}

#[cfg(test)]
include!("write_exchange.tests.rs");
