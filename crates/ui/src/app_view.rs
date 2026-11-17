//! The screen's state and its drawing: events from the backend go in, a
//! frame comes out. No mouse and no key handling yet.

use std::collections::BTreeSet;

use model::config::Config;
use model::{Event, MailDate, MessageId};
use surface::{Capabilities, Frame, Rect};

use crate::app_identity::AppIdentity;
use crate::icons::IconSet;
use crate::composer::{AddressBook, ComposeForm};
use crate::composer::EditorBuffer;
use crate::ask::Ask;
use crate::delete_prompt::DeletePrompt;
use crate::input::done_note::DoneNote;
use crate::move_picker::MovePicker;
use crate::options::OptionsState;
use crate::options::draw::draw_options;
use crate::options::host::OptionsHost;
use crate::passphrase::PassphraseDialog;
use crate::passphrase::dialog_draw::draw_dialog;
use crate::composer::MailtoDraft;
use crate::reader_tools::ReaderTools;
use crate::search::ActiveSearch;
use crate::setup::SetupForm;
use crate::setup::draw::draw_setup;
use crate::input::memory::InputMemory;
use crate::view_bands::draw_too_small;
use crate::input::menus::OpenMenu;
use crate::keys::KeyMap;
use crate::list_order::Arrangement;
use crate::input::new_mail::NewMail;
use crate::input::scrollbars::BarDrag;
use crate::input::{CommandQueue, Drag};
use crate::layout::{
    FolderSpacing, GridSize, Layout, LayoutOptions, LayoutPrefs, SidebarContent, compute_layout,
};
use crate::theme::{Theme, roles};
use crate::view_state::{
    Focus, MailSnapshot, Overlays, Place, ScrollState, Selection,
};
use crate::widgets::{
    DrawContext, StatusLine, content_rows, draw_chrome_lines, fill_cells, visible_rows,
};

pub struct AppView {
    pub(crate) identity: AppIdentity,
    pub(crate) theme: Theme,
    pub(crate) config: Config,
    /// The keys in force: the defaults with the `[keys]` table over them.
    pub(crate) keys: KeyMap,
    pub(crate) icons: IconSet,
    capabilities: Capabilities,
    pub(crate) now: MailDate,
    pub(crate) prefs: LayoutPrefs,
    pub(crate) mail: MailSnapshot,
    pub(crate) selection: Selection,
    pub(crate) scroll: ScrollState,
    pub(crate) focus: Focus,
    pub(crate) overlays: Overlays,
    pub(crate) queue: CommandQueue,
    /// The message that will be marked read, and when (milliseconds on the
    /// caller's clock).
    pub(crate) read_timer: Option<(MessageId, u64)>,
    /// The message this view already marked read by itself, so it is not
    /// marked again right after the user marks it unread.
    pub(crate) auto_marked: Option<MessageId>,
    /// The bar being dragged, while a button is held on it.
    pub(crate) drag: Option<Drag>,
    /// The scrollbar thumb being dragged, while a button is held on it.
    pub(crate) bar_drag: Option<BarDrag>,
    /// The place remembered from the last run that is still to be opened.
    pub(crate) restore: Option<Place>,
    /// The notice of new mail the footer shows.
    pub(crate) new_mail: Option<NewMail>,
    /// The last time the caller's clock was told (see `tick`).
    pub(crate) last_tick_ms: u64,
    /// The caller's clock as of the last input (see `set_clock`), for telling
    /// a double-click from two clicks.
    pub(crate) clock_ms: u64,
    /// The context menu, while one is open.
    pub(crate) menu: Option<OpenMenu>,
    /// What the pointer and the keys left behind for the next input.
    pub(crate) memory: InputMemory,
    /// When the program last sent a change of mail, on that clock.
    pub(crate) own_change_ms: Option<u64>,
    /// The message being written, when the composer is open.
    pub(crate) composer: Option<ComposeForm>,
    /// The addresses seen and written to, for completing recipients.
    pub(crate) addresses: AddressBook,
    /// The user clicked Quit; the program closes it, the screen cannot.
    pub(crate) quit_requested: bool,
    /// Web addresses the user clicked, for the program to open in a browser.
    pub(crate) link_requests: Vec<String>,
    /// Text the user copied from a menu (a link, a sender, a subject), for the
    /// host to put on the clipboard.
    pub(crate) copied_link: Option<String>,
    /// The letter of a command key just pressed, whose text is still to come:
    /// a window and a terminal both report a printable key as the key and
    /// then the text it types, and the text of a key that did something is
    /// not for whatever that opened.
    pub(crate) paired_text: Option<char>,
    /// Messages marked with Space, which the change keys then act on together.
    pub(crate) marked: BTreeSet<MessageId>,
    /// The folder picker, while one is open to move messages.
    pub(crate) picker: Option<MovePicker>,
    /// The question before mail is deleted, while it is open.
    pub(crate) delete_prompt: Option<DeletePrompt>,
    /// The box of a folder action or of the tags, while one is open.
    pub(crate) ask: Option<Ask>,
    /// The word that waits for the backend's answer (see `done_note`).
    pub(crate) done_note: Option<DoneNote>,
    /// The account row the keyboard cursor is on, once the sidebar's cursor
    /// has run on past the last folder.
    pub(crate) account_cursor: Option<usize>,
    /// What the footer says once the move the user just asked for is done
    /// ("Moved 3 to Archive"); dropped when the backend reports a failure.
    pub(crate) moved_note: Option<String>,
    /// The search box, while one is open to take a query.
    pub(crate) search_bar: Option<EditorBuffer>,
    /// The search whose results are showing in the list.
    pub(crate) search: Option<ActiveSearch>,
    /// The number of the last search asked for, so answers can be told apart.
    pub(crate) next_request: u64,
    /// What was wrong with the user's settings, shown in the footer for as
    /// long as nothing more urgent is.
    pub(crate) config_problem: Option<String>,
    /// The account form, while the program has no account yet.
    pub(crate) setup: Option<SetupForm>,
    /// The account file text the user asked to try, until the program takes it.
    pub(crate) setup_request: Option<String>,
    /// The account that was tried has signed in; the program saves it next.
    pub(crate) setup_online: bool,
    /// What the form asked the program to do, not yet taken.
    pub(crate) setup_actions: Vec<crate::setup::SetupAction>,
    /// The client ids the program holds, which the form fills in for a
    /// provider it knows.
    pub(crate) saved_clients: Vec<crate::setup::SavedClient>,
    /// What the reading pane does besides showing the message: the selected
    /// text, the find box, the raw view and the pages for the browser.
    pub(crate) reading: ReaderTools,
    /// A mail link the program was started with, waiting for an account to
    /// write from.
    pub(crate) pending_mailto: Option<MailtoDraft>,
    /// The Options screen, while it is open.
    pub(crate) options: Option<OptionsState>,
    /// What the program told the Options screen, and what it asked for.
    pub(crate) options_host: OptionsHost,
    /// The dialog that sets, changes or gives up a passphrase, while it is
    /// open over the Options screen.
    pub(crate) passphrase_dialog: Option<PassphraseDialog>,
}

