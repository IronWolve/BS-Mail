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
use std::path::{Path, PathBuf};
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

    /// Starts opening the page file at `path` in the browser; the result comes
    /// later through `poll`. Only a page this program wrote itself is opened.
    fn open_file(&mut self, path: &Path);

    /// Starts opening the saved attachment at `path` with the program the
    /// system has for its kind; the result comes later through `poll`. Only a
    /// file this program saved in its attachments folder is opened, and never
    /// a program, a script or a page that runs code.
    fn open_attachment(&mut self, path: &Path);

    /// The result of the last `open`, once, when there is one.
    fn poll(&mut self) -> Option<Result<(), String>>;
}

/// Opens nothing: for a screen that must never start a browser by itself, as
/// every test's is.
pub struct NoOpener;

impl LinkOpener for NoOpener {
    fn open(&mut self, _url: &str) {}

    fn open_file(&mut self, _path: &Path) {}

    fn open_attachment(&mut self, _path: &Path) {}

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

/// The opener of a run: the system's, which may also open the pages kept in
/// `html/` of the data folder `data` and the attachments saved in its
/// `attachments/`, and no other file.
pub fn opener_for(data: Option<&Path>) -> Box<dyn LinkOpener> {
    let opener = SystemOpener::new();
    Box::new(match data {
        Some(dir) => opener
            .with_pages_in(dir.join(crate::html_store::HTML_DIR_NAME))
            .with_attachments_in(dir.join(crate::attachment_store::ATTACHMENTS_DIR_NAME)),
        None => opener,
    })
}

/// Opens links with whatever the system has.
pub struct SystemOpener {
    launchers: Vec<Launcher>,
    settle: Duration,
    waiting: Option<Receiver<Result<(), String>>>,
    /// The only folder a page file is opened from: `html/` in the data folder.
    pages_dir: Option<PathBuf>,
    /// The only folder an attachment is opened from: `attachments/` in the
    /// data folder.
    attachments_dir: Option<PathBuf>,
}

impl SystemOpener {
    pub fn new() -> Self {
        Self::with(launchers(), SETTLE)
    }

    /// An opener with its own launchers and patience, for tests.
    pub fn with(launchers: Vec<Launcher>, settle: Duration) -> Self {
        Self { launchers, settle, waiting: None, pages_dir: None, attachments_dir: None }
    }

    /// Lets the opener open the page files that are in `dir`, and no others.
    pub fn with_pages_in(mut self, dir: PathBuf) -> Self {
        self.pages_dir = Some(dir);
        self
    }

    /// Lets the opener open the saved attachments that are in `dir`, and no
    /// others.
    pub fn with_attachments_in(mut self, dir: PathBuf) -> Self {
        self.attachments_dir = Some(dir);
        self
    }

    /// Starts the launchers with `file` (already checked), or answers with the
    /// refusal.
    fn start_file(&mut self, checked: Result<String, String>) {
        let (sender, receiver) = channel();
        self.waiting = Some(receiver);
        let file = match checked {
            Ok(file) => file,
            Err(reason) => {
                let _ = sender.send(Err(reason));
                return;
            }
        };
        let launchers = self.launchers.clone();
        let settle = self.settle;
        thread::spawn(move || {
            let _ = sender.send(try_each(&launchers, &file, settle));
        });
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

    fn open_file(&mut self, path: &Path) {
        let checked = checked_page(self.pages_dir.as_deref(), path);
        self.start_file(checked);
    }

    fn open_attachment(&mut self, path: &Path) {
        let checked = checked_attachment(self.attachments_dir.as_deref(), path);
        self.start_file(checked);
    }

    fn poll(&mut self) -> Option<Result<(), String>> {
        let answer = self.waiting.as_ref()?.try_recv().ok()?;
        self.waiting = None;
        Some(answer)
    }
}

/// The path to hand to a launcher for a page file, or why it is refused. It
/// must be a file named as this program names its pages, found (after links
/// are followed) inside the pages folder and nowhere else, and its path must
/// be plain text a launcher cannot take for anything but a path.
fn checked_page(pages_dir: Option<&Path>, path: &Path) -> Result<String, String> {
    const REFUSED: &str = "that file is not a page this program made, so it was not opened";
    let dir = pages_dir.ok_or("there is no folder for pages here, so nothing was opened")?;
    let dir = dir.canonicalize().map_err(|_| REFUSED.to_string())?;
    let file = path.canonicalize().map_err(|_| REFUSED.to_string())?;
    let named_right = file.file_name().and_then(OsStr::to_str).and_then(crate::html_store::page_number).is_some();
    if file.parent() != Some(dir.as_path()) || !named_right || !file.is_file() {
        return Err(REFUSED.to_string());
    }
    let text = launchable(&file).ok_or_else(|| REFUSED.to_string())?;
    if text.starts_with('-') || text.chars().any(|ch| ch.is_control()) {
        return Err(REFUSED.to_string());
    }
    Ok(text)
}

/// The path to hand to a launcher for a saved attachment, or why it is
/// refused. It must be a file in the attachments folder and nowhere else
/// (after links are followed), named in the safe form this program saves
/// names in, and not a program, a script or a page that runs code. Its path
/// is handed over as one argument that no launcher can read as a switch: it
/// starts with a drive letter or a `/` that names a folder, never with a dash,
/// and on Windows never with a slash.
fn checked_attachment(dir: Option<&Path>, path: &Path) -> Result<String, String> {
    const REFUSED: &str = "that file is not an attachment this program saved, so it was not opened";
    let dir = dir.ok_or("there is no folder for attachments here, so nothing was opened")?;
    let dir = dir.canonicalize().map_err(|_| REFUSED.to_string())?;
    let file = path.canonicalize().map_err(|_| REFUSED.to_string())?;
    let name = file.file_name().and_then(OsStr::to_str).ok_or_else(|| REFUSED.to_string())?;
    if file.parent() != Some(dir.as_path()) || !file.is_file() || crate::attachment_name::safe_name(name, "") != name {
        return Err(REFUSED.to_string());
    }
    if crate::attachment_name::is_never_opened(name, "") {
        return Err(format!("{name} is a program, a script or a page that runs code, so it was saved and not opened"));
    }
    let text = launchable(&file).ok_or_else(|| REFUSED.to_string())?;
    let switch_like = text.starts_with('-') || (cfg!(windows) && text.starts_with('/'));
    if switch_like || text.chars().any(|ch| ch.is_control()) {
        return Err(REFUSED.to_string());
    }
    Ok(text)
}

/// The path as text; on Windows without the `\\?\` that canonical paths carry,
/// which the system's openers do not read.
fn launchable(file: &Path) -> Option<String> {
    let text = file.to_str()?;
    Some(text.strip_prefix(r"\\?\").filter(|rest| !rest.starts_with("UNC")).unwrap_or(text).to_string())
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
