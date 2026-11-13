//! Frame lines drawn as shapes instead of glyphs.
//!
//! Every arm runs from the cell's edge to the cell's center line, so lines
//! join exactly at any cell size and no font decides where a border sits.
//! Rounded corners are a true arc.

use tiny_skia::{LineCap, PathBuilder, Stroke, Transform};

use super::canvas::Canvas;
use super::colors::Rgb;
use super::grid::PixelRect;
use super::shapes::{solid, with_bitmap};

/// How thick one arm is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    None,
    Light,
    Heavy,
    Double,
}

impl Weight {
    fn is_drawn(self) -> bool {
        self != Self::None
    }

    /// Thickness and the offsets from the center line, in pixels.
    fn lines(self, stroke: u32) -> Vec<(i32, u32)> {
        match self {
            Self::None => Vec::new(),
            Self::Light => vec![(0, stroke)],
            Self::Heavy => vec![(0, (stroke * 2).max(stroke + 1))],
            Self::Double => {
                let gap = stroke.max(1) as i32;
                vec![(-gap, stroke), (gap, stroke)]
            }
        }
    }
}

/// The four arms of one box-drawing character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxGlyph {
    pub up: Weight,
    pub right: Weight,
    pub down: Weight,
    pub left: Weight,
    /// A corner drawn as an arc rather than a square meeting of two arms.
    pub rounded: bool,
}

impl BoxGlyph {
    const fn new(up: Weight, right: Weight, down: Weight, left: Weight) -> Self {
        Self {
            up,
            right,
            down,
            left,
            rounded: false,
        }
    }

    const fn rounded(self) -> Self {
        Self {
            rounded: true,
            ..self
        }
    }

    /// The heaviest arm, which sets the thickness of a corner's arc.
    fn weight(&self) -> Weight {
        [self.up, self.right, self.down, self.left]
            .into_iter()
            .find(|weight| weight.is_drawn())
            .unwrap_or(Weight::Light)
    }
}

/// Draws `glyph` into one cell.
pub fn draw_box(canvas: &mut Canvas, cell: PixelRect, stroke: u32, glyph: BoxGlyph, color: Rgb) {
    if glyph.rounded {
        draw_corner_arc(canvas, cell, stroke, glyph, color);
        return;
    }
    let center_x = cell.x + cell.width as i32 / 2;
    let center_y = cell.y + cell.height as i32 / 2;
    for (offset, thickness) in glyph.left.lines(stroke) {
        let top = center_y + offset - thickness as i32 / 2;
        let right = center_x + (thickness as i32 + 1) / 2;
        fill_span(canvas, cell.x, top, (right - cell.x) as u32, thickness, color);
    }
    for (offset, thickness) in glyph.right.lines(stroke) {
        let top = center_y + offset - thickness as i32 / 2;
        let left = center_x - thickness as i32 / 2;
        fill_span(canvas, left, top, (cell.right() - left) as u32, thickness, color);
    }
    for (offset, thickness) in glyph.up.lines(stroke) {
        let left = center_x + offset - thickness as i32 / 2;
        let bottom = center_y + (thickness as i32 + 1) / 2;
        fill_span(canvas, left, cell.y, thickness, (bottom - cell.y) as u32, color);
    }
    for (offset, thickness) in glyph.down.lines(stroke) {
        let left = center_x + offset - thickness as i32 / 2;
        let top = center_y - thickness as i32 / 2;
        fill_span(canvas, left, top, thickness, (cell.bottom() - top) as u32, color);
    }
}

/// One arm, if it has any size at all.
fn fill_span(canvas: &mut Canvas, x: i32, y: i32, width: u32, height: u32, color: Rgb) {
    if width > 0 && height > 0 {
        canvas.fill(x, y, width, height, color);
    }
}

