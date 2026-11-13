//! The reading pane's header block: one `Label:  value` line per configured
//! header, labels in one column and values in the next.

use model::Envelope;
use surface::{Frame, Rect};

use super::cell_text::put_cut;
use super::draw_context::DrawContext;
use crate::format::{CivilTime, format_pattern, header_address_list};
use crate::theme::roles;

/// Cells from the left edge of the block to the label.
const LABEL_AT: u16 = 1;
/// Cells from the label column to the value column.
const VALUE_OFFSET: u16 = 10;

/// A header line: its label, its value and the role the value is drawn in.
struct HeaderLine {
    label: &'static str,
    value: String,
    role: &'static str,
}

fn header_line(ctx: &DrawContext<'_>, name: &str, envelope: &Envelope) -> Option<HeaderLine> {
    let (label, value, role) = match name {
        "from" => ("From:", header_address_list(&envelope.from), roles::READER_HEADER_VALUE),
        "to" => ("To:", header_address_list(&envelope.to), roles::READER_HEADER_VALUE),
        "cc" => ("Cc:", header_address_list(&envelope.cc), roles::READER_HEADER_VALUE),
        "subject" => ("Subject:", envelope.subject.clone(), roles::READER_HEADER_VALUE),
        "date" => ("Date:", date_text(ctx, envelope), roles::READER_HEADER_VALUE),
        "tags" => ("Tags:", envelope.tags.join(", "), roles::READER_HEADER_TAGS),
        _ => return None,
    };
    (!value.trim().is_empty()).then_some(HeaderLine { label, value, role })
}

/// The date in the sender's own zone, in the configured pattern.
fn date_text(ctx: &DrawContext<'_>, envelope: &Envelope) -> String {
    let time = CivilTime::from_unix(envelope.date.unix, envelope.date.offset_minutes);
    format_pattern(&ctx.config.reader.date_format, &time)
}

/// Draws the configured header lines of `envelope` into `rect`, top down.
/// A header with no value is skipped, an unknown header name is ignored, and
/// the lines stop when the rows run out.
pub fn draw_reader_headers(
    frame: &mut Frame,
    ctx: &DrawContext<'_>,
    rect: Rect,
    envelope: Option<&Envelope>,
) {
    let Some(envelope) = envelope else {
        return;
    };
    let label_x = rect.x.saturating_add(LABEL_AT);
    let value_x = label_x.saturating_add(VALUE_OFFSET);
    let lines = ctx
        .config
        .reader
        .headers
        .iter()
        .filter_map(|name| header_line(ctx, name, envelope));
    for (offset, line) in lines.enumerate().take(usize::from(rect.height)) {
        let y = rect.y.saturating_add(u16::try_from(offset).unwrap_or(u16::MAX));
        put_cut(frame, rect, label_x, y, line.label, VALUE_OFFSET, ctx.fg(roles::READER_HEADER_LABEL));
        let room = rect.right().saturating_sub(value_x);
        put_cut(frame, rect, value_x, y, &line.value, room, ctx.fg(line.role));
    }
}

#[cfg(test)]
include!("reader_headers.tests.rs");
