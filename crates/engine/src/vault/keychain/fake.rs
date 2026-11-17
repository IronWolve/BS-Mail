//! A keychain in memory, for tests: nothing here touches the real keychain,
//! the session bus or the screen, so no test can make a dialog appear on a
//! desktop. Clones share one state, so a test keeps one handle to look at and
//! to break things with while the vault holds another.
//!
//! Faults are made on purpose: every call can be made to fail with a chosen
//! failure (all of them, or one kind), a read can be made to give another key
//! back, a call can be made to hang for a while, and the keychain can be made
//! unavailable as a whole. Every call is recorded.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use super::failure::KeychainFailure;
use super::key_id::PROBE_ACCOUNT;
use super::store::KeychainStore;
use super::stored_key::StoredKey;

/// The kinds of call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Get,
    Set,
    Delete,
    Probe,
}

#[derive(Default)]
struct State {
    entries: HashMap<String, [u8; 32]>,
    /// Every call fails with this, whatever it is.
    broken: Option<KeychainFailure>,
    /// Calls of one kind fail with this.
    failing: HashMap<Op, KeychainFailure>,
    /// A read gives back these bytes in place of what was stored.
    wrong_reads: Option<[u8; 32]>,
    /// Every call waits this long first.
    hang: Option<Duration>,
    calls: Vec<(Op, String)>,
}

#[derive(Clone, Default)]
pub struct FakeKeychain {
    state: Arc<Mutex<State>>,
}

impl FakeKeychain {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Makes every call fail with `failure` until `repair`.
    pub fn break_with(&self, failure: KeychainFailure) {
        self.lock().broken = Some(failure);
    }

    /// No keychain here at all: every call says `Unavailable`.
    pub fn make_unavailable(&self) {
        self.break_with(KeychainFailure::Unavailable);
    }

    /// Makes calls of one kind fail with `failure` until `repair`.
    pub fn fail(&self, op: Op, failure: KeychainFailure) {
        self.lock().failing.insert(op, failure);
    }

    /// A read gives `bytes` back, whatever was stored, until `repair`.
    pub fn give_back(&self, bytes: [u8; 32]) {
        self.lock().wrong_reads = Some(bytes);
    }

    /// Every call waits `time` before it does anything, until `repair`.
    pub fn hang_for(&self, time: Duration) {
        self.lock().hang = Some(time);
    }

    /// Takes every fault away; the entries stay.
    pub fn repair(&self) {
        let mut state = self.lock();
        state.broken = None;
        state.failing.clear();
        state.wrong_reads = None;
        state.hang = None;
    }

    /// Puts an entry in by hand, as if another run had made it.
    pub fn put(&self, account: &str, bytes: [u8; 32]) {
        self.lock().entries.insert(account.to_string(), bytes);
    }

    /// Removes an entry by hand, as if the person had deleted it.
    pub fn remove(&self, account: &str) {
        self.lock().entries.remove(account);
    }

    /// The bytes kept under `account`, for a test to look for them elsewhere.
    pub fn bytes_of(&self, account: &str) -> Option<[u8; 32]> {
        self.lock().entries.get(account).copied()
    }

    /// The names of the entries held, sorted.
    pub fn accounts(&self) -> Vec<String> {
        let mut names: Vec<String> = self.lock().entries.keys().cloned().collect();
        names.sort();
        names
    }

    /// Every call made so far, in order, with the entry it was about.
    pub fn calls(&self) -> Vec<(Op, String)> {
        self.lock().calls.clone()
    }

    /// How many calls of one kind were made.
    pub fn count(&self, op: Op) -> usize {
        self.lock().calls.iter().filter(|(made, _)| *made == op).count()
    }

    /// Records the call, waits if it is made to, and says how it is made to
    /// fail (`None`: go on).
    fn enter(&self, op: Op, account: &str) -> Option<KeychainFailure> {
        let (hang, failure) = {
            let mut state = self.lock();
            state.calls.push((op, account.to_string()));
            (state.hang, state.broken.or_else(|| state.failing.get(&op).copied()))
        };
        if let Some(time) = hang {
            std::thread::sleep(time);
        }
        failure
    }
}

impl KeychainStore for FakeKeychain {
    fn get(&self, account: &str) -> Result<StoredKey, KeychainFailure> {
        if let Some(failure) = self.enter(Op::Get, account) {
            return Err(failure);
        }
        let state = self.lock();
        let stored = state.entries.get(account).ok_or(KeychainFailure::NotFound)?;
        Ok(StoredKey::new(state.wrong_reads.unwrap_or(*stored)))
    }

    fn set(&self, account: &str, key: &StoredKey) -> Result<(), KeychainFailure> {
        if let Some(failure) = self.enter(Op::Set, account) {
            return Err(failure);
        }
        self.lock().entries.insert(account.to_string(), *key.bytes());
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<(), KeychainFailure> {
        if let Some(failure) = self.enter(Op::Delete, account) {
            return Err(failure);
        }
        self.lock().entries.remove(account);
        Ok(())
    }

    fn probe(&self) -> Result<(), KeychainFailure> {
        self.enter(Op::Probe, PROBE_ACCOUNT).map_or(Ok(()), Err)
    }
}

#[cfg(test)]
include!("fake.tests.rs");
