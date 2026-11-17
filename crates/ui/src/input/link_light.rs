//! The link under the pointer and the link the keys are on: which they are,
//! whether they may be lit, and what the footer says about the one it speaks of.
//! The click, the hand pointer and the light all ask `link_hit_at`, which asks
//! the body's own hit test, so they agree on which cells are a link. Nothing
//! here is stored but the footer note that was showing when the pointer came
//! onto a link (or the keys onto one), so the hint knows which notes arrived
//! after it.

use super::hit::Hit;
use crate::app_view::AppView;
use crate::layout::Layout;
use crate::reader_tools::Basis;
use crate::view_facts::ReaderContent;
use crate::widgets::{BodyView, DrawContext, HighlightRules, LinkHint, LinkHit, LinkState, body_link_hit};

/// A link the reader lights, as the screen draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LitLink {
    pub hit: LinkHit,
    pub state: LinkState,
}

/// The links lit in one drawing: the one the pointer is on and the one the
/// keys are on, which may be two links, or one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LitLinks {
    pub pointer: Option<LitLink>,
    pub keys: Option<LitLink>,
}

impl LitLinks {
    /// The links as the drawing wants them. The pointer's goes first, so on a
    /// link both are on it is the pointer's light that shows.
    pub(crate) fn lights(&self) -> Vec<(usize, LinkState)> {
        [&self.pointer, &self.keys].into_iter().flatten().map(|lit| (lit.hit.index, lit.state)).collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pointer.is_none() && self.keys.is_none()
    }

    /// The link the footer says where it goes: the pointer's, else the keys'.
    fn spoken_of(&self) -> Option<&LitLink> {
        self.pointer.as_ref().or(self.keys.as_ref())
    }
}

impl AppView {
    /// Whether the pointer may light a link at all: nothing is open over the
    /// mail, nothing is being dragged (a bar, rows, or text being selected),
    /// and the pane shows the message, not its raw source.
    pub(crate) fn link_light_allowed(&self) -> bool {
        self.pointer_is_free() && self.reading.raw_of(self.selection.message).is_none()
    }

    /// The link of the body at the cell `col`, `row` of the screen, with the
    /// same answer the click and the hand pointer get. The find box takes a
    /// press before a link does, so a cell it covers holds no link.
    pub(crate) fn link_hit_at(&self, layout: &Layout, col: u16, row: u16) -> Option<LinkHit> {
        let content = self.reader_content()?;
        self.link_hit_in(layout, &content, col, row)
    }

    fn link_hit_in(&self, layout: &Layout, content: &ReaderContent<'_>, col: u16, row: u16) -> Option<LinkHit> {
        let reader = layout.reader.as_ref()?;
        let ctx = DrawContext::new(&self.theme, &self.icons, &self.config, self.now);
        let view = BodyView {
            lead: &content.lead,
            lines: content.body,
            first: self.scroll.reader,
            app_name: &self.identity.name,
            rules: HighlightRules::default(),
        };
        let hit = body_link_hit(&ctx, reader.body, &view, col, row)?;
        (!self.over_find_bar(layout, col, row)).then_some(hit)
    }

    /// The place of the link under the cell among the message's links, for the
    /// pointer's moves: the answer is kept while the cell, the scroll and the
    /// body are the same, so a move within one cell, and the two questions each
    /// move asks, hash the body once at most.
    pub(crate) fn link_index_at(&self, layout: &Layout, col: u16, row: u16) -> Option<usize> {
        let reader = layout.reader.as_ref()?;
        let content = self.reader_content()?;
        let basis = Basis {
            cell: (col, row),
            first: self.scroll.reader,
            body: reader.body,
            message: self.selection.message,
            lines: (content.body.as_ptr() as usize, content.body.len()),
            lead: content.lead.len(),
            settings: (self.config.reader.wrap, self.config.reader.highlight_body),
        };
        self.reading.link_answers.answer(basis, || self.link_hit_in(layout, &content, col, row).map(|hit| hit.index))
    }

    /// The links lit now, for drawing.
    pub(crate) fn lit_links(&self, layout: &Layout) -> LitLinks {
        LitLinks { pointer: self.lit_link(layout), keys: self.key_link().filter(|_| !layout.too_small) }
    }

    /// The link the pointer is lighting now, worked out from where the pointer
    /// is, for drawing: never from a kept answer, so what is drawn is what the
    /// body drawn in the same frame holds.
    fn lit_link(&self, layout: &Layout) -> Option<LitLink> {
        let (col, row) = self.memory.hover?;
        if layout.too_small || !self.link_light_allowed() || self.hit(layout, col, row) != Hit::Reader {
            return None;
        }
        let hit = self.link_hit_at(layout, col, row)?;
        // The button is down on the link and has not become a selection yet.
        let pressed = self.reading.pending_link.is_some() && self.reading.selection.as_ref().is_some_and(|held| held.dragging && !held.spread);
        Some(LitLink { hit, state: if pressed { LinkState::Pressed } else { LinkState::Hover } })
    }

    /// The note the footer is showing, if any: a failure, a note, the word the
    /// backend is awaited for or a problem with the settings.
    pub(crate) fn footer_note(&self) -> Option<&str> {
        self.mail.error.as_deref().or(self.mail.setup_problem.as_deref()).or(self.mail.done_text.as_deref()).or(self.config_problem.as_deref())
    }

    /// What the footer says about the lit link, while it is lit: the one the
    /// pointer is on, else the one the keys are on. A note that came while it
    /// was lit (the browser is being opened, none was found) is what the footer
    /// shows, not this: it is the one the user must read. This beats an older
    /// note, the notice of new mail and the usual segments. Worked out at each
    /// drawing and kept nowhere, so it is gone in the frame the link goes out.
    pub(crate) fn link_hint(&self, lit: &LitLinks) -> Option<LinkHint> {
        let lit = lit.spoken_of()?;
        let newer = self.footer_note().is_some_and(|note| self.memory.hint_note.as_deref() != Some(note));
        (!newer).then(|| LinkHint::for_address(&lit.hit.address))
    }
}

#[cfg(test)]
include!("link_light.tests.rs");
