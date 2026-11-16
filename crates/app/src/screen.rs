//! The screen on the other end of a host's loop: it answers events and hands
//! back a frame whenever the host asks for one.
//!
//! Input goes to the view, which moves the selection; the commands that
//! selection needs go to the backend; what the backend says comes back to the
//! view. Only the keys that close the program are read here.

use std::path::PathBuf;
use std::time::Instant;

use model::{MailBackend, MailDate};
use surface::{EventSink, Flow, InputEvent, KeyEvent, LogicalKey, Presenter};
use ui::AppView;

use crate::accounts::Live;
use crate::clock::system_now;
use crate::errors::AppError;
use crate::link_opener::{LinkOpener, NoOpener};
use crate::options_flow::{self, Opened, Settings, is_setting};
use crate::pump::pump;
use crate::setup_flow::SetupFlow;
use crate::state_file::{self, State};
use crate::ticker::Ticker;

mod addresses;
mod links;

/// How many times mail is moved again after the accounts changed, at most.
const ROUNDS: usize = 3;

pub struct Screen {
    view: AppView,
    backend: Box<dyn MailBackend>,
    cols: u16,
    rows: u16,
    frames: u32,
    /// A clock that stands still; `None` follows the system clock.
    fixed_now: Option<MailDate>,
    /// Frames after which the run stops itself, for a check with nobody
    /// watching; `None` runs until the window is closed.
    stop_after: Option<u32>,
    done: bool,
    ticker: Option<Ticker>,
    /// When the screen started, for the steady clock the view's timers use.
    started: Instant,
    /// Where the sizes are remembered, and what was last written there.
    remember: Option<(PathBuf, State)>,
    /// Whether the window's grid is part of what is remembered. A terminal's
    /// grid is the terminal's, so it keeps whatever the window last saved.
    track_grid: bool,
    /// The title last given to the window, so it is set only when it changes.
    title: String,
    /// What answers the account form, while there is one.
    setup: Option<SetupFlow>,
    /// The accounts of this run, which the Options screen and the form can
    /// change; `None` for mock mail.
    live: Option<Live>,
    /// What answers the Options screen's requests about settings.
    settings: Settings,
    /// What opens a web address the user clicked in a message.
    links: Box<dyn LinkOpener>,
    /// Where the address book is kept, when this run keeps one.
    addresses: Option<addresses::AddressSave>,
}

impl Screen {
    pub fn new(
        view: AppView,
        backend: Box<dyn MailBackend>,
        grid: (u16, u16),
        fixed_now: Option<MailDate>,
    ) -> Self {
        Self {
            view,
            backend,
            cols: grid.0,
            rows: grid.1,
            frames: 0,
            fixed_now,
            stop_after: None,
            done: false,
            ticker: None,
            started: Instant::now(),
            remember: None,
            track_grid: true,
            title: String::new(),
            setup: None,
            live: None,
            settings: Settings::default(),
            links: Box::new(NoOpener),
            addresses: None,
        }
    }

    /// Lets a click on a web address in a message open it with `opener`. A
    /// screen that is given none opens nothing.
    pub fn with_link_opener(mut self, opener: Box<dyn LinkOpener>) -> Self {
        self.links = opener;
        self
    }

    /// Lets the Options screen change and reload the settings `settings`
    /// describes.
    pub fn with_settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    /// Lets the Options screen and the account form add, change and remove
    /// the accounts `live` describes.
    pub fn with_accounts(mut self, live: Option<Live>) -> Self {
        self.live = live;
        self
    }

    /// Remembers the sizes of the bars and the window in `dir`.
    pub fn remember_in(mut self, dir: PathBuf) -> Self {
        let current = self.state_now();
        self.remember = Some((dir, current));
        self
    }

    /// Remembers only the bar sizes, starting from what was loaded: for a
    /// terminal, whose grid is not the window's.
    pub fn remember_layout_in(mut self, dir: PathBuf, loaded: State) -> Self {
        self.track_grid = false;
        self.remember = Some((dir, loaded));
        self
    }

    /// What would be remembered now.
    fn state_now(&self) -> State {
        let grid = if self.track_grid {
            Some((self.cols, self.rows))
        } else {
            self.remember.as_ref().and_then(|(_, saved)| saved.grid)
        };
        let saved_place = self.remember.as_ref().map(|(_, saved)| saved.place.clone()).unwrap_or_default();
        State {
            layout: self.view.prefs().clone(),
            grid,
            place: self.view.place().over(&saved_place),
        }
    }

