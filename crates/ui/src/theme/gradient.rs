//! Color at a position along a list of gradient stops (the banner title).

use model::theme_schema::Rgb;

/// The color at `t` along `stops`, which are spread evenly from `t = 0` to
/// `t = 1`. `t` is clamped to that range. `None` only for an empty list.
pub fn color_at(stops: &[Rgb], t: f32) -> Option<Rgb> {
    let last = stops.len().checked_sub(1)?;
    if last == 0 {
        return Some(stops[0]);
    }
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    let scaled = t * last as f32;
    let index = (scaled.floor() as usize).min(last - 1);
    let fraction = scaled - index as f32;
    Some(blend(stops[index], stops[index + 1], fraction))
}

/// One color per character for text `count` characters long: the first gets
/// the first stop and the last gets the last stop. Empty for no stops.
pub fn spread(stops: &[Rgb], count: usize) -> Vec<Rgb> {
    (0..count)
        .filter_map(|i| {
            let t = if count > 1 {
                i as f32 / (count - 1) as f32
            } else {
                0.0
            };
            color_at(stops, t)
        })
        .collect()
}

fn blend(from: Rgb, to: Rgb, fraction: f32) -> Rgb {
    let channel = |a: u8, b: u8| {
        let (a, b) = (f32::from(a), f32::from(b));
        (a + (b - a) * fraction).round().clamp(0.0, 255.0) as u8
    };
    Rgb {
        r: channel(from.r, to.r),
        g: channel(from.g, to.g),
        b: channel(from.b, to.b),
    }
}

#[cfg(test)]
include!("gradient.tests.rs");
