//! The cell grid onto the canvas.
//!
//! Backgrounds are laid down for the whole grid first and the symbols drawn
//! over them, so a letter that leans or an icon that fills its slot is never
//! clipped by the cell painted after it.

use surface::{Color, Frame, Modifier, Rect};
use unicode_width::UnicodeWidthStr;

use crate::font::{FaceKind, choose, is_icon};

use super::block_elements::{block_glyph, draw_block};
use super::box_drawing::{box_glyph, draw_box};
use super::canvas::Canvas;
use super::colors::{Ink, Palette, Rgb};
use super::context::PaintContext;
use super::effects::CellChanges;
use super::glyph_draw::draw_glyph;
use super::grid::PixelRect;
use super::icon_fit::draw_icon;

/// Cells an icon is given, whatever its own width.
const ICON_CELLS: u16 = 2;
/// How far dimmed text is faded toward its background, out of 255.
const DIM: u8 = 150;

/// What one cell is drawn with, after the style's modifiers are applied.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CellStyle {
    fg: Rgb,
    bg: Rgb,
    face: FaceKind,
    hidden: bool,
    underlined: bool,
    crossed_out: bool,
}

/// One cell's style, before the palette turns it into pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StyleParts {
    fg: Color,
    bg: Color,
    modifier: Modifier,
}

/// Every cell's background color.
pub fn paint_backgrounds(canvas: &mut Canvas, context: &mut PaintContext<'_>, frame: &Frame) {
    let area = frame.area();
    for row in area.top()..area.bottom() {
        for col in area.left()..area.right() {
            let Some(cell) = frame.cells.cell((col, row)) else {
                continue;
            };
            let style = resolve(&parts(cell.fg, cell.bg, cell.modifier), &context.palette);
            let rect = context.grid.cell(col, row);
            canvas.fill(rect.x, rect.y, rect.width, rect.height, style.bg);
        }
    }
}

/// Every cell's symbol, with what the frame's effects change about them:
/// cells an effect replaces are left out, and cells it lowers sit half a row
/// down so they line up with double-size text beside them.
pub fn paint_symbols(
    canvas: &mut Canvas,
    context: &mut PaintContext<'_>,
    frame: &Frame,
    changes: &CellChanges,
) {
    let area = frame.area();
    let stroke = context.frame_stroke(frame.line_weight);
    for row in area.top()..area.bottom() {
        for col in area.left()..area.right() {
            if changes.replaced.iter().any(|area| contains(*area, col, row)) {
                continue;
            }
            let shifted = changes.shifts.iter().any(|area| contains(*area, col, row));
            let plan = CellPlan {
                offset: if shifted {
                    context.grid.metrics.height as i32 / 2
                } else {
                    0
                },
                inside: frame_inside(context, changes, col, row),
                stroke,
            };
            paint_symbol(canvas, context, frame, (col, row), plan);
        }
    }
}

/// What the effects and the frame's line weight do to one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CellPlan {
    /// Pixels to lower this cell by.
    offset: i32,
    /// The inside of an outline the cell sits in, which a bar stays within.
    inside: Option<PixelRect>,
    /// How thick the lines of a border character are, in pixels.
    stroke: u32,
}

fn contains(area: Rect, col: u16, row: u16) -> bool {
    col >= area.x && row >= area.y && col < area.right() && row < area.bottom()
}

/// The inside of the outline around this cell, when there is one.
fn frame_inside(
    context: &PaintContext<'_>,
    changes: &CellChanges,
    col: u16,
    row: u16,
) -> Option<PixelRect> {
    let (area, weight) = changes
        .framed
        .iter()
        .find(|(area, _)| contains(*area, col, row))?;
    let weight = u8::try_from(*weight).unwrap_or(u8::MAX);
    context.grid.area(*area).shrink(context.line_pixels(weight) + 1)
}

/// The symbol of one cell, and the lines its style asks for.
fn paint_symbol(
    canvas: &mut Canvas,
    context: &mut PaintContext<'_>,
    frame: &Frame,
    (col, row): (u16, u16),
    plan: CellPlan,
) {
    let Some(cell) = frame.cells.cell((col, row)) else {
        return;
    };
    let style = resolve(&parts(cell.fg, cell.bg, cell.modifier), &context.palette);
    if style.hidden {
        return;
    }
    let symbol = cell.symbol();
    let Some(ch) = symbol.chars().next() else {
        return;
    };
    let cells = span(symbol, ch);
    let mut slot = context.grid.slot(col, row, cells);
    slot.y += plan.offset;
    draw_lines(canvas, context, slot, &style);
    if ch == ' ' {
        return;
    }
    let baseline = context.grid.baseline(row) + plan.offset;
    draw_shape_or_glyph(canvas, context, (ch, slot, baseline), (&style, plan));
}

