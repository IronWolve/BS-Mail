//! The pixel buffer everything is drawn into.
//!
//! One opaque color per pixel, red green blue in the low three bytes, which
//! is what the window buffer and the image writer both want. Every drawing
//! call clips to the canvas, so no caller has to check bounds.

use super::colors::Rgb;

pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
}

impl Canvas {
    /// A canvas filled with `background`.
    pub fn new(width: u32, height: u32, background: Rgb) -> Self {
        let count = (width as usize) * (height as usize);
        Self {
            width,
            height,
            pixels: vec![background.packed(); count],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Repaints every pixel with `background`, keeping the size.
    pub fn clear(&mut self, background: Rgb) {
        self.pixels.fill(background.packed());
    }

    /// Resizes and clears, reusing the memory when the size is unchanged.
    pub fn resize(&mut self, width: u32, height: u32, background: Rgb) {
        let count = (width as usize) * (height as usize);
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.pixels.resize(count, background.packed());
    }

    pub fn pixel(&self, x: i32, y: i32) -> Option<Rgb> {
        let index = self.index(x, y)?;
        Some(Rgb::unpack(self.pixels[index]))
    }

    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = color.packed();
        }
    }

    /// `color` over the pixel at `alpha` of 255.
    pub fn blend(&mut self, x: i32, y: i32, color: Rgb, alpha: u8) {
        match alpha {
            0 => {}
            255 => self.set(x, y, color),
            _ => {
                if let Some(index) = self.index(x, y) {
                    let under = Rgb::unpack(self.pixels[index]);
                    self.pixels[index] = color.mix(under, alpha).packed();
                }
            }
        }
    }

    /// A solid rectangle, clipped to the canvas.
    pub fn fill(&mut self, x: i32, y: i32, width: u32, height: u32, color: Rgb) {
        let packed = color.packed();
        for row in self.rows(y, height) {
            let (start, end) = self.span(x, width);
            if start < end {
                let base = row * self.width as usize;
                self.pixels[base + start..base + end].fill(packed);
            }
        }
    }

    /// The rows of a rectangle that are on the canvas.
    fn rows(&self, y: i32, height: u32) -> std::ops::Range<usize> {
        let top = y.max(0) as usize;
        let bottom = (y as i64 + i64::from(height)).clamp(0, i64::from(self.height)) as usize;
        top.min(bottom)..bottom
    }

    /// The columns of a rectangle that are on the canvas.
    fn span(&self, x: i32, width: u32) -> (usize, usize) {
        let left = x.max(0) as usize;
        let right = (x as i64 + i64::from(width)).clamp(0, i64::from(self.width)) as usize;
        (left.min(right), right)
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return None;
        }
        Some((y as usize) * (self.width as usize) + x as usize)
    }

    /// Three bytes per pixel, for the image writer.
    pub fn rgb_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pixels.len() * 3);
        for pixel in &self.pixels {
            bytes.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8]);
        }
        bytes
    }
}

#[cfg(test)]
include!("canvas.tests.rs");