impl AppView {
    pub fn new(identity: AppIdentity, theme: Theme, config: Config, icons: IconSet) -> Self {
        let arrangement = Arrangement::from_config(&config.list);
        Self {
            identity,
            theme,
            keys: KeyMap::new(&config.keys),
            config,
            icons,
            capabilities: Capabilities::window(),
            now: MailDate::default(),
            prefs: LayoutPrefs::default(),
            mail: MailSnapshot { arrangement, ..MailSnapshot::default() },
            selection: Selection::default(),
            scroll: ScrollState::default(),
            focus: Focus::default(),
            overlays: Overlays::default(),
            queue: CommandQueue::default(),
            read_timer: None,
            auto_marked: None,
            drag: None,
            bar_drag: None,
            restore: None,
            new_mail: None,
            last_tick_ms: 0,
            clock_ms: 0,
            menu: None,
            memory: InputMemory::default(),
            own_change_ms: None,
            composer: None,
            addresses: AddressBook::default(),
            quit_requested: false,
            link_requests: Vec::new(),
            copied_link: None,
            paired_text: None,
            marked: BTreeSet::new(),
            picker: None,
            delete_prompt: None,
            ask: None,
            done_note: None,
            account_cursor: None,
            moved_note: None,
            search_bar: None,
            search: None,
            next_request: 0,
            config_problem: None,
            setup: None,
            setup_request: None,
            setup_online: false,
            setup_actions: Vec::new(),
            saved_clients: Vec::new(),
            reading: ReaderTools::default(),
            pending_mailto: None,
            options: None,
            options_host: OptionsHost::default(),
            passphrase_dialog: None,
        }
    }

    /// The moment the screen treats as the present, for Today and Yesterday
    /// and for clock times.
    pub fn set_now(&mut self, now: MailDate) {
        self.now = now;
    }

    /// What the host showing these frames can do. A window is assumed until
    /// a host says otherwise, so the screen is laid out for every effect.
    pub fn set_capabilities(&mut self, capabilities: Capabilities) {
        self.capabilities = capabilities;
        if let Some(shown) = self.theme.for_depth(capabilities.color_depth) {
            self.theme = shown;
        }
    }

    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// The preferred sizes now, for saving after a drag.
    pub fn prefs(&self) -> &LayoutPrefs {
        &self.prefs
    }

    /// Replaces the preferred sizes (a drag or a restored state file).
    pub fn set_prefs(&mut self, prefs: LayoutPrefs) {
        self.prefs = prefs;
    }

