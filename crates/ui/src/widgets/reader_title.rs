//! The reading pane's title row: envelope icon, the subject in one style, and
//! the position counter (`1 / 1,286 (42 unread)`) at the right edge.

use surface::{Frame, Rect};

use super::cell_text::{Run, put_runs, put_runs_cut, put_text, runs_width};
use super::draw_context::DrawContext;
use super::subject_style::subject_runs;
use crate::format::group_thousands;
use crate::icons::IconId;
use crate::theme::roles;

/// Cells from the left edge to the subject (the icon slot and a blank).
const TITLE_AT: u16 = 3;
/// Fewest cells the subject keeps when the counter is shown.
const MIN_TITLE: u16 = 8;

/// What the title shows for the open message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReaderTitleData<'a> {
    pub subject: &'a str,
    /// The message's 1-based position in the folder.
    pub position: u64,
    /// Messages in the folder.
    pub total: u64,
    /// Unread messages in the folder.
    pub unread: u32,
}

fn counter_runs(ctx: &DrawContext<'_>, data: &ReaderTitleData<'_>) -> Vec<Run> {
    let place = format!(
        "{} / {}",
        group_thousands(data.position),
        group_thousands(data.total)
    );
    let unread = format!("({} unread)", group_thousands(u64::from(data.unread)));
    vec![
        Run::new(place, ctx.fg(roles::READER_COUNTER)),
        Run::new(" ", ctx.fg(roles::READER_COUNTER)),
        Run::new(unread, ctx.fg(roles::READER_COUNTER_UNREAD)),
    ]
}

/// Draws the title into the one-row `rect` (the reader's text width). With no
/// message only the icon is drawn. The counter is left out when
/// `[reader] show_counter` is off or the row is too narrow for it.
pub fn draw_reader_title(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    data: Option<&ReaderTitleData<'_>>,
) {
    if rect.is_empty() {
        return;
    }
    put_text(frame, rect, rect.x, rect.y, ctx.icon(IconId::Mail), ctx.fg(roles::READER_ICON));
    let Some(data) = data else {
        return;
    };
    let title_x = rect.x.saturating_add(TITLE_AT);
    let counter = counter_runs(ctx, data);
    let counter_width = runs_width(&counter);
    let wanted = counter_width.saturating_add(1).saturating_add(MIN_TITLE);
    let show = ctx.config.reader.show_counter
        && rect.right().saturating_sub(title_x) >= wanted;
    let right = if show { rect.right() - counter_width - 1 } else { rect.right() };
    let base = ctx.fg(roles::READER_TITLE);
    let runs = subject_runs(ctx, data.subject, base, &|role| ctx.fg(role), false);
    put_runs_cut(frame, rect, title_x, rect.y, &runs, right.saturating_sub(title_x));
    if show {
        put_runs(frame, rect, rect.right() - counter_width, rect.y, &counter);
    }
}

#[cfg(test)]
include!("reader_title.tests.rs");