/// A drawn shape where the character is one, otherwise a glyph from a face.
fn draw_shape_or_glyph(
    canvas: &mut Canvas,
    context: &mut PaintContext<'_>,
    (ch, slot, baseline): (char, PixelRect, i32),
    (style, plan): (&CellStyle, CellPlan),
) {
    if let Some(glyph) = box_glyph(ch) {
        draw_box(canvas, slot, plan.stroke, glyph, style.fg);
        return;
    }
    if let Some(block) = block_glyph(ch) {
        let area = match plan.inside {
            Some(inside) => slot.intersect(&inside),
            None => Some(slot),
        };
        if let Some(area) = area {
            draw_block(canvas, area, block, style.fg);
        }
        return;
    }
    let Some(glyph) = choose(context.fonts, style.face, ch) else {
        draw_missing(canvas, context, slot, style.fg);
        return;
    };
    if is_icon(ch) {
        draw_icon(canvas, context, slot, glyph, style.fg);
        return;
    }
    let size = context.pixel_size();
    if let Some(bitmap) = context.cache.glyph(context.fonts, glyph.face, glyph.id, size) {
        draw_glyph(canvas, bitmap, (slot.x, baseline), style.fg, slot.grow(1));
    }
}

/// A character no face has: a hollow box of the cell's width.
fn draw_missing(canvas: &mut Canvas, context: &mut PaintContext<'_>, slot: PixelRect, color: Rgb) {
    let stroke = context.grid.metrics.stroke;
    let inset = slot.height / 6;
    let rect = PixelRect::new(
        slot.x + stroke as i32,
        slot.y + inset as i32,
        slot.width.saturating_sub(stroke * 2),
        slot.height.saturating_sub(inset * 2),
    );
    canvas.fill(rect.x, rect.y, rect.width, stroke, color);
    canvas.fill(rect.x, rect.bottom() - stroke as i32, rect.width, stroke, color);
    canvas.fill(rect.x, rect.y, stroke, rect.height, color);
    canvas.fill(rect.right() - stroke as i32, rect.y, stroke, rect.height, color);
}

/// The underline and the strikethrough.
fn draw_lines(canvas: &mut Canvas, context: &PaintContext<'_>, rect: PixelRect, style: &CellStyle) {
    let metrics = &context.grid.metrics;
    if style.underlined {
        canvas.fill(rect.x, rect.y + metrics.underline, rect.width, metrics.stroke, style.fg);
    }
    if style.crossed_out {
        let middle = rect.y + metrics.baseline - (metrics.height as i32) / 5;
        canvas.fill(rect.x, middle, rect.width, metrics.stroke, style.fg);
    }
}

/// Cells this symbol occupies; an icon always gets its slot.
fn span(symbol: &str, ch: char) -> u16 {
    if is_icon(ch) {
        return ICON_CELLS;
    }
    u16::try_from(UnicodeWidthStr::width(symbol)).unwrap_or(1).max(1)
}

fn parts(fg: Color, bg: Color, modifier: Modifier) -> StyleParts {
    StyleParts { fg, bg, modifier }
}

/// Turns a style into pixels: reverse swaps the two, dim fades the text
/// toward its own background.
fn resolve(parts: &StyleParts, palette: &Palette) -> CellStyle {
    let reversed = parts.modifier.contains(Modifier::REVERSED);
    let (fg_color, bg_color) = if reversed {
        (parts.bg, parts.fg)
    } else {
        (parts.fg, parts.bg)
    };
    let bg = palette.resolve(bg_color, Ink::Background);
    let mut fg = palette.resolve(fg_color, Ink::Foreground);
    if parts.modifier.contains(Modifier::DIM) {
        fg = fg.mix(bg, DIM);
    }
    CellStyle {
        fg,
        bg,
        face: FaceKind::text(
            parts.modifier.contains(Modifier::BOLD),
            parts.modifier.contains(Modifier::ITALIC),
        ),
        hidden: parts.modifier.contains(Modifier::HIDDEN),
        underlined: parts.modifier.contains(Modifier::UNDERLINED),
        crossed_out: parts.modifier.contains(Modifier::CROSSED_OUT),
    }
}

#[cfg(test)]
include!("cell_painter.tests.rs");
