//! A steady wake-up for the window's loop.
//!
//! A backend works on its own thread and has no way to call the window, so
//! the window is woken a few times a second to ask it what it has. The
//! thread stops when the ticker is dropped.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use surface::Waker;

/// How often the window asks the backend for news.
const INTERVAL: Duration = Duration::from_millis(200);

pub struct Ticker {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Ticker {
    pub fn start(waker: Waker) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                thread::sleep(INTERVAL);
                waker.wake();
            }
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
