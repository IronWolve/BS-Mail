//! The events that describe the connection itself: who the account is, what
//! state it is in, when it last synced.

use model::{Account, ConnectionState, Event, Quota, SyncState, SyncStatus};

use super::host_info::host_info;
use super::sink::EventSink;
use crate::account_config::AccountConfig;

/// The first events of a run, sent before any connection is tried so the
/// screen can show the account as connecting: state, host, account list and
/// whether changes are allowed.
pub fn announce(config: &AccountConfig, sink: &EventSink) {
    let (host_name, system) = host_info();
    sink.send(connection(sink, ConnectionState::Connecting));
    sink.send(Event::HostInfo { host_name, system });
    sink.send(Event::AccountList(vec![Account {
        id: sink.account().clone(),
        name: config.name.clone(),
        address: config.address.clone(),
        state: ConnectionState::Connecting,
        unread: None,
        selected: true,
    }]));
    sink.send(changes_allowed(config, sink));
}

/// Whether this account lets the program change mail on the server.
pub fn changes_allowed(config: &AccountConfig, sink: &EventSink) -> Event {
    Event::ChangesAllowed {
        account: sink.account().clone(),
        allowed: !config.read_only,
    }
}

pub fn connection(sink: &EventSink, state: ConnectionState) -> Event {
    Event::ConnectionState {
        account: sink.account().clone(),
        state,
    }
}

/// The sync status event for the account. `new_messages` is how many
/// messages have arrived since the program started, as the checks for new
/// mail found them.
pub fn progress(
    sink: &EventSink,
    state: SyncState,
    last_sync: Option<i64>,
    quota: Option<Quota>,
    new_messages: u32,
) -> Event {
    Event::SyncProgress {
        account: sink.account().clone(),
        status: SyncStatus {
            last_sync,
            new_messages,
            state,
            queue_len: 0,
            quota,
        },
    }
}

pub fn quota_event(sink: &EventSink, quota: Option<Quota>) -> Event {
    Event::Quota {
        account: sink.account().clone(),
        quota,
    }
}

/// Seconds since the Unix epoch, or 0 on a clock set before it.
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
}
