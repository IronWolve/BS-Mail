//! Draws the body: highlight the plain spans, wrap every line, show the
//! visible rows from the scroll position and the scrollbar on overflow.

use model::{BodyRole, StyledLine, StyledSpan};
use surface::{Frame, Rect};

use super::cache::{self, Wrapped, fingerprint};
use super::wrap::{Piece, wrap_line};
use crate::theme::roles;
use crate::widgets::body_highlight::{HighlightRules, highlight_line};
use crate::widgets::cell_text::put_text;
use crate::widgets::scrollbar::draw_scrollbar;
use crate::widgets::DrawContext;

/// Cells between the body's left edge and its text.
const TEXT_AT: u16 = 1;
/// Cells the text keeps free: the blank on the left and the scrollbar's
/// column on the right.
const SIDE_CELLS: u16 = 2;

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
fn wrap_width(ctx: &DrawContext<'_>, rect: Rect) -> u16 {
    let room = rect.width.saturating_sub(SIDE_CELLS);
    match ctx.config.reader.wrap {
        0 => room,
        wrap => wrap.min(room),
    }
}

/// The wrapped lines of the body: highlighted when the options ask for it and
/// cut to the width. The last body's lines are kept, so scrolling and drawing
/// one body again and again does the work once.
fn wrapped(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> Wrapped {
    let width = wrap_width(ctx, rect);
    let highlight = ctx.config.reader.highlight_body;
    let key = fingerprint(view.lead, view.lines, width, highlight, &view.rules, view.app_name);
    if let Some(kept) = cache::remembered(key) {
        return kept;
    }
    let made = std::rc::Rc::new(wrap_all(view, width, highlight));
    cache::remember(key, &made);
    made
}

fn wrap_all(view: &BodyView<'_>, width: u16, highlight: bool) -> Vec<Vec<Piece>> {
    let mut visual = Vec::new();
    for line in view.lead.iter().chain(view.lines) {
        let spans: Vec<StyledSpan> = if highlight {
            highlight_line(line, &view.rules, view.app_name)
        } else {
            line.spans.clone()
        };
        visual.extend(wrap_line(&spans, width));
    }
    visual
}

/// How many visual lines the body wraps to, which is the most it can scroll.
pub fn visual_len(ctx: &DrawContext<'_>, rect: Rect, view: &BodyView<'_>) -> usize {
    wrapped(ctx, rect, view).len()
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
    for (line, pieces) in visual.iter().skip(first).take(visible).enumerate() {
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