    /// Stores what one backend event carries for the regions.
    pub fn apply_event(&mut self, event: Event) {
        self.reading.link_answers.forget();
        self.note_send_outcome(&event);
        self.note_addresses(&event);
        self.note_search_event(&event);
        self.note_reading_event(&event);
        if matches!(event, Event::EnvelopePage { offset: 0, .. }) {
            self.queue.first_page_arrived();
        }
        let (folder, message) = (self.selection.folder, self.selection.message);
        let done = self.settle_moved_note(&event);
        let said = self.settle_done_note(&event);
        let lists = matches!(event, Event::AccountList(_) | Event::FolderList { .. });
        let rise = self.unread_rise(&event);
        self.mail.apply(event, &mut self.selection);
        if let Some((account, by)) = rise {
            self.note_new_mail(&account, by);
        }
        if done.is_some() {
            self.mail.set_note(done);
        }
        if said.is_some() {
            self.mail.done_text = said;
        }
        // A folder list that no longer has the open folder moves the
        // selection to another one: nothing of the old folder stays shown.
        if folder.is_some() && self.selection.folder != folder {
            self.forget_folder_view();
        }
        self.settle_unread_waits();
        // Another message is read from the top.
        if self.selection.message != message {
            self.scroll.reader = 0;
        }
        if lists {
            self.apply_restore();
        }
        self.prune_marks();
        self.try_pending_mailto();
    }

    /// How many messages are marked.
    pub fn marked_count(&self) -> usize {
        self.marked.len()
    }

    /// Whether the user asked to quit from the screen, by the Quit button.
    pub fn wants_quit(&self) -> bool {
        self.quit_requested
    }

    /// Whether the composer is open, so the program leaves the letters to it.
    pub fn is_composing(&self) -> bool {
        self.composer.is_some()
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn mail(&self) -> &MailSnapshot {
        &self.mail
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn overlays(&self) -> &Overlays {
        &self.overlays
    }

    pub fn identity(&self) -> &AppIdentity {
        &self.identity
    }

    /// The layout of a `cols` x `rows` grid for the data held now.
    pub fn layout(&self, cols: u16, rows: u16) -> Layout {
        let folders = visible_rows(&self.mail.folders);
        let count = |len: usize| u16::try_from(len).unwrap_or(u16::MAX);
        let content = SidebarContent {
            folder_rows_roomy: content_rows(&folders, FolderSpacing::Roomy),
            folder_rows_compact: content_rows(&folders, FolderSpacing::Compact),
            account_rows: count(self.mail.accounts.len()),
            status_lines: count(StatusLine::list_from_names(&self.config.sidebar.status_lines).len()),
        };
        let mut options = LayoutOptions::from_config(&self.config, content)
            .fit_to_host(&self.capabilities.effects);
        options.reader_full = self.focus == Focus::Reader;
        compute_layout(&self.prefs, &options, GridSize::new(cols, rows))
    }

    /// Draws the whole screen for a `cols` x `rows` grid.
    pub fn render(&self, cols: u16, rows: u16) -> Frame {
        let mut frame = Frame::new(Rect::new(0, 0, cols, rows));
        frame.line_weight = self.config.appearance.line_weight.max(1);
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now).with_keys(&self.keys);
        let area = frame.area();
        if let Some(form) = &self.setup {
            draw_setup(&mut frame, &ctx, area, form);
            return frame;
        }
        if let Some(state) = &self.options {
            draw_options(&mut frame, &ctx, area, &self.options_view(), state);
            if let Some(dialog) = &self.passphrase_dialog {
                draw_dialog(&mut frame, &ctx, area, dialog);
            }
            return frame;
        }
        fill_cells(&mut frame, area, ctx.full(roles::BASE));
        let layout = self.layout(cols, rows);
        if layout.too_small {
            draw_too_small(&mut frame, &ctx);
            return frame;
        }
        draw_chrome_lines(&mut frame, &ctx, &layout);
        let lit = self.lit_link(&layout);
        self.draw_bands(&mut frame, &ctx, &layout, lit.as_ref());
        if let Some(sidebar) = &layout.sidebar {
            self.draw_sidebar(&mut frame, &ctx, sidebar);
        }
        if let Some(list) = &layout.list {
            self.draw_list(&mut frame, &ctx, list);
        }
        if let Some(reader) = &layout.reader {
            self.draw_reader(&mut frame, &ctx, reader, lit.as_ref());
        }
        self.mark_focus(&mut frame, &layout);
        self.draw_hover(&mut frame, &ctx, &layout);
        self.draw_modals(&mut frame, &ctx, &layout, &self.capabilities);
        frame
    }
}

#[cfg(test)]
include!("app_view.tests.rs");
