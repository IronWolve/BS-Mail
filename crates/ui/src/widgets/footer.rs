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
use crate::layout::inset_h;
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

/// The segment named in `[footer] segments`, or `None` when its data is
/// missing or the name is unknown.
fn segment(ctx: &DrawContext<'_>, name: &str, data: &FooterData<'_>) -> Option<Vec<Run>> {
    match name {
        "name_version" => Some(vec![text_run(ctx, data.identity.name_and_version())]),
        "connection" => Some(connection_segment(ctx, data.connection)),
        "unread" => data.mailbox.map(|mailbox| {
            let text = format!("{} unread", group_thousands(u64::from(mailbox.unread)));
            vec![Run::new(text, ctx.fg(roles::FOOTER_UNREAD))]
        }),
        "total" => data.mailbox.map(|mailbox| {
            vec![text_run(ctx, format!("{} total", group_thousands(u64::from(mailbox.total))))]
        }),
        "last_sync" => {
            let when = data.status.last_sync.map_or_else(|| "never".to_string(), |at| ctx.clock(at));
            Some(vec![text_run(ctx, format!("Last sync: {when}"))])
        }
        "help_hint" => Some(help_segment(ctx)),
        _ => None,
    }
}

/// The connection state and the failure, the failure cut by display width to
/// the cells left beside the connection segment and the gap. When nothing is
/// left, only the connection segment remains.
fn error_segments(
    ctx: &DrawContext<'_>,
    data: &FooterData<'_>,
    error: &str,
    width: u16,
) -> Vec<Vec<Run>> {
    let connection = connection_segment(ctx, data.connection);
    let room = width
        .saturating_sub(runs_width(&connection))
        .saturating_sub(MIN_GAP);
    if room == 0 {
        return vec![connection];
    }
    let text = truncate(error, usize::from(room), ELLIPSIS_CHAR);
    vec![connection, vec![Run::new(text, ctx.fg(roles::FOOTER_ERROR))]]
}

/// Draws the footer into the one-row `rect` (inside the frame). Segments
/// that do not fit are dropped from the right.
pub fn draw_footer(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, data: &FooterData<'_>) {
    if rect.is_empty() {
        return;
    }
    let footer = &ctx.config.footer;
    let content = inset_h(rect, 1);
    let mut area = content;
    if footer.slashes {
        let style = ctx.fg(roles::FOOTER_SLASHES);
        put_text(frame, content, content.x, rect.y, &"/".repeat(usize::from(LEFT_SLASHES)), style);
        let right_x = content.right().saturating_sub(RIGHT_SLASHES);
        put_text(frame, content, right_x, rect.y, &"/".repeat(usize::from(RIGHT_SLASHES)), style);
        let left = content.x + LEFT_SLASHES + SLASH_GAP;
        let right = right_x.saturating_sub(SLASH_GAP);
        area = Rect::new(left, rect.y, right.saturating_sub(left), 1);
    }
    let mut segments: Vec<Vec<Run>> = match data.error {
        Some(error) => error_segments(ctx, data, error, area.width),
        None => footer
            .segments
            .iter()
            .filter_map(|name| segment(ctx, name, data))
            .collect(),
    };
    while segments.len() > 1 && !segments_fit(&segments, area.width) {
        segments.pop();
    }
    draw_segments(frame, ctx, area, rect.y, &segments);
}

#[cfg(test)]
include!("footer.tests.rs");
