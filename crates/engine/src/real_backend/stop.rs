//! The signal that tells the worker to finish, and a way to cut any wait
//! short when it arrives.

use std::future::Future;

use tokio::sync::watch;

/// A cloneable view of the stop flag.
#[derive(Debug, Clone)]
pub struct Stop(watch::Receiver<bool>);

impl Stop {
    /// The flag's two ends: set it to `true` to stop the worker.
    pub fn pair() -> (watch::Sender<bool>, Stop) {
        let (sender, receiver) = watch::channel(false);
        (sender, Stop(receiver))
    }

    /// Resolves when stopping was asked for, or when the owner is gone.
    pub async fn requested(&self) {
        let mut watcher = self.0.clone();
        let _ = watcher.wait_for(|stopped| *stopped).await;
    }

    /// Runs `work` unless stopping is asked for first; `None` means stop.
    pub async fn guard<F: Future>(&self, work: F) -> Option<F::Output> {
        tokio::select! {
            biased;
            () = self.requested() => None,
            out = work => Some(out),
        }
    }
}

#[cfg(test)]
include!("stop.tests.rs");
