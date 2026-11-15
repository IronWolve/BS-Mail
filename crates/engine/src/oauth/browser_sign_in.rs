//! The sign-in that needs a person with a browser. `start` binds the local
//! listener, builds the link and returns at once; one thread, with a small
//! runtime of its own, waits for the browser's return, trades the code for
//! tokens and writes the token file. The caller shows `url()`, calls `poll()`
//! now and then, and may `cancel()`; dropping the value cancels too, so no
//! thread or port is left behind. The program never opens the browser.

use std::fmt;
use std::future::Future;
use std::net::TcpListener as StdListener;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::watch;

use super::authorize_url::{AuthorizeRequest, build};
use super::http::HttpLimits;
use super::loopback_listener::{
    ListenError, ListenLimits, Redirect, wait_for_redirect, wait_for_redirect_on_both,
};
use super::pkce::{Pkce, random_state};
use super::token_endpoint::exchange_code;
use super::token_source::store_sign_in;
use crate::account_config::AccountConfig;
use crate::account_oauth::OAuthSettings;

/// The sentence for a sign-in that was cancelled.
const CANCELLED: &str = "the sign-in was cancelled";

/// The waits of a browser sign-in. Tests use short ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignInLimits {
    /// The whole wait for the browser to come back.
    pub wait: Duration,
    /// One request to the local listener.
    pub request: Duration,
    /// The calls to the token endpoint.
    pub http: HttpLimits,
}

impl SignInLimits {
    /// Five minutes for the person, five seconds per request.
    pub const fn production() -> Self {
        Self { wait: Duration::from_secs(300), request: Duration::from_secs(5), http: HttpLimits::production() }
    }
}

/// Where a sign-in stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignInPoll {
    Waiting,
    /// The tokens are in the token file.
    Done,
    /// It ended without tokens; the sentence says why.
    Failed(String),
}

/// A sign-in in progress.
pub struct BrowserSignIn {
    url: String,
    cancel: watch::Sender<bool>,
    result: Receiver<Result<(), String>>,
    thread: Option<JoinHandle<()>>,
    last: Option<SignInPoll>,
}

impl fmt::Debug for BrowserSignIn {
    /// The link carries this sign-in's `state`, so it is not printed.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserSignIn").field("finished", &self.last.is_some()).finish_non_exhaustive()
    }
}

/// Everything the sign-in thread owns.
struct Job {
    settings: OAuthSettings,
    listener: StdListener,
    /// The same port on the IPv6 loopback address, for a `localhost` redirect.
    listener6: Option<StdListener>,
    redirect_uri: String,
    state: String,
    pkce: Pkce,
    limits: SignInLimits,
    cancelled: watch::Receiver<bool>,
}

impl BrowserSignIn {
    /// Starts a sign-in with the production waits.
    pub fn start(config: &AccountConfig) -> Result<Self, String> {
        Self::start_with(config, SignInLimits::production())
    }

    /// Starts a sign-in with the waits chosen by the caller.
    pub fn start_with(config: &AccountConfig, limits: SignInLimits) -> Result<Self, String> {
        let settings = config.oauth.clone().ok_or_else(|| {
            "this account does not use OAuth2 sign-in; set auth = \"oauth2\" in the account file".to_owned()
        })?;
        if settings.token_file.as_os_str().is_empty() {
            return Err("this account has no place set for its sign-in tokens (it must be read from a file, \
                        or given a state folder)"
                .to_owned());
        }
        let (listener, listener6) = bind_loopback(&settings.redirect_host)?;
        let port = listener.local_addr().map_err(|error| error.to_string())?.port();
        // No trailing slash: a registered `http://localhost` is matched on the host
        // only, and an added path can make it mismatch. The browser still comes
        // back to `GET /?code=..` because an empty path is sent as `/`.
        let redirect_uri = format!("http://{}:{port}", settings.redirect_host);
        let pkce = Pkce::generate()?;
        let state = random_state()?;
        let url = build(&AuthorizeRequest {
            settings: &settings,
            redirect_uri: &redirect_uri,
            state: &state,
            challenge: pkce.challenge(),
            login_hint: &config.address,
        });
        let (cancel, cancelled) = watch::channel(false);
        let (sender, result) = channel();
        let job = Job { settings, listener, listener6, redirect_uri, state, pkce, limits, cancelled };
        let thread = std::thread::Builder::new()
            .name("oauth-sign-in".to_owned())
            .spawn(move || run(job, &sender))
            .map_err(|error| format!("cannot start the sign-in: {error}"))?;
        Ok(Self { url, cancel, result, thread: Some(thread), last: None })
    }

    /// The link to open in a browser.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Where the sign-in stands; never waits.
    pub fn poll(&mut self) -> SignInPoll {
        if let Some(finished) = &self.last {
            return finished.clone();
        }
        let outcome = match self.result.try_recv() {
            Ok(Ok(())) => SignInPoll::Done,
            Ok(Err(text)) => SignInPoll::Failed(text),
            Err(TryRecvError::Empty) => return SignInPoll::Waiting,
            Err(TryRecvError::Disconnected) => SignInPoll::Failed("the sign-in stopped unexpectedly".to_owned()),
        };
        self.finish(outcome)
    }

