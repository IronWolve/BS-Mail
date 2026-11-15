//! Rounded shapes: the fill behind a keycap and the outline around a gauge.
//!
//! Both are true arcs at any cell size, which a cell of text cannot give.

use tiny_skia::{FillRule, LineCap, Path, PathBuilder, Stroke, Transform};
use surface::{GaugeOutline, RoundedFill};

use super::canvas::Canvas;
use super::colors::Ink;
use super::context::PaintContext;
use super::shapes::{solid, with_bitmap};

/// The rounded background behind a run of cells.
pub fn draw_rounded_fill(canvas: &mut Canvas, context: &PaintContext<'_>, effect: &RoundedFill) {
    let rect = context.grid.area(effect.area);
    let radius = effect.radius * context.grid.metrics.width as f32;
    let Some(path) = rounded_rect(rect.width as f32, rect.height as f32, radius) else {
        return;
    };
    let color = context.palette.resolve(effect.color, Ink::Background);
    with_bitmap(canvas, rect, |bitmap| {
        bitmap.fill_path(&path, &solid(color), FillRule::Winding, Transform::identity(), None);
    });
}

/// The rounded outline around a one-row bar. Its weight is given at scale
/// factor 1, so it thickens with the display's scale like the frame lines.
pub fn draw_gauge_outline(canvas: &mut Canvas, context: &PaintContext<'_>, effect: &GaugeOutline) {
    let rect = context.grid.area(effect.area);
    let weight = context.line_pixels(effect.weight) as f32;
    // The stroke straddles the path, so it is inset by half its weight to
    // stay inside the cells the gauge was given.
    let inset = weight / 2.0;
    let width = rect.width as f32 - weight;
    let height = rect.height as f32 - weight;
    let radius = effect.radius * context.grid.metrics.width as f32;
    let Some(path) = rounded_rect(width, height, radius) else {
        return;
    };
    let color = context.palette.resolve(effect.color, Ink::Foreground);
    let stroke = Stroke {
        width: weight,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    with_bitmap(canvas, rect, |bitmap| {
        let shift = Transform::from_translate(inset, inset);
        bitmap.stroke_path(&path, &solid(color), &stroke, shift, None);
    });
}

/// A rectangle of `width` x `height` with corners of `radius`, starting at
/// the origin. The radius is cut back when the rectangle is too small for it.
fn rounded_rect(width: f32, height: f32, radius: f32) -> Option<Path> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let radius = radius.clamp(0.0, width.min(height) / 2.0);
    let mut path = PathBuilder::new();
    path.move_to(radius, 0.0);
    path.line_to(width - radius, 0.0);
    path.quad_to(width, 0.0, width, radius);
    path.line_to(width, height - radius);
    path.quad_to(width, height, width - radius, height);
    path.line_to(radius, height);
    path.quad_to(0.0, height, 0.0, height - radius);
    path.line_to(0.0, radius);
    path.quad_to(0.0, 0.0, radius, 0.0);
    path.close();
    path.finish()
}

#[cfg(test)]
include!("rounded.tests.rs");
