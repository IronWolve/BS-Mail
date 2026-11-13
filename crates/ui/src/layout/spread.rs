//! Even spacing: places items left to right across a span so the free space
//! is shared out in equal gaps (the strip, the key bar and the footer).

/// Where each item starts and how wide each gap between items is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spread {
    /// Start of each item, counted from the left end of the span.
    pub starts: Vec<u16>,
    /// The gap after each item except the last; one fewer than the items.
    pub gaps: Vec<u16>,
}

/// Spreads items of the given widths over `available` cells. The first item
/// starts at 0 and the last ends at `available` when there are two or more;
/// gaps differ by at most one cell. `None` when the gaps would fall under
/// `min_gap`. A lone item starts at 0.
pub fn spread_items(widths: &[u16], available: u16, min_gap: u16) -> Option<Spread> {
    let total: u32 = widths.iter().map(|w| u32::from(*w)).sum();
    let free = u32::from(available).checked_sub(total)?;
    let count = widths.len();
    if count < 2 {
        return Some(Spread {
            starts: vec![0; count],
            gaps: Vec::new(),
        });
    }
    let slots = u32::try_from(count - 1).ok()?;
    if free / slots < u32::from(min_gap) {
        return None;
    }
    let gaps: Vec<u16> = (0..slots)
        .map(|i| (free * (i + 1) / slots - free * i / slots) as u16)
        .collect();
    let mut starts = Vec::with_capacity(count);
    let mut at = 0u16;
    for (index, width) in widths.iter().enumerate() {
        starts.push(at);
        at = at.saturating_add(*width);
        if let Some(gap) = gaps.get(index) {
            at = at.saturating_add(*gap);
        }
    }
    Some(Spread { starts, gaps })
}

/// Offset of the separator bar inside a gap of `gap` cells: the middle cell
/// (the right of the two middle cells when the gap is even).
pub fn bar_offset(gap: u16) -> u16 {
    gap / 2
}

#[cfg(test)]
include!("spread.tests.rs");
