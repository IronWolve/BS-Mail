//! Draws the body: highlight the plain spans, wrap every line, show the
//! visible rows from the scroll position and the scrollbar on overflow.

use model::{BodyRole, StyledLine, StyledSpan};
use surface::{Frame, Rect};

use super::cache::{self, Wrapped, WrappedBody, fingerprint};
use super::wrap::wrap_line_linked;
use crate::theme::roles;
use crate::widgets::body_highlight::{HighlightRules, highlight_line};
use crate::widgets::cell_text::{cells, put_text};
use crate::widgets::scrollbar::draw_scrollbar;
use crate::widgets::DrawContext;

/// Cells between the body's left edge and its text.
pub(super) const TEXT_AT: u16 = 1;
/// Cells the text keeps free: the blank on the left and the scrollbar's
/// column on the right.
const SIDE_CELLS: u16 = 2;

/// The rules left on when the body is not highlighted: only the addresses.
const LINKS_ONLY: HighlightRules = HighlightRules {
    bullets: false,
    lead: false,
    tokens: false,
    versions: false,
    paths: false,
    signature: false,
    quote: false,
    links: true,
    name: false,
};

/// What to draw: the lines, where they are scrolled to and the name to
/// highlight.
pub struct BodyView<'a> {
    /// Lines shown before the body (the attachments), kept apart from it so
    /// the body, which can be large, is borrowed and not copied to add them.
    pub lead: &'a [StyledLine],
    pub lines: &'a [StyledLine],
    /// Index of the first visual (wrapped) line shown.
    pub first: usize,
    /// The application's display name, shown as `Name` when it appears.
    pub app_name: &'a str,
    pub rules: HighlightRules,
}

/// The theme role a body role is drawn in.
fn role_name(role: BodyRole) -> &'static str {
    match role {
        BodyRole::Text => roles::BODY_TEXT,
        BodyRole::Quote => roles::BODY_QUOTE,
        BodyRole::Name => roles::BODY_NAME,
        BodyRole::BulletDone => roles::BODY_BULLET_DONE,
        BodyRole::BulletOpen => roles::BODY_BULLET_OPEN,
        BodyRole::Lead => roles::BODY_LEAD,
        BodyRole::LeadNext => roles::BODY_LEAD_NEXT,
        BodyRole::Token => roles::BODY_TOKEN,
        BodyRole::Path => roles::BODY_PATH,
        BodyRole::Link => roles::BODY_LINK,
        BodyRole::Signature => roles::BODY_SIGNATURE,
    }
}

/// The width lines wrap at: the body less its side cells, or the configured
/// wrap column when that is narrower.
pub(super) fn wrap_width(ctx: &DrawContext<'_>, rect: Rect) -> u16 {
    let room = rect.width.saturating_sub(SIDE_CELLS);
    match ctx.config.reader.wrap {
        0 => room,
        wrap => wrap.min(room),
    }
}

/// The wrapped lines of the body: highlighted when the options ask for it and
/// cut to the width. The last body's lines are kept, so scrolling and drawing
/// one body again and again does the work once.
pub(super) fn wrapped(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> Wrapped {
    let width = wrap_width(ctx, rect);
    let highlight = ctx.config.reader.highlight_body;
    let key = fingerprint(view.lead, view.lines, width, highlight, &view.rules, view.app_name);
    if let Some(kept) = cache::remembered(key) {
        return kept;
    }
    let made = std::rc::Rc::new(wrap_all(view, width, highlight, key));
    cache::remember(key, &made);
    made
}

fn wrap_all(view: &BodyView<'_>, width: u16, highlight: bool, key: u64) -> WrappedBody {
    let mut rows = Vec::new();
    let mut links = Vec::new();
    let mut continued = Vec::new();
    for line in view.lead.iter().chain(view.lines) {
        let spans: Vec<StyledSpan> = if highlight {
            highlight_line(line, &view.rules, view.app_name)
        } else {
            // Web addresses stay marked (and so clickable) with the other
            // colouring off: nothing else is.
            highlight_line(line, &LINKS_ONLY, view.app_name)
        };
        let wrapped = wrap_line_linked(&spans, width, &mut links);
        continued.extend((0..wrapped.len()).map(|at| at > 0));
        rows.extend(wrapped);
    }
    WrappedBody { rows, links, continued, key }
}

/// How many visual lines the body wraps to, which is the most it can scroll.
pub fn visual_len(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> usize {
    wrapped(ctx, rect, view).len()
}

/// A link of the body: its place among the links of this body (the number its
/// pieces carry, so two links with one address are told apart) and its address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkHit {
    pub index: usize,
    pub address: String,
}

/// The web address under the cell `col`, `row` of the body drawn in `rect`
/// with `view`, whole even when it is cut over several lines.
pub fn body_link_at(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>, col: u16, row: u16) -> Option<String> {
    body_link_hit(ctx, rect, view, col, row).map(|hit| hit.address)
}

/// The link under the cell `col`, `row` of the body drawn in `rect` with
/// `view`. The click, the hand pointer and the light on a link all ask this,
/// so they cannot disagree about which cells are a link.
pub fn body_link_hit(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>, col: u16, row: u16) -> Option<LinkHit> {
    if rect.is_empty() || row < rect.y || row >= rect.bottom() {
        return None;
    }
    let body = wrapped(ctx, rect, view);
    let first = view.first.min(body.len().saturating_sub(usize::from(rect.height)));
    let pieces = body.rows.get(first + usize::from(row - rect.y))?;
    let text_x = rect.x.saturating_add(TEXT_AT);
    let clip_right = text_x.saturating_add(wrap_width(ctx, rect));
    let mut x = text_x;
    for piece in pieces {
        let end = x.saturating_add(cells(&piece.text));
        if col >= x && col < end.min(clip_right) {
            let index = piece.link?;
            return body.links.get(index).map(|address| LinkHit { index, address: address.clone() });
        }
        x = end;
    }
    None
}

/// Draws the body into `rect` (the reader's text width, below the headers).
pub fn draw_reader_body(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    view: &BodyView<'_>,
) {
    if rect.is_empty() {
        return;
    }
    let visual = wrapped(ctx, rect, view);
    let visible = usize::from(rect.height);
    let first = view.first.min(visual.len().saturating_sub(visible));
    let text_x = rect.x.saturating_add(TEXT_AT);
    let text_clip = Rect::new(text_x, rect.y, wrap_width(ctx, rect), rect.height);
    for (line, pieces) in visual.rows.iter().skip(first).take(visible).enumerate() {
        let y = rect.y.saturating_add(u16::try_from(line).unwrap_or(u16::MAX));
        let mut x = text_x;
        for piece in pieces {
            x = put_text(frame, text_clip, x, y, &piece.text, ctx.fg(role_name(piece.role)));
        }
    }
    let last_col = rect.right().saturating_sub(1);
    draw_scrollbar(frame, ctx, last_col, rect, first, visible, visual.len());
}

#[cfg(test)]
include!("draw.tests.rs");
