//! The status footer: slashes at both ends and bar-separated segments
//! between them (name and version, connection, counts, last sync, help hint).

use model::{ConnectionState, SyncStatus};
use surface::{Frame, Rect};

use super::account_strip::MailboxSummary;
use super::cell_text::{Run, put_text, runs_width};
use super::draw_context::DrawContext;
use super::key_bar::key_text;
use super::segment_row::{MIN_GAP, draw_segments, segments_fit};
use crate::app_identity::AppIdentity;
use crate::format::{ELLIPSIS_CHAR, group_thousands, truncate};
use crate::icons::IconId;
use crate::layout::{inset_h, spread_items};
use crate::theme::roles;

/// Slashes at the left and at the right edge.
const LEFT_SLASHES: u16 = 5;
const RIGHT_SLASHES: u16 = 6;
/// Blank cells between a run of slashes and the segments.
const SLASH_GAP: u16 = 2;

/// What the footer shows.
#[derive(Debug, Clone, Copy)]
pub struct FooterData<'a> {
    pub identity: &'a AppIdentity,
    pub connection: ConnectionState,
    pub mailbox: Option<&'a MailboxSummary>,
    pub status: &'a SyncStatus,
    /// The last failure the backend reported, shown in place of the usual
    /// segments until something succeeds.
    pub error: Option<&'a str>,
    /// A notice that is not a failure (new mail), shown in place of the
    /// usual segments while there is no failure to show.
    pub notice: Option<&'a str>,
}

fn text_run(ctx: &DrawContext<'_>, text: impl Into<String>) -> Run {
    Run::new(text, ctx.fg(roles::FOOTER_TEXT))
}

fn connection_segment(ctx: &DrawContext<'_>, state: ConnectionState) -> Vec<Run> {
    match state {
        ConnectionState::Online => vec![
            text_run(ctx, "IMAP "),
            Run::new(ctx.icon_inline(IconId::Check), ctx.fg(roles::FOOTER_OK)),
        ],
        ConnectionState::Connecting => vec![text_run(ctx, "IMAP connecting")],
        ConnectionState::Offline => {
            vec![Run::new("IMAP offline", ctx.fg(roles::FOOTER_ERROR))]
        }
    }
}

fn help_segment(ctx: &DrawContext<'_>) -> Vec<Run> {
    vec![
        text_run(ctx, "Press "),
        Run::new(key_text("help").unwrap_or("?"), ctx.fg(roles::FOOTER_KEY)),
        text_run(ctx, " for help"),
    ]
}

/// What a click on a footer segment does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterItem {
    /// The connection: sync now.
    Connection,
    /// "Press ? for help": open the help.
    Help,
    /// The notice of new mail: go to that account.
    NewMail,
}

/// A segment as drawn: its runs and what a click on it does, if anything.
type Segment = (Option<FooterItem>, Vec<Run>);

/// The segment named in `[footer] segments`, or `None` when its data is
/// missing or the name is unknown.
fn segment(ctx: &DrawContext<'_>, name: &str, data: &FooterData<'_>) -> Option<Segment> {
    let plain = |runs| Some((None, runs));
    match name {
        "name_version" => plain(vec![text_run(ctx, data.identity.name_and_version())]),
        "connection" => Some((Some(FooterItem::Connection), connection_segment(ctx, data.connection))),
        "unread" => data.mailbox.and_then(|mailbox| {
            let text = format!("{} unread", group_thousands(u64::from(mailbox.unread)));
            plain(vec![Run::new(text, ctx.fg(roles::FOOTER_UNREAD))])
        }),
        "total" => data.mailbox.and_then(|mailbox| {
            plain(vec![text_run(ctx, format!("{} total", group_thousands(u64::from(mailbox.total))))])
        }),
        "last_sync" => {
            let when = data.status.last_sync.map_or_else(|| "never".to_string(), |at| ctx.clock(at));
            plain(vec![text_run(ctx, format!("Last sync: {when}"))])
        }
        "help_hint" => Some((Some(FooterItem::Help), help_segment(ctx))),
        _ => None,
    }
}