    /// Stops a sign-in that is still going: the listener is closed and the
    /// thread has ended when this returns. A sign-in that already finished
    /// keeps its outcome.
    pub fn cancel(&mut self) {
        if self.last.is_some() {
            return;
        }
        let _ = self.cancel.send(true);
        let (outcome, stopped) = match self.result.recv_timeout(JOIN_WAIT) {
            Ok(Ok(())) => (SignInPoll::Done, true),
            Ok(Err(text)) if text != CANCELLED => (SignInPoll::Failed(text), true),
            Ok(Err(_)) | Err(RecvTimeoutError::Disconnected) => (SignInPoll::Failed(CANCELLED.to_owned()), true),
            Err(RecvTimeoutError::Timeout) => (SignInPoll::Failed(CANCELLED.to_owned()), false),
        };
        self.settle(outcome, stopped);
    }

    fn finish(&mut self, outcome: SignInPoll) -> SignInPoll {
        self.settle(outcome, true)
    }

    /// Records the outcome. The thread is joined only when it has said it is
    /// finishing; one that never answered is left to end on its own rather
    /// than hang the caller.
    fn settle(&mut self, outcome: SignInPoll, stopped: bool) -> SignInPoll {
        if let Some(thread) = self.thread.take() {
            if stopped {
                let _ = thread.join();
            }
        }
        self.last = Some(outcome.clone());
        outcome
    }
}

/// How long `cancel` waits for the thread to say it has stopped.
const JOIN_WAIT: Duration = Duration::from_secs(5);

impl Drop for BrowserSignIn {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// The listener for the browser's return: IPv4 loopback on a port the system
/// picks. A redirect to `localhost` is also listened for on the same port at
/// the IPv6 loopback address, because the name may lead there first and a
/// browser need not fall back to IPv4. That second socket is best effort: a
/// machine with no IPv6 loopback (or the port taken there) is served on IPv4
/// alone, as it was before. The IP literal `127.0.0.1` needs only IPv4.
fn bind_loopback(redirect_host: &str) -> Result<(StdListener, Option<StdListener>), String> {
    let quiet = |l: StdListener| l.set_nonblocking(true).map(|()| l);
    let listener = StdListener::bind(("127.0.0.1", 0))
        .and_then(quiet)
        .map_err(|error| format!("cannot listen on this machine for the browser's return: {error}"))?;
    let port = listener.local_addr().map_err(|error| error.to_string())?.port();
    let second = (redirect_host == "localhost").then(|| StdListener::bind(("::1", port)).and_then(quiet).ok()).flatten();
    Ok((listener, second))
}

/// The thread's body: a runtime of its own, the whole flow, one answer.
fn run(job: Job, answer: &Sender<Result<(), String>>) {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    let outcome = match &runtime {
        Ok(runtime) => runtime.block_on(flow(job)),
        Err(error) => Err(format!("cannot start the sign-in: {error}")),
    };
    let _ = answer.send(outcome);
    // A name lookup still running must not hold the thread up.
    if let Ok(runtime) = runtime {
        runtime.shutdown_background();
    }
}

async fn flow(job: Job) -> Result<(), String> {
    let Job { settings, listener, listener6, redirect_uri, state, pkce, limits, mut cancelled } = job;
    let failed = |error: std::io::Error| format!("the local listener failed: {error}");
    let listener = TcpListener::from_std(listener).map_err(failed)?;
    let listen = ListenLimits { wait: limits.wait, request: limits.request };
    let heard = match listener6 {
        Some(second) => {
            let second = TcpListener::from_std(second).map_err(failed)?;
            until_cancelled(&mut cancelled, wait_for_redirect_on_both(listener, second, &state, &listen)).await
        }
        None => until_cancelled(&mut cancelled, wait_for_redirect(listener, &state, &listen)).await,
    };
    let code = match heard? {
        Ok(Redirect::Code(code)) => code,
        Ok(Redirect::Denied(words)) => return Err(format!("the provider did not allow the sign-in: {words}")),
        Err(ListenError::NoAnswer) => return Err("no answer from the browser".to_owned()),
        Err(ListenError::Failed(text)) => return Err(text),
    };
    let traded = exchange_code(&settings, &code, pkce.verifier(), &redirect_uri, &limits.http);
    let tokens = until_cancelled(&mut cancelled, traded).await?.map_err(|error| error.to_string())?;
    // The token file may be waiting on the lock of a refresh. Giving up must
    // not leave this thread to write the file after the caller has cleaned up:
    // the write itself is one step after the lock is had, so a cancel either
    // comes first (nothing is written) or finds it done.
    let stored = until_cancelled(&mut cancelled, store_sign_in(&settings, tokens)).await?;
    stored.map_err(|error| error.to_string())
}

/// The work's result, or the cancelled sentence when cancelling comes first.
async fn until_cancelled<T>(
    cancelled: &mut watch::Receiver<bool>,
    work: impl Future<Output = T>,
) -> Result<T, String> {
    tokio::select! {
        biased;
        _ = cancelled.wait_for(|stop| *stop) => Err(CANCELLED.to_owned()),
        done = work => Ok(done),
    }
}

#[cfg(test)]
include!("browser_sign_in.tests.rs");
