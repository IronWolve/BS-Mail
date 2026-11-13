//! Answers for commands that cannot be served right now.

use model::Command;

use super::unsupported::{READ_ONLY_REASON, unavailable_operation};
use super::worker::Worker;

/// Why a reading command could not run.
const OFFLINE_REASON: &str = "not connected to the server";

impl Worker {
    /// Replies with a failure notice. A folder or body that was asked for is
    /// remembered and served again as soon as the connection is back.
    pub(super) fn answer_offline(&mut self, command: Command) {
        if let Some(operation) = unavailable_operation(&command) {
            self.sink.send(self.sink.failure(operation, READ_ONLY_REASON));
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