    /// Keeps the account and folder that are open when they are not the ones
    /// kept, so a run that ends without a goodbye still comes back to them.
    fn save_place_when_changed(&mut self) {
        let changed = self.remember.as_ref().is_some_and(|(_, saved)| self.state_now().place != saved.place);
        if changed {
            // A failure waits: closing the program tries again and says so.
            let _ = self.save_state();
        }
    }

    /// Writes what is remembered, when it has changed. Called when a drag
    /// ends, when the account or folder open changes and when the window
    /// closes. A write that failed is an error, and
    /// what was saved before is left as it was; the sizes stay marked as not
    /// saved, so the next call tries again.
    pub fn save_state(&mut self) -> Result<(), AppError> {
        let current = self.state_now();
        let Some((dir, saved)) = &mut self.remember else {
            return Ok(());
        };
        if current == *saved {
            return Ok(());
        }
        state_file::save(dir, &current).map_err(|error| AppError::State(dir.clone(), error))?;
        *saved = current;
        Ok(())
    }

    /// Stops the run once `frames` frames have been drawn.
    pub fn stop_after(mut self, frames: u32) -> Self {
        self.stop_after = Some(frames.max(1));
        self
    }

    /// How many frames were drawn, for the stats block.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    pub fn grid(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Moves mail both ways until quiet; returns whether the view changed.
    /// What the Options screen and the account form asked for is answered
    /// first; when that changed which accounts run, mail is moved again so
    /// the new state is on screen before the next frame.
    fn exchange(&mut self, host: &mut dyn Presenter) -> bool {
        let mut changed = false;
        for _ in 0..ROUNDS {
            let asked = self.account_steps(host);
            let mail = pump(&mut self.view, self.backend.as_mut()) > 0;
            changed |= asked | mail;
            if !asked {
                break;
            }
        }
        self.keep_addresses();
        changed
    }

    /// The Options screen's requests, then the form's.
    fn account_steps(&mut self, host: &mut dyn Presenter) -> bool {
        let actions = self.view.take_options_actions();
        let mut changed = !actions.is_empty();
        if changed {
            self.answer_options(actions, host);
        }
        changed |= self.setup_step();
        changed |= self.link_steps();
        changed
    }

    /// The requests about settings go to the settings, with the host at hand
    /// to show a new text size; the rest are the accounts'.
    fn answer_options(&mut self, actions: Vec<ui::OptionsAction>, host: &mut dyn Presenter) {
        let (about_settings, about_accounts): (Vec<_>, Vec<_>) = actions.into_iter().partition(is_setting);
        if !about_settings.is_empty()
            && let Some((cols, rows)) = self.settings.answer(&mut self.view, host, about_settings)
        {
            self.cols = cols;
            self.rows = rows;
        }
        if !about_accounts.is_empty() {
            self.answer_accounts(about_accounts);
        }
    }

    fn answer_accounts(&mut self, actions: Vec<ui::OptionsAction>) {
        let edit = match options_flow::answer(&mut self.view, &mut self.backend, self.live.as_ref(), actions) {
            Opened::Nothing => return,
            Opened::New => None,
            Opened::Editing(id) => Some(id),
        };
        // A form opened over another one replaces it: the old attempt ends.
        if let Some(mut old) = self.setup.take() {
            old.abandon();
        }
        self.setup = self.live.clone().map(|live| SetupFlow::new(live, edit));
    }

    /// What a key that closes the program does, or `None` when it is only a
    /// letter for a text box. While text is being typed every letter is for
    /// the text. A message that is being sent takes no text and cannot be
    /// called back, so there `q` and `Ctrl+Q` ask to quit the way the
    /// window's close button does: the first press says what quitting costs
    /// and the second goes ahead. A key held down counts as one press.
    fn close_key(&mut self, key: &KeyEvent, host: &mut dyn Presenter) -> Option<Flow> {
        if !self.view.is_sending() {
            return (!self.view.takes_text()).then_some(Flow::Quit);
        }
        if key.repeat {
            return Some(Flow::Continue);
        }
        if self.view.may_close() {
            return Some(Flow::Quit);
        }
        host.request_redraw();
        Some(Flow::Continue)
    }

