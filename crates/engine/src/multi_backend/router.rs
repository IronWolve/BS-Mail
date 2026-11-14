//! Sends commands to the members they are about and collects the answers.

use model::{AccountId, Command, Event};

use super::composite::MultiBackend;
use super::decode_command::{Plan, plan, unknown_account_reason};
use super::intake::{Gate, admit_all};
use super::slots::Slot;

/// Why a command for mail of a member that is gone is refused.
const SLOT_GONE: &str = "the account that mail belongs to is no longer there";

fn refusal(operation: &str, reason: String) -> Event {
    Event::OpFailed {
        account: None,
        operation: operation.to_owned(),
        reason,
    }
}

impl MultiBackend {
    pub(super) fn dispatch(&mut self, command: Command) -> Vec<Event> {
        let plan = plan(command, &|account| self.slot_of(account));
        match plan {
            Plan::Forward(items) => self.forward(items),
            Plan::Everyone(command) => self.everyone(&command),
            Plan::Select(account) => self.select(account),
            Plan::Refused { operation, reason } => vec![refusal(operation, reason)],
        }
    }

    fn slot_of(&self, account: &AccountId) -> Option<Slot> {
        self.entries
            .iter()
            .find(|entry| &entry.id == account)
            .map(|entry| entry.slot)
    }

    /// Sends each command to the member of its slot. Commands for slots that
    /// no member has are answered together with one failure.
    fn forward(&mut self, items: Vec<(Slot, Command)>) -> Vec<Event> {
        let mut events = Vec::new();
        let mut gone = false;
        for (slot, command) in items {
            match self.call(slot, command) {
                Some(answer) => events.extend(answer),
                None => gone = true,
            }
        }
        if gone {
            events.push(refusal("command", SLOT_GONE.to_owned()));
        }
        events
    }

    /// One command to one member; `None` when no member has the slot.
    fn call(&mut self, slot: Slot, command: Command) -> Option<Vec<Event>> {
        let entry = self.entries.iter_mut().find(|entry| entry.slot == slot)?;
        let answer = entry.backend.handle(command);
        let mut gate = Gate {
            active: self.active.as_ref(),
            host_sent: &mut self.host_sent,
        };
        Some(admit_all(entry, answer, &mut gate))
    }

    fn everyone(&mut self, command: &Command) -> Vec<Event> {
        let mut events = Vec::new();
        for entry in &mut self.entries {
            let answer = entry.backend.handle(command.clone());
            let mut gate = Gate {
                active: self.active.as_ref(),
                host_sent: &mut self.host_sent,
            };
            events.extend(admit_all(entry, answer, &mut gate));
        }
        events
    }

    /// Makes `account` the active one and returns what the screen needs
    /// about it: the member's remembered state, then whatever the member
    /// itself answers to being selected.
    fn select(&mut self, account: AccountId) -> Vec<Event> {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == account) else {
            return vec![refusal("command", unknown_account_reason(&account))];
        };
        self.active = Some(account.clone());
        let mut events = entry.cache.replay();
        let answer = entry.backend.handle(Command::SelectAccount { account });
        let mut gate = Gate {
            active: self.active.as_ref(),
            host_sent: &mut self.host_sent,
        };
        events.extend(admit_all(entry, answer, &mut gate));
        events
    }
}

#[cfg(test)]
include!("router.tests.rs");
