//! How far each scrollable region is scrolled, and the rule that keeps a
//! selected row in view.

/// First visible row of each scrollable region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollState {
    pub folders: usize,
    pub list: usize,
    pub reader: usize,
}

/// The first row to show so that row `selected` is visible in a window of
/// `visible` rows, moving `first` as little as possible. `total` bounds the
/// result; a window larger than the content shows everything from row 0.
pub fn first_row_showing(first: usize, selected: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total == 0 {
        return 0;
    }
    let last_start = total.saturating_sub(visible);
    let moved = if selected < first {
        selected
    } else if selected >= first + visible {
        selected + 1 - visible
    } else {
        first
    };
    moved.min(last_start)
}

/// Like `first_row_showing` for rows at uneven offsets from the top (the
/// folder tree with blank rows between entries): the first row to show so
/// that row `selected`, which starts `offsets[selected]` rows down, ends
/// inside a window `height` rows high.
pub fn first_row_at_offsets(offsets: &[u16], first: usize, selected: usize, height: u16) -> usize {
    let last = offsets.len().saturating_sub(1);
    let selected = selected.min(last);
    let mut first = first.min(last);
    if selected < first {
        return selected;
    }
    while first < selected && offsets[selected] - offsets[first] >= height {
        first += 1;
    }
    first
}

#[cfg(test)]
include!("scroll.tests.rs");