    /// Answers the account form. The flow is made when the form opens, and
    /// let go when it closes.
    fn setup_step(&mut self) -> bool {
        if self.setup.is_none() && self.view.is_setting_up() {
            self.setup = self.live.clone().map(|live| SetupFlow::new(live, None));
        }
        let Some(flow) = self.setup.as_mut() else {
            return false;
        };
        let changed = flow.step(&mut self.view, &mut self.backend);
        if !self.view.is_setting_up() {
            self.setup = None;
        }
        changed
    }
}

impl EventSink for Screen {
    fn on_input(&mut self, event: InputEvent, host: &mut dyn Presenter) -> Flow {
        match &event {
            InputEvent::Resized { cols, rows } => {
                self.cols = *cols;
                self.rows = *rows;
                host.request_redraw();
                return Flow::Continue;
            }
            InputEvent::CloseRequested => {
                if self.view.may_close() {
                    return Flow::Quit;
                }
                // A message with text in it, or on its way, asked first.
                host.request_redraw();
                return Flow::Continue;
            }
            InputEvent::Key(key) if closes(key) => {
                if let Some(flow) = self.close_key(key, host) {
                    return flow;
                }
            }
            // Ctrl+Z is undo wherever text is being typed; anywhere else a
            // terminal gives the shell its display back and stops.
            InputEvent::Key(key) if suspends(key) && !self.view.takes_text() => {
                if host.suspend() {
                    host.request_redraw();
                    return Flow::Continue;
                }
            }
            InputEvent::PointerMoved { pos, .. } => {
                let shape = self.view.pointer_shape(pos.col, pos.row, (self.cols, self.rows));
                host.set_pointer_shape(shape);
            }
            _ => {}
        }
        // Two presses are a double-click by the time between them, which the
        // wake-ups a few times a second are too coarse to tell.
        self.view.set_clock(u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX));
        let used = self.view.handle_input(&event, (self.cols, self.rows));
        if self.view.wants_quit() {
            return Flow::Quit;
        }
        if let Some(text) = self.view.take_copied() {
            host.set_clipboard(&text);
        }
        if used {
            self.exchange(host);
            host.request_redraw();
            self.save_place_when_changed();
        }
        if matches!(event, InputEvent::Button { state: surface::ButtonState::Released, .. }) {
            // A failure here waits: closing the program tries again and says
            // so if it still cannot.
            let _ = self.save_state();
        }
        Flow::Continue
    }

    fn on_wake(&mut self, host: &mut dyn Presenter) -> Flow {
        if self.done {
            return Flow::Quit;
        }
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.view.tick(elapsed);
        if self.exchange(host) {
            host.request_redraw();
        }
        Flow::Continue
    }

    fn on_redraw(&mut self, host: &mut dyn Presenter) {
        if self.ticker.is_none() {
            self.ticker = Some(Ticker::start(host.waker()));
        }
        self.view.set_capabilities(host.capabilities());
        self.view.set_now(self.fixed_now.unwrap_or_else(system_now));
        let title = self.view.window_title();
        if title != self.title {
            host.set_title(&title);
            self.title = title;
        }
        let frame = self.view.render(self.cols, self.rows);
        host.present(&frame);
        self.frames = self.frames.saturating_add(1);
        if self.stop_after.is_some_and(|limit| self.frames >= limit) {
            // Asking the host's own loop to come back lets it finish showing
            // this frame before the run ends.
            self.done = true;
            host.waker().wake();
        }
    }
}

/// The key that suspends a terminal: `Ctrl+Z`, as in every terminal program.
fn suspends(key: &KeyEvent) -> bool {
    let held = key.modifiers;
    key.key == LogicalKey::Char('z') && held.ctrl && !held.shift && !held.alt && !held.meta
}

/// The keys that close the program: `q`, as the key bar says, and `Ctrl+Q`.
fn closes(key: &KeyEvent) -> bool {
    let held = key.modifiers;
    match key.key {
        LogicalKey::Char('q') => !held.alt && !held.meta && (held.ctrl || !held.shift),
        _ => false,
    }
}

#[cfg(test)]
include!("screen.tests.rs");
