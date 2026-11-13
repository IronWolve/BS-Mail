//! The account strip: account, mailbox, host and connection segments spread
//! across one row with a bar in each gap.

use model::{Account, ConnectionState};
use surface::{Frame, Rect};

use super::cell_text::Run;
use super::draw_context::DrawContext;
use super::segment_row::{draw_segments, segments_fit};
use crate::format::group_thousands;
use crate::icons::IconId;
use crate::layout::inset_h;
use crate::theme::roles;
use crate::view_state::HostSummary;

/// The folder shown, for the mailbox segment and the footer counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxSummary {
    pub name: String,
    pub unread: u32,
    pub total: u32,
}

/// What the strip shows.
#[derive(Debug, Clone, Copy)]
pub struct StripData<'a> {
    pub account: Option<&'a Account>,
    pub mailbox: Option<&'a MailboxSummary>,
    pub host: Option<&'a HostSummary>,
    pub connection: ConnectionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StripSegment {
    Account,
    Mailbox,
    Host,
    Connection,
}

impl StripSegment {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "account" => Some(Self::Account),
            "mailbox" => Some(Self::Mailbox),
            "host" => Some(Self::Host),
            "connection" => Some(Self::Connection),
            _ => None,
        }
    }
}

/// When the strip is too narrow, segments go in this order; the connection
/// segment stays.
const DROP_ORDER: [StripSegment; 3] = [StripSegment::Host, StripSegment::Mailbox, StripSegment::Account];

/// Connection state as the strip and footer word it, with its strip role.
pub fn connection_words(state: ConnectionState) -> (&'static str, &'static str) {
    match state {
        ConnectionState::Online => ("Online", roles::STRIP_ONLINE),
        ConnectionState::Connecting => ("Connecting", roles::STRIP_CONNECTING),
        ConnectionState::Offline => ("Offline", roles::STRIP_OFFLINE),
    }
}

fn account_runs(ctx: &DrawContext<'_>, account: &Account) -> Vec<Run> {
    vec![
        Run::new(ctx.icon(IconId::Person), ctx.fg(roles::STRIP_ICON)),
        Run::new("Account: ", ctx.fg(roles::STRIP_LABEL)),
        Run::new(account.address.clone(), ctx.fg(roles::STRIP_VALUE)),
    ]
}

fn mailbox_runs(ctx: &DrawContext<'_>, mailbox: &MailboxSummary) -> Vec<Run> {
    let label = ctx.fg(roles::STRIP_LABEL);
    vec![
        Run::new(ctx.icon(IconId::Inbox), ctx.fg(roles::STRIP_ICON)),
        Run::new(format!("Mailbox: {} (", mailbox.name), label),
        Run::new(
            format!("{} unread", group_thousands(u64::from(mailbox.unread))),
            ctx.fg(roles::STRIP_UNREAD),
        ),
        Run::new(format!(" / {} total)", group_thousands(u64::from(mailbox.total))), label),
    ]
}

fn host_runs(ctx: &DrawContext<'_>, host: &HostSummary) -> Vec<Run> {
    let label = ctx.fg(roles::STRIP_LABEL);
    let mut runs = vec![
        Run::new(ctx.icon(IconId::Host), ctx.fg(roles::STRIP_ICON)),
        Run::new("Host: ", label),
        Run::new(host.name.clone(), ctx.fg(roles::STRIP_VALUE)),
    ];
    if !host.system.is_empty() {
        runs.push(Run::new(format!(" ({})", host.system), label));
    }
    runs
}

fn connection_runs(ctx: &DrawContext<'_>, state: ConnectionState) -> Vec<Run> {
    let (word, role) = connection_words(state);
    vec![
        Run::new(ctx.icon(IconId::Dot), ctx.fg(role)),
        Run::new(format!("IMAP: {word}"), ctx.fg(role)),
    ]
}

fn runs_for(ctx: &DrawContext<'_>, kind: StripSegment, data: &StripData<'_>) -> Option<Vec<Run>> {
    match kind {
        StripSegment::Account => data.account.map(|account| account_runs(ctx, account)),
        StripSegment::Mailbox => data.mailbox.map(|mailbox| mailbox_runs(ctx, mailbox)),
        StripSegment::Host => data.host.map(|host| host_runs(ctx, host)),
        StripSegment::Connection => Some(connection_runs(ctx, data.connection)),
    }
}

/// Draws the strip into the one-row `rect` (inside the frame).
pub fn draw_account_strip(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    data: &StripData<'_>,
) {
    if rect.is_empty() {
        return;
    }
    let area = inset_h(rect, 1);
    let mut kinds: Vec<StripSegment> = ctx
        .config
        .strip
        .segments
        .iter()
        .filter_map(|name| StripSegment::from_name(name))
        .collect();
    let mut segments = build(ctx, &kinds, data);
    for dropped in DROP_ORDER {
        if segments_fit(&segments, area.width) {
            break;
        }
        kinds.retain(|kind| *kind != dropped);
        segments = build(ctx, &kinds, data);
    }
    draw_segments(frame, ctx, area, rect.y, &segments);
}

fn build(ctx: &DrawContext<'_>, kinds: &[StripSegment], data: &StripData<'_>) -> Vec<Vec<Run>> {
    kinds.iter().filter_map(|kind| runs_for(ctx, *kind, data)).collect()
}

#[cfg(test)]
include!("account_strip.tests.rs");
