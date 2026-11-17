//! The window icon: a PNG picture turned into the pixels the window system
//! takes. Only a plain 8-bit picture with transparency is accepted, and a
//! large one is refused, so a wrong file costs a missing icon and nothing more.

use winit::window::Icon;

/// The most pixels along a side that are read.
const MAX_SIDE: u32 = 512;

/// The icon in `bytes`, or `None` when they are not a PNG picture of 8-bit
/// colour with transparency that is at most `MAX_SIDE` pixels on a side.
pub fn from_png(bytes: &[u8]) -> Option<Icon> {
    let (rgba, width, height) = decode(bytes)?;
    Icon::from_rgba(rgba, width, height).ok()
}

/// The pixels of the picture as RGBA bytes, with its width and height.
pub fn decode(bytes: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().ok()?;
    let info = reader.info();
    let (width, height) = (info.width, info.height);
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    let mut pixels = vec![0u8; reader.output_buffer_size()?];
    let frame = reader.next_frame(&mut pixels).ok()?;
    pixels.truncate(frame.buffer_size());
    Some((pixels, width, height))
}

#[cfg(test)]
include!("icon.tests.rs");
