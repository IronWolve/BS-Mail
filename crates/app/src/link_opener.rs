//! Hands a link to the user's web browser: the sign-in page does it by itself
//! when a sign-in starts, and again when the user presses Open. The tests use a
//! stand-in that opens nothing, and a test that runs the built program sets
//! `BROWSER` to a command of its own.
//!
//! `BROWSER` is the usual way to choose a browser: a command, or several
//! separated like a `PATH` is, with the link added after its arguments (or put
//! where a `%s` stands). It is split on spaces and never run through a shell.
//!
//! The work happens on its own short thread so the screen never waits for a
//! browser to start; the answer is picked up with `poll`.

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

/// How long a launcher is given to say it failed before it is taken to have
/// worked (a browser that starts stays running).
const SETTLE: Duration = Duration::from_millis(1500);

const NOTHING_FOUND: &str = "no way to open a browser was found here; copy the link and open it yourself";

/// Something that can open a link for the user.
pub trait LinkOpener {
    /// Starts opening `url`; the result comes later through `poll`.
    fn open(&mut self, url: &str);

    /// The result of the last `open`, once, when there is one.
    fn poll(&mut self) -> Option<Result<(), String>>;
}

/// Opens nothing: for a screen that must never start a browser by itself, as
/// every test's is.
pub struct NoOpener;

impl LinkOpener for NoOpener {
    fn open(&mut self, _url: &str) {}

    fn poll(&mut self) -> Option<Result<(), String>> {
        None
    }
}

/// One way of asking the system to open a link.
#[derive(Debug, Clone)]
pub struct Launcher {
    pub program: String,
    /// Arguments before the link. One with `%s` in it has the link put there
    /// instead, and then the link is not added after.
    pub args: Vec<String>,
    /// Whether a quick non-zero exit means it failed. Some launchers exit
    /// non-zero after working.
    pub exit_code_counts: bool,
}

impl Launcher {
    fn new(program: &str, args: &[&str]) -> Self {
        Self { program: program.to_string(), args: args.iter().map(|arg| (*arg).to_string()).collect(), exit_code_counts: true }
    }

    /// The command line of a launcher with the link in it.
    fn arguments(&self, url: &str) -> Vec<String> {
        let placed = self.args.iter().any(|arg| arg.contains("%s"));
        let mut all: Vec<String> = self.args.iter().map(|arg| arg.replace("%s", url)).collect();
        if !placed {
            all.push(url.to_string());
        }
        all
    }
}

/// The launchers a `BROWSER` setting names: each command split on spaces.
pub fn launchers_from_setting(setting: &OsStr) -> Vec<Launcher> {
    std::env::split_paths(setting)
        .filter_map(|command| {
            let text = command.to_string_lossy().into_owned();
            let mut words = text.split_whitespace();
            let program = words.next()?.to_string();
            let args: Vec<String> = words.map(str::to_string).collect();
            Some(Launcher { program, args, exit_code_counts: true })
        })
        .collect()
}

/// The launchers to try, in order: what `BROWSER` says, then what this
/// system has.
pub fn launchers() -> Vec<Launcher> {
    let mut all = std::env::var_os("BROWSER").map(|setting| launchers_from_setting(&setting)).unwrap_or_default();
    all.extend(system_launchers());
    all
}

fn system_launchers() -> Vec<Launcher> {
    if cfg!(target_os = "macos") {
        vec![Launcher::new("open", &[])]
    } else if cfg!(target_os = "windows") {
        vec![Launcher { exit_code_counts: false, ..Launcher::new("explorer.exe", &[]) }]
    } else {
        vec![
            Launcher::new("xdg-open", &[]),
            Launcher::new("wslview", &[]),
            Launcher::new("sensible-browser", &[]),
            Launcher { exit_code_counts: false, ..Launcher::new("explorer.exe", &[]) },
        ]
    }
}

/// Opens links with whatever the system has.
pub struct SystemOpener {
    launchers: Vec<Launcher>,
    settle: Duration,
    waiting: Option<Receiver<Result<(), String>>>,
}

impl SystemOpener {
    pub fn new() -> Self {
        Self::with(launchers(), SETTLE)
    }

    /// An opener with its own launchers and patience, for tests.
    pub fn with(launchers: Vec<Launcher>, settle: Duration) -> Self {
        Self { launchers, settle, waiting: None }
    }
}

impl LinkOpener for SystemOpener {
    fn open(&mut self, url: &str) {
        let (sender, receiver) = channel();
        self.waiting = Some(receiver);
        if !is_plain_web_link(url) {
            let _ = sender.send(Err("the link is not a web address, so it was not opened".to_string()));
            return;
        }
        let url = url.to_string();
        let launchers = self.launchers.clone();
        let settle = self.settle;
        thread::spawn(move || {
            let _ = sender.send(try_each(&launchers, &url, settle));
        });
    }

    fn poll(&mut self) -> Option<Result<(), String>> {
        let answer = self.waiting.as_ref()?.try_recv().ok()?;
        self.waiting = None;
        Some(answer)
    }
}

/// An http or https address with nothing in it that a launcher could read as
/// anything else.
pub fn is_plain_web_link(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && !url.chars().any(|ch| ch.is_whitespace() || ch.is_control())
}

fn try_each(launchers: &[Launcher], url: &str, settle: Duration) -> Result<(), String> {
    for launcher in launchers {
        match run(launcher, url, settle) {
            Ok(true) => return Ok(()),
            Ok(false) => continue,
            Err(reason) => return Err(reason),
        }
    }
    Err(NOTHING_FOUND.to_string())
}

/// Starts one launcher. `Ok(true)` it worked, `Ok(false)` try the next one
/// (it is not here, or it failed straight away), `Err` it cannot be used.
fn run(launcher: &Launcher, url: &str, settle: Duration) -> Result<bool, String> {
    let spawned = Command::new(&launcher.program)
        .args(launcher.arguments(url))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("the browser could not be started: {error}")),
    };
    let deadline = Instant::now() + settle;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status.success() || !launcher.exit_code_counts),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            // Still running: a browser that is starting. Let it be, and see
            // that the finished process is collected.
            Ok(None) => {
                thread::spawn(move || {
                    let _ = child.wait();
                });
                return Ok(true);
            }
            Err(error) => return Err(format!("the browser could not be started: {error}")),
        }
    }
}

#[cfg(test)]
include!("link_opener.tests.rs");
