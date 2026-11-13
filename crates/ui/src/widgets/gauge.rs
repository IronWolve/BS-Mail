//! The storage gauge: a row of block characters for the filled part, and an
//! outline the window host draws around the whole bar.

use surface::{Effect, Frame, GaugeOutline, Rect};
use model::config::GaugeStyle;

use super::cell_text::put_text;
use super::draw_context::DrawContext;
use crate::icons::IconKind;
use crate::theme::roles;

/// A whole cell of fill.
const FULL_BLOCK: char = '\u{2588}';
/// Left blocks from one eighth up to seven eighths of a cell.
const PARTIAL_BLOCKS: [char; 7] = [
    '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}',
];
/// The empty part of a `bar` gauge.
const TRACK_BLOCK: char = '\u{2591}';
const ASCII_FILL: char = '#';
const ASCII_TRACK: char = '.';

/// The fill and track text of a gauge `width` cells wide filled to
/// `fraction` (0 to 1). Block sets fill in eighths of a cell; the ASCII set
/// rounds to whole cells. `track` is what the unfilled cells show.
fn gauge_text(width: u16, fraction: f64, kind: IconKind, track: Option<char>) -> (String, String) {
    let cells = usize::from(width);
    let fraction = if fraction.is_nan() { 0.0 } else { fraction.clamp(0.0, 1.0) };
    if kind == IconKind::Ascii {
        let filled = (fraction * cells as f64).round() as usize;
        let fill = ASCII_FILL.to_string().repeat(filled);
        let rest = track.map_or(' ', |_| ASCII_TRACK).to_string().repeat(cells - filled);
        return (fill, rest);
    }
    let eighths = (fraction * cells as f64 * 8.0).round() as usize;
    let (whole, part) = (eighths / 8, eighths % 8);
    let mut fill = FULL_BLOCK.to_string().repeat(whole);
    if part > 0 {
        fill.push(PARTIAL_BLOCKS[part - 1]);
    }
    let used = whole + usize::from(part > 0);
    let rest = track.unwrap_or(' ').to_string().repeat(cells - used);
    (fill, rest)
}

/// Draws the gauge in the one-row `rect`. `fraction` is `None` when the
/// server reports no storage limit: only the outline shows.
pub fn draw_gauge(frame: &mut Frame, ctx: &DrawContext<'_>, rect: Rect, fraction: Option<f64>) {
    if rect.is_empty() {
        return;
    }
    let appearance = &ctx.config.appearance;
    let outlined = appearance.gauge == GaugeStyle::Outlined;
    let track = (!outlined).then_some(TRACK_BLOCK);
    let (fill, rest) = gauge_text(rect.width, fraction.unwrap_or(0.0), ctx.icons.kind(), track);
    let after = put_text(frame, rect, rect.x, rect.y, &fill, ctx.fg(roles::GAUGE_FILL));
    put_text(frame, rect, after, rect.y, &rest, ctx.fg(roles::GAUGE_TRACK));
    if outlined {
        frame.push_effect(Effect::GaugeOutline(GaugeOutline {
            area: Rect::new(rect.x, rect.y, rect.width, 1),
            color: ctx.fg_color(roles::GAUGE_OUTLINE),
            radius: appearance.corner_radius,
            weight: appearance.line_weight,
        }));
    }
}

#[cfg(test)]
include!("gauge.tests.rs");
