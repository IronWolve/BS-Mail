//! Work for the Options screen that can wait on the system keychain (looking
//! for one, moving the key into it or out of it), done on a thread of its own
//! so the screen keeps drawing and answering keys while it waits. One piece of
//! work at a time; its answer is a change to the view, made on the screen's own
//! thread the next time the screen looks (`poll`, from the screen's wake-ups).

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use ui::AppView;

/// What a finished piece of work does to the screen.
pub type Apply = Box<dyn FnOnce(&mut AppView) + Send>;

/// Said when the thread ended without an answer (it never does by design).
const NO_ANSWER: &str = "That did not finish. Open this page again to see how the passwords are protected.";

#[derive(Default)]
pub struct Background {
    waiting: Option<Receiver<Apply>>,
}

impl Background {
    /// Runs `work` on a thread of its own. `false`, and `work` dropped, when
    /// one is running already or no thread could be made.
    pub fn start(&mut self, work: impl FnOnce() -> Apply + Send + 'static) -> bool {
        if self.waiting.is_some() {
            return false;
        }
        let (send, receive) = channel();
        let spawned = std::thread::Builder::new().name("options work".to_string()).spawn(move || {
            // The screen may be gone by now (the program is closing): nobody needs the answer.
            let _ = send.send(work());
        });
        if spawned.is_err() {
            return false;
        }
        self.waiting = Some(receive);
        true
    }

    /// Makes the answer of finished work on `view`; whether anything changed.
    pub fn poll(&mut self, view: &mut AppView) -> bool {
        let Some(receive) = &self.waiting else {
            return false;
        };
        match receive.try_recv() {
            Ok(apply) => {
                self.waiting = None;
                apply(view);
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.waiting = None;
                view.options_problem(NO_ANSWER);
                true
            }
        }
    }
}

#[cfg(test)]
include!("background.tests.rs");
