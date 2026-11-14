//! Answers for commands that cannot be served right now.

use model::Command;

use super::unsupported::{
    OFFLINE_CHANGE_REASON, change_operation, unavailable_operation, unavailable_reason,
};
use super::worker::Worker;

/// Why a reading command could not run.
const OFFLINE_REASON: &str = "not connected to the server";

impl Worker {
    /// Replies with a failure notice. A folder or body that was asked for is
    /// remembered and served again as soon as the connection is back. A
    /// change is never remembered: it is refused, not queued, so nothing is
    /// sent to the server later that the person has forgotten about.
    pub(super) fn answer_offline(&mut self, command: Command) {
        let writes = !self.config.read_only;
        if let Some(operation) = unavailable_operation(&command, writes) {
            self.sink.send(self.sink.failure(operation, unavailable_reason(writes)));
            return;
        }
        if let Some(operation) = change_operation(&command) {
            self.sink.send(self.sink.failure(operation, OFFLINE_CHANGE_REASON));
            return;
        }
        let (operation, remembered) = match command {
            Command::OpenFolder { folder } => {
                self.wanted.folder = Some(folder);
                ("open folder", true)
            }
            Command::FetchBody { message } => {
                self.wanted.body = Some(message);
                ("fetch body", true)
            }
            _ => ("sync", false),
        };
        let reason = if remembered {
            format!("{OFFLINE_REASON}; it will load when the connection is back")
        } else {
            OFFLINE_REASON.to_owned()
        };
        self.sink.send(self.sink.failure(operation, reason));
    }
}
