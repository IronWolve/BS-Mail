//! The effects of a frame, in two passes.
//!
//! A rounded fill is a background and goes under the cells' text; scaled text
//! and a gauge outline go over it. A half-row shift moves the cells
//! themselves, so it is not drawn here at all: the cell painter is told which
//! areas to lower.

use surface::{Effect, Frame, Rect};

use super::canvas::Canvas;
use super::context::PaintContext;
use super::rounded::{draw_gauge_outline, draw_rounded_fill};
use super::scaled_text::draw_scaled_text;

/// Effects that belong behind the cells' symbols.
pub fn paint_under(canvas: &mut Canvas, context: &mut PaintContext<'_>, frame: &Frame) {
    for effect in &frame.effects {
        if let Effect::RoundedFill(fill) = effect {
            draw_rounded_fill(canvas, context, fill);
        }
    }
}

/// Effects that belong over the cells' symbols.
pub fn paint_over(canvas: &mut Canvas, context: &mut PaintContext<'_>, frame: &Frame) {
    for effect in &frame.effects {
        match effect {
            Effect::ScaledText(text) => draw_scaled_text(canvas, context, text),
            Effect::GaugeOutline(outline) => draw_gauge_outline(canvas, context, outline),
            Effect::RoundedFill(_) | Effect::HalfRowShift(_) => {}
        }
    }
}

/// What the effects of a frame change about the cells themselves.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CellChanges {
    /// Drawn half a row lower, to sit on a double-size line's center.
    pub shifts: Vec<Rect>,
    /// Not drawn at all: an effect draws these cells instead. The cells hold
    /// the same text at normal size for a host that cannot draw the effect,
    /// and drawing both would print one over the other.
    pub replaced: Vec<Rect>,
    /// Inside an outline, with the pixels that outline needs. A bar of block
    /// characters fills a whole cell, which is right in a terminal; here it
    /// is kept inside the frame drawn around it.
    pub framed: Vec<(Rect, u32)>,
}

pub fn cell_changes(frame: &Frame) -> CellChanges {
    let mut changes = CellChanges::default();
    for effect in &frame.effects {
        match effect {
            Effect::HalfRowShift(shift) => changes.shifts.push(shift.area),
            Effect::ScaledText(text) => changes.replaced.push(text.area),
            Effect::GaugeOutline(outline) => {
                changes.framed.push((outline.area, u32::from(outline.weight.max(1))));
            }
            Effect::RoundedFill(_) => {}
        }
    }
    changes
}

#[cfg(test)]
include!("effects.tests.rs");
