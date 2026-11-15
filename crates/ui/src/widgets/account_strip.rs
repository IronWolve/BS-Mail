//! The account strip: account, mailbox, host and connection segments spread
//! across one row with a bar in each gap.

use model::{Account, ConnectionState};
use surface::{Frame, Rect};

use super::cell_text::{Run, runs_width};
use super::draw_context::DrawContext;
use super::segment_row::{MIN_GAP, draw_segments, segments_fit};
use crate::format::{ELLIPSIS_CHAR, group_thousands, truncate};
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

/// When the strip is too narrow, these segments go first, in this order. The
/// account is then cut, not dropped; the connection segment stays.
const DROP_ORDER: [StripSegment; 2] = [StripSegment::Host, StripSegment::Mailbox];

/// Fewest cells of a cut address worth showing; with less room the account
/// segment is dropped.
const MIN_ADDRESS: u16 = 4;

/// Connection state as the strip and footer word it, with its strip role.
pub fn connection_words(state: ConnectionState) -> (&'static str, &'static str) {
    match state {
        ConnectionState::Online => ("Online", roles::STRIP_ONLINE),
        ConnectionState::Connecting => ("Connecting", roles::STRIP_CONNECTING),
        ConnectionState::Offline => ("Offline", roles::STRIP_OFFLINE),
    }
}

/// The icon and label of the account segment, before the address.
fn account_prefix(ctx: &DrawContext<'_>) -> Vec<Run> {
    vec![
        Run::new(ctx.icon(IconId::Person), ctx.fg(roles::STRIP_ICON)),
        Run::new("Account: ", ctx.fg(roles::STRIP_LABEL)),
    ]
}

/// The account segment, the address cut to `max_address` cells when given.
fn account_runs(ctx: &DrawContext<'_>, account: &Account, max_address: Option<u16>) -> Vec<Run> {
    let mut runs = account_prefix(ctx);
    let address = match max_address {
        Some(max) => truncate(&account.address, usize::from(max), ELLIPSIS_CHAR),
        None => account.address.clone(),
    };
    runs.push(Run::new(address, ctx.fg(roles::STRIP_VALUE)));
    runs
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

fn runs_for(
    ctx: &DrawContext<'_>,
    kind: StripSegment,
    data: &StripData<'_>,
    max_address: Option<u16>,
) -> Option<Vec<Run>> {
    match kind {
        StripSegment::Account => data.account.map(|account| account_runs(ctx, account, max_address)),
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
    let mut segments = build(ctx, &kinds, data, None);
    for dropped in DROP_ORDER {
        if segments_fit(&segments, area.width) {
            break;
        }
        kinds.retain(|kind| *kind != dropped);
        segments = build(ctx, &kinds, data, None);
    }
    if !segments_fit(&segments, area.width) && kinds.contains(&StripSegment::Account) {
        match address_room(ctx, &kinds, data, area.width) {
            Some(room) => segments = build(ctx, &kinds, data, Some(room)),
            None => {
                kinds.retain(|kind| *kind != StripSegment::Account);
                segments = build(ctx, &kinds, data, None);
            }
        }
    }
    draw_segments(frame, ctx, area, rect.y, &segments);
}

fn build(
    ctx: &DrawContext<'_>,
    kinds: &[StripSegment],
    data: &StripData<'_>,
    max_address: Option<u16>,
) -> Vec<Vec<Run>> {
    kinds
        .iter()
        .filter_map(|kind| runs_for(ctx, *kind, data, max_address))
        .collect()
}

/// The cells the address may use once every other segment and the gaps
/// between them are placed; `None` when that is under `MIN_ADDRESS` or there
/// is no account to show.
fn address_room(
    ctx: &DrawContext<'_>,
    kinds: &[StripSegment],
    data: &StripData<'_>,
    width: u16,
) -> Option<u16> {
    data.account?;
    let others: Vec<StripSegment> = kinds
        .iter()
        .copied()
        .filter(|kind| *kind != StripSegment::Account)
        .collect();
    let placed = build(ctx, &others, data, None);
    let others_width: u32 = placed.iter().map(|runs| u32::from(runs_width(runs))).sum();
    // One gap beside the account for each other segment that is drawn.
    let gaps = u32::try_from(placed.len()).ok()? * u32::from(MIN_GAP);
    let prefix = u32::from(runs_width(&account_prefix(ctx)));
    let room = u32::from(width).checked_sub(others_width + gaps + prefix)?;
    let room = u16::try_from(room).ok()?;
    (room >= MIN_ADDRESS).then_some(room)
}

#[cfg(test)]
include!("account_strip.tests.rs");
