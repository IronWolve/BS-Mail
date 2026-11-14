//! The browser step of signing in, as the screen sees it: a link to show and
//! an answer that comes later. The real one is the engine's; the tests of the
//! flow use a stand-in that never touches a network.

use engine::{AccountConfig, BrowserSignIn, SignInPoll};

/// Where the browser step stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserPoll {
    /// The user has not finished yet.
    Waiting,
    /// The sign-in worked and the tokens are saved.
    Done,
    /// It ended without a sign-in; the reason is for the user.
    Failed(String),
}

/// One browser sign-in in progress. Dropping it gives up: the program stops
/// listening for the browser.
pub trait BrowserLogin {
    /// The link the user opens.
    fn url(&self) -> &str;

    fn poll(&mut self) -> BrowserPoll;
}

/// Starts browser sign-ins.
pub trait BrowserLoginStarter {
    fn start(&self, config: &AccountConfig) -> Result<Box<dyn BrowserLogin>, String>;
}

/// The engine's browser sign-in.
pub struct EngineLogin;

impl BrowserLoginStarter for EngineLogin {
    fn start(&self, config: &AccountConfig) -> Result<Box<dyn BrowserLogin>, String> {
        Ok(Box::new(EngineSession(BrowserSignIn::start(config)?)))
    }
}

struct EngineSession(BrowserSignIn);

impl BrowserLogin for EngineSession {
    fn url(&self) -> &str {
        self.0.url()
    }

    fn poll(&mut self) -> BrowserPoll {
        match self.0.poll() {
            SignInPoll::Waiting => BrowserPoll::Waiting,
            SignInPoll::Done => BrowserPoll::Done,
            SignInPoll::Failed(reason) => BrowserPoll::Failed(reason),
        }
    }
}