/// A corner as a quarter arc between its two arms.
fn draw_corner_arc(canvas: &mut Canvas, cell: PixelRect, stroke: u32, glyph: BoxGlyph, color: Rgb) {
    let thickness = glyph.weight().lines(stroke).first().map_or(stroke, |line| line.1);
    let (width, height) = (cell.width as f32, cell.height as f32);
    let (center_x, center_y) = (width / 2.0, height / 2.0);
    let radius = center_x.min(center_y);
    let step_x = if glyph.right.is_drawn() { 1.0 } else { -1.0 };
    let step_y = if glyph.down.is_drawn() { 1.0 } else { -1.0 };
    let end_x = if glyph.right.is_drawn() { width } else { 0.0 };
    let end_y = if glyph.down.is_drawn() { height } else { 0.0 };
    let mut path = PathBuilder::new();
    path.move_to(end_x, center_y);
    path.line_to(center_x + step_x * radius, center_y);
    path.quad_to(center_x, center_y, center_x, center_y + step_y * radius);
    path.line_to(center_x, end_y);
    let Some(path) = path.finish() else {
        return;
    };
    let stroke_style = Stroke {
        width: thickness as f32,
        line_cap: LineCap::Butt,
        ..Stroke::default()
    };
    with_bitmap(canvas, cell, |bitmap| {
        bitmap.stroke_path(&path, &solid(color), &stroke_style, Transform::identity(), None);
    });
}

/// The arms of a box-drawing character, or `None` when it is not one.
pub fn box_glyph(ch: char) -> Option<BoxGlyph> {
    use Weight::{Double as D, Heavy as H, Light as L, None as N};
    let glyph = match ch {
        '\u{2500}' => BoxGlyph::new(N, L, N, L),
        '\u{2501}' => BoxGlyph::new(N, H, N, H),
        '\u{2502}' => BoxGlyph::new(L, N, L, N),
        '\u{2503}' => BoxGlyph::new(H, N, H, N),
        '\u{250c}' => BoxGlyph::new(N, L, L, N),
        '\u{250f}' => BoxGlyph::new(N, H, H, N),
        '\u{2510}' => BoxGlyph::new(N, N, L, L),
        '\u{2513}' => BoxGlyph::new(N, N, H, H),
        '\u{2514}' => BoxGlyph::new(L, L, N, N),
        '\u{2517}' => BoxGlyph::new(H, H, N, N),
        '\u{2518}' => BoxGlyph::new(L, N, N, L),
        '\u{251b}' => BoxGlyph::new(H, N, N, H),
        '\u{251c}' => BoxGlyph::new(L, L, L, N),
        '\u{2523}' => BoxGlyph::new(H, H, H, N),
        '\u{2524}' => BoxGlyph::new(L, N, L, L),
        '\u{252b}' => BoxGlyph::new(H, N, H, H),
        '\u{252c}' => BoxGlyph::new(N, L, L, L),
        '\u{2533}' => BoxGlyph::new(N, H, H, H),
        '\u{2534}' => BoxGlyph::new(L, L, N, L),
        '\u{253b}' => BoxGlyph::new(H, H, N, H),
        '\u{253c}' => BoxGlyph::new(L, L, L, L),
        '\u{254b}' => BoxGlyph::new(H, H, H, H),
        '\u{2550}' => BoxGlyph::new(N, D, N, D),
        '\u{2551}' => BoxGlyph::new(D, N, D, N),
        '\u{2554}' => BoxGlyph::new(N, D, D, N),
        '\u{2557}' => BoxGlyph::new(N, N, D, D),
        '\u{255a}' => BoxGlyph::new(D, D, N, N),
        '\u{255d}' => BoxGlyph::new(D, N, N, D),
        '\u{2560}' => BoxGlyph::new(D, D, D, N),
        '\u{2563}' => BoxGlyph::new(D, N, D, D),
        '\u{2566}' => BoxGlyph::new(N, D, D, D),
        '\u{2569}' => BoxGlyph::new(D, D, N, D),
        '\u{256c}' => BoxGlyph::new(D, D, D, D),
        '\u{256d}' => BoxGlyph::new(N, L, L, N).rounded(),
        '\u{256e}' => BoxGlyph::new(N, N, L, L).rounded(),
        '\u{256f}' => BoxGlyph::new(L, N, N, L).rounded(),
        '\u{2570}' => BoxGlyph::new(L, L, N, N).rounded(),
        _ => return None,
    };
    Some(glyph)
}

#[cfg(test)]
include!("box_drawing.tests.rs");
