//! Coverage of a run of glyphs, kept apart from the canvas.
//!
//! Scaled text is stamped into a mask first so the same shape can be blurred
//! for the glow and then colored for the text, each pixel once.

use crate::font::{BitmapContent, GlyphBitmap};

use super::grid::PixelRect;

pub struct TextMask {
    pub rect: PixelRect,
    data: Vec<u8>,
}

impl TextMask {
    pub fn new(rect: PixelRect) -> Self {
        let count = (rect.width as usize) * (rect.height as usize);
        Self {
            rect,
            data: vec![0; count],
        }
    }

    /// Adds one glyph, keeping the stronger coverage where shapes overlap.
    pub fn stamp(&mut self, bitmap: &GlyphBitmap, pen: (i32, i32)) {
        if bitmap.content != BitmapContent::Mask {
            return;
        }
        let left = pen.0 + bitmap.left;
        let top = pen.1 - bitmap.top;
        for y in 0..bitmap.height {
            for x in 0..bitmap.width {
                let coverage = bitmap.coverage(x, y);
                if coverage == 0 {
                    continue;
                }
                let canvas_x = left + x as i32;
                let canvas_y = top + y as i32;
                if let Some(index) = self.index(canvas_x, canvas_y) {
                    self.data[index] = self.data[index].max(coverage);
                }
            }
        }
    }

    /// Coverage at a canvas pixel, 0 outside the mask.
    pub fn coverage(&self, x: i32, y: i32) -> u8 {
        self.index(x, y).map_or(0, |index| self.data[index])
    }

    pub fn is_blank(&self) -> bool {
        self.data.iter().all(|value| *value == 0)
    }

    /// The same mask softened, for a glow. Three box passes read close
    /// enough to a true blur and cost one pass each way.
    pub fn blurred(&self, radius: u8) -> Self {
        let mut blurred = Self {
            rect: self.rect,
            data: self.data.clone(),
        };
        if radius == 0 || self.rect.width == 0 || self.rect.height == 0 {
            return blurred;
        }
        for _ in 0..3 {
            blurred.data = box_pass(&blurred.data, self.rect.width, self.rect.height, radius);
            blurred.data = transpose(&blurred.data, self.rect.width, self.rect.height);
            blurred.data = box_pass(&blurred.data, self.rect.height, self.rect.width, radius);
            blurred.data = transpose(&blurred.data, self.rect.height, self.rect.width);
        }
        blurred
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if !self.rect.contains(x, y) {
            return None;
        }
        let x = (x - self.rect.x) as usize;
        let y = (y - self.rect.y) as usize;
        Some(y * self.rect.width as usize + x)
    }
}

/// One horizontal averaging pass.
fn box_pass(data: &[u8], width: u32, height: u32, radius: u8) -> Vec<u8> {
    let radius = i64::from(radius);
    let width = width as i64;
    let mut out = vec![0u8; data.len()];
    for row in 0..height as i64 {
        let base = (row * width) as usize;
        for x in 0..width {
            let mut total = 0u32;
            let mut count = 0u32;
            for offset in -radius..=radius {
                let sample = x + offset;
                if sample >= 0 && sample < width {
                    total += u32::from(data[base + sample as usize]);
                    count += 1;
                }
            }
            out[base + x as usize] = (total / count.max(1)) as u8;
        }
    }
    out
}

/// Rows become columns, so the same pass blurs the other way.
fn transpose(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    for y in 0..height as usize {
        for x in 0..width as usize {
            out[x * height as usize + y] = data[y * width as usize + x];
        }
    }
    out
}

#[cfg(test)]
include!("text_mask.tests.rs");
