//! The links of the open message from the keyboard. `l` puts the keys on the
//! next link (and `L` on the one before; past the last comes the first and
//! before the first the last, as `x` and `X` go round the attachments),
//! whichever pane had the focus, and the reader then reads two keys as its
//! own: `Enter` opens the link as a click on it does (a web address in the
//! browser, a mail address in the composer) and `Esc` lets it go. The link is
//! drawn as the keyboard's link (see `link_light`) with the footer saying
//! where it goes, and the pane scrolls by the least that shows all of it. The
//! cursor belongs to one message; another message opening lets it go.
//!
//! With the cursor on none, the first press starts where the reader is, so a
//! reader who has scrolled down does not jump back to the top: `l` takes the
//! first link that starts at or after the first row in view and `L` the last
//! link that starts before the last row in view; where there is none that way
//! they go round, to the first and the last link of the message.
//!
//! `y` is the copy key of the reader (the selected text, or the whole message),
//! so it does not copy the link: the menu keys open the link's menu while it is
//! chosen (see `link_menu`), and Copy link is in it, as in the right-click menu.

use surface::{KeyEvent, LogicalKey, NamedKey};

use crate::app_view::AppView;
use crate::input::link_light::LitLink;
use crate::layout::Layout;
use crate::reader_tools::LinkCursor;
use crate::view_state::Focus;
use crate::widgets::{BodyView, DrawContext, HighlightRules, LinkHit, LinkSpan, LinkState, body_links};

/// The place among `links` (not empty, in the order they come in the body)
/// that a first press takes, with the cursor on none: the first link that
/// starts at or after row `first` for `l`, the last that starts at or before
/// row `last` (the last row in view) for `L`, and the first or the last link
/// when there is none that way.
fn first_taken(links: &[LinkSpan], forward: bool, (first, last): (usize, usize)) -> usize {
    if forward {
        links.iter().position(|link| link.rows.0 >= first).unwrap_or(0)
    } else {
        links.iter().rposition(|link| link.rows.0 <= last).unwrap_or(links.len() - 1)
    }
}

/// What the footer says when the open message has no link to choose.
const NO_LINKS: &str = "No links";
/// What it says while the raw headers or source take the place of the message.
const RAW_VIEW: &str = "No links to choose in the raw view";

impl AppView {
    /// The links of the message the reader shows, in the order they come in it,
    /// as the reader's body is wrapped in `layout`.
    pub(super) fn reader_links(&self, layout: &Layout) -> Vec<LinkSpan> {
        let (Some(reader), Some(content)) = (layout.reader.as_ref(), self.reader_content()) else {
            return Vec::new();
        };
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let view = BodyView {
            lead: &content.lead,
            lines: content.body,
            first: self.scroll.reader,
            app_name: &self.identity.name,
            rules: HighlightRules::default(),
        };
        body_links(&ctx, reader.body, &view)
    }

    /// Whether the keys' link is in play: the reader has the focus, the cursor
    /// is on a link of the message shown, and nothing is over the mail or
    /// taking the reader's keys (the find box, the raw view). Then the link is
    /// drawn, named in the footer, and `Enter` and `Esc` are for it.
    pub(crate) fn link_mode(&self) -> bool {
        self.focus == Focus::Reader
            && self.reading.link_cursor.as_ref().is_some_and(|cursor| cursor.is_on(self.selection.message))
            && !self.find_is_open()
            && self.mail_is_free()
            && self.reading.raw_of(self.selection.message).is_none()
    }

    /// The link the keys are on, as the screen draws it, while it is in play.
    pub(crate) fn key_link(&self) -> Option<LitLink> {
        let cursor = self.reading.link_cursor.as_ref().filter(|_| self.link_mode())?;
        Some(LitLink { hit: LinkHit { index: cursor.index, address: cursor.address.clone() }, state: LinkState::Focus })
    }

    /// `l` and `L`: the keys to the next link of the open message, or the one
    /// before; the first (or the last) when they were on none. The reader takes
    /// the focus and shows the link. A message with no link only says so.
    pub(crate) fn step_link(&mut self, forward: bool, grid: (u16, u16)) {
        if self.reading.raw_of(self.selection.message).is_some() {
            self.mail.set_note(Some(RAW_VIEW.to_owned()));
            return;
        }
        // The links are wrapped to the pane they are drawn in, which is the
        // whole screen's when the reader covers the list: look in the reader's.
        let before = self.focus;
        self.focus = Focus::Reader;
        let layout = self.layout(grid.0, grid.1);
        let links = self.reader_links(&layout);
        let Some(message) = self.selection.message.filter(|_| !links.is_empty()) else {
            self.focus = before;
            self.mail.set_note(Some(NO_LINKS.to_owned()));
            return;
        };
        let count = links.len();
        let on = self.reading.link_cursor.as_ref().filter(|cursor| cursor.is_on(Some(message)));
        let next = match (on.and_then(|cursor| links.iter().position(|link| link.index == cursor.index)), forward) {
            (None, forward) => first_taken(&links, forward, self.rows_in_view(&layout)),
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
        };
        let chosen = &links[next];
        self.reading.link_cursor = Some(LinkCursor { message, index: chosen.index, address: chosen.address.clone() });
        // One cursor at a time in the reader: the attachment's goes.
        self.reading.attachments.cursor = None;
        self.show_link(chosen, &layout);
        self.mail.set_note(None);
        self.memory.hint_note = self.footer_note().map(str::to_owned);
    }

    /// The first and the last wrapped row of the message in view in the reader,
    /// as the scroll holds it (never past the last page).
    pub(super) fn rows_in_view(&self, layout: &Layout) -> (usize, usize) {
        let (lines, height) = self.reader_extent(layout).unwrap_or((0, 0));
        let first = self.scroll.reader.min(lines.saturating_sub(height));
        (first, first + height.saturating_sub(1))
    }

    /// Scrolls the reader by the least that shows all of `link`; a link taller
    /// than the pane is shown from its first row.
    fn show_link(&mut self, link: &LinkSpan, layout: &Layout) {
        let Some((lines, height)) = self.reader_extent(layout) else {
            return;
        };
        let (top, bottom) = link.rows;
        let last_first = lines.saturating_sub(height);
        let first = self.scroll.reader.min(last_first);
        let wanted = if top < first {
            top
        } else if bottom >= first + height {
            if bottom - top < height { bottom + 1 - height } else { top }
        } else {
            first
        };
        self.scroll.reader = wanted.min(last_first);
    }

    /// The keys of the link mode: `Enter` opens the link as a click does, `Esc`
    /// lets it go (the reader keeps the focus). False for any other key.
    pub(crate) fn link_mode_key(&mut self, key: &KeyEvent) -> bool {
        let held = key.modifiers;
        if held.ctrl || held.alt || held.meta {
            return false;
        }
        match key.key {
            // A key held down must not start the browser again and again.
            LogicalKey::Named(NamedKey::Enter) if key.repeat => {}
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(address) = self.reading.link_cursor.as_ref().map(|cursor| cursor.address.clone()) {
                    self.open_link(address);
                }
            }
            LogicalKey::Named(NamedKey::Escape) => self.reading.link_cursor = None,
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
include!("link_keys.tests.rs");