/// The connection state and the failure or the notice, cut by display width
/// to the cells left beside the connection segment and the gap. When nothing
/// is left, only the connection segment remains.
fn error_segments(
    ctx: &DrawContext<'_>,
    data: &FooterData<'_>,
    (error, role): (&str, &str),
    width: u16,
) -> Vec<Segment> {
    let connection = connection_segment(ctx, data.connection);
    let room = width
        .saturating_sub(runs_width(&connection))
        .saturating_sub(MIN_GAP);
    let first = (Some(FooterItem::Connection), connection);
    if room == 0 {
        return vec![first];
    }
    let text = truncate(error, usize::from(room), ELLIPSIS_CHAR);
    // Only a notice that is not a failure names something to go to.
    let item = (data.error.is_none() && data.notice.is_some()).then_some(FooterItem::NewMail);
    vec![first, (item, vec![Run::new(text, ctx.fg(role))])]
}

/// Where the footer's segments go: the cells between the slashes, and the
/// segments that fit, each with what a click on it does.
fn footer_row(ctx: &DrawContext<'_>, rect: Rect, data: &FooterData<'_>) -> (Rect, Vec<Segment>) {
    let footer = &ctx.config.footer;
    let content = inset_h(rect, 1);
    let mut area = content;
    if footer.slashes {
        let left = content.x + LEFT_SLASHES + SLASH_GAP;
        let right = content.right().saturating_sub(RIGHT_SLASHES).saturating_sub(SLASH_GAP);
        area = Rect::new(left, rect.y, right.saturating_sub(left), 1);
    }
    let shown = data
        .error
        .map(|error| (error, roles::FOOTER_ERROR))
        .or(data.notice.map(|notice| (notice, roles::FOOTER_UNREAD)));
    let mut segments: Vec<Segment> = match shown {
        Some(text) => error_segments(ctx, data, text, area.width),
        None => footer.segments.iter().filter_map(|name| segment(ctx, name, data)).collect(),
    };
    let fits = |segments: &[Segment]| {
        let runs: Vec<Vec<Run>> = segments.iter().map(|(_, runs)| runs.clone()).collect();
        segments_fit(&runs, area.width)
    };
    while segments.len() > 1 && !fits(&segments) {
        segments.pop();
    }
    (area, segments)
}

/// Draws the footer into the one-row `rect` (inside the frame). Segments
/// that do not fit are dropped from the right.
pub fn draw_footer(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, data: &FooterData<'_>) {
    if rect.is_empty() {
        return;
    }
    if ctx.config.footer.slashes {
        let content = inset_h(rect, 1);
        let style = ctx.fg(roles::FOOTER_SLASHES);
        put_text(frame, content, content.x, rect.y, &"/".repeat(usize::from(LEFT_SLASHES)), style);
        let right_x = content.right().saturating_sub(RIGHT_SLASHES);
        put_text(frame, content, right_x, rect.y, &"/".repeat(usize::from(RIGHT_SLASHES)), style);
    }
    let (area, segments) = footer_row(ctx, rect, data);
    let runs: Vec<Vec<Run>> = segments.into_iter().map(|(_, runs)| runs).collect();
    draw_segments(frame, ctx, area, rect.y, &runs);
}

/// The cells a clickable footer segment covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterSpan {
    pub item: FooterItem,
    pub x: u16,
    pub width: u16,
}

/// The clickable segment of the footer in `rect` at column `col`, if any.
pub fn footer_item_at(ctx: &DrawContext<'_>, rect: Rect, data: &FooterData<'_>, col: u16) -> Option<FooterSpan> {
    if rect.is_empty() {
        return None;
    }
    let (area, segments) = footer_row(ctx, rect, data);
    let widths: Vec<u16> = segments.iter().map(|(_, runs)| runs_width(runs)).collect();
    let spread = spread_items(&widths, area.width, MIN_GAP)?;
    segments.iter().enumerate().find_map(|(index, (item, _))| {
        let x = area.x.saturating_add(*spread.starts.get(index)?);
        let width = widths[index];
        let span = FooterSpan { item: (*item)?, x, width };
        (col >= x && col < x.saturating_add(width)).then_some(span)
    })
}

#[cfg(test)]
include!("footer.tests.rs");
