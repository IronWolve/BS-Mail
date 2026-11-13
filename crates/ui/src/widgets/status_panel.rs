//! The status section body: last sync, new messages, sync state, storage and
//! the gauge, in the configured order.

use model::{SyncState, SyncStatus};
use surface::{Frame, Rect};

use super::cell_text::{cells, put_cut, put_text};
use super::draw_context::DrawContext;
use super::gauge::draw_gauge;
use crate::format::{format_count, format_quota};
use crate::icons::IconId;
use crate::theme::roles;

/// Blank cells between a label and its value.
const VALUE_GAP: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLine {
    LastSync,
    NewMessages,
    SyncState,
    Storage,
    Gauge,
}

impl StatusLine {
    /// The line named in `[sidebar] status_lines`; `None` for an unknown name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "last_sync" => Some(Self::LastSync),
            "new_messages" => Some(Self::NewMessages),
            "sync_state" => Some(Self::SyncState),
            "storage" => Some(Self::Storage),
            "gauge" => Some(Self::Gauge),
            _ => None,
        }
    }

    /// The configured lines in order, unknown names dropped.
    pub fn list_from_names(names: &[String]) -> Vec<Self> {
        names.iter().filter_map(|name| Self::from_name(name)).collect()
    }
}

fn state_text(state: SyncState) -> &'static str {
    match state {
        SyncState::Idle => "Idle",
        SyncState::Syncing => "Syncing",
        SyncState::Failed => "Failed",
    }
}

/// Draws the lines top to bottom, as many as fit in `body`.
pub fn draw_status_panel(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    body: Rect,
    status: &SyncStatus,
    lines: &[StatusLine],
) {
    for (index, line) in lines.iter().enumerate() {
        let Ok(dy) = u16::try_from(index) else { break };
        if dy >= body.height {
            break;
        }
        let row = Rect::new(body.x, body.y + dy, body.width, 1);
        draw_line(frame, ctx, row, status, *line);
    }
}

fn draw_line(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    row: Rect,
    status: &SyncStatus,
    line: StatusLine,
) {
    let value_role = roles::STATUS_VALUE;
    let (label, value, role) = match line {
        StatusLine::Gauge => {
            let fraction = status
                .quota
                .filter(|quota| quota.limit > 0)
                .map(|quota| quota.used as f64 / quota.limit as f64);
            draw_gauge(frame, ctx, row, fraction);
            return;
        }
        StatusLine::LastSync => (
            "Last sync:",
            status.last_sync.map_or_else(|| "never".to_string(), |at| ctx.clock(at)),
            value_role,
        ),
        StatusLine::NewMessages => (
            "New messages:",
            format_count("{n}", u64::from(status.new_messages)),
            roles::STATUS_COUNT,
        ),
        StatusLine::SyncState => ("Sync:", state_text(status.state).to_string(), value_role),
        StatusLine::Storage => (
            "Storage:",
            status.quota.map_or_else(|| "n/a".to_string(), format_quota),
            value_role,
        ),
    };
    let value_x = row.x.saturating_add(cells(label)).saturating_add(VALUE_GAP);
    put_cut(frame, row, row.x, row.y, label, row.width, ctx.fg(roles::STATUS_LABEL));
    let room = row.right().saturating_sub(value_x);
    let end = put_cut(frame, row, value_x, row.y, &value, room, ctx.fg(role));
    if line == StatusLine::LastSync && status.last_sync.is_some() {
        put_text(frame, row, end + 1, row.y, ctx.icon_inline(IconId::Check), ctx.fg(roles::STATUS_OK));
    }
}

#[cfg(test)]
include!("status_panel.tests.rs");
