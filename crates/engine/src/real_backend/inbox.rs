//! A queue from the tasks that run on their own back to the worker, which
//! owns the mailbox session and so is the only one that can act on what they
//! report: a saved copy to append, a message to mark as answered.

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub(super) struct Inbox<T> {
    sender: UnboundedSender<T>,
    receiver: UnboundedReceiver<T>,
}

impl<T> Inbox<T> {
    pub(super) fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self { sender, receiver }
    }

    pub(super) fn sender(&self) -> UnboundedSender<T> {
        self.sender.clone()
    }

    /// The next item. The queue never ends while the worker lives, because
    /// the worker holds a sender itself.
    pub(super) async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}
