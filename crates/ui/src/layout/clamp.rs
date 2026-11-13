//! Fits preferred sizes to the current grid. Clamping never changes the
//! stored preference, so widening the window brings the owner's size back.

/// Sidebar divider column limits (plan 4.4).
pub const SIDEBAR_MIN: u16 = 16;
pub const SIDEBAR_MAX: u16 = 60;
/// The list needs at least this many rows (title, rule, header, two rows).
pub const LIST_MIN_ROWS: u16 = 5;
/// The reader needs at least this many rows.
pub const READER_MIN_ROWS: u16 = 7;
/// Column limits when the reader sits right of the list.
pub const LIST_MIN_COLS: u16 = 40;
pub const READER_MIN_COLS: u16 = 30;
/// A sidebar section needs a title, a rule and one line.
pub const SECTION_MIN_ROWS: u16 = 3;

/// The divider column for a preferred width: 16 up to the smaller of 60 and
/// half the grid width.
pub fn sidebar_width(preferred: u16, grid_cols: u16) -> u16 {
    let upper = (grid_cols / 2).min(SIDEBAR_MAX).max(SIDEBAR_MIN);
    preferred.clamp(SIDEBAR_MIN, upper)
}

/// Splits `total` cells by a permille share, leaving one cell for the divider
/// between the parts. The first part is at least `first_min` and the second
/// at least `second_min`; `None` when the total is too small for both.
pub fn split_by_share(
    total: u16,
    permille: u16,
    first_min: u16,
    second_min: u16,
) -> Option<(u16, u16)> {
    let need = first_min.checked_add(1)?.checked_add(second_min)?;
    if total < need {
        return None;
    }
    let share = u32::from(permille.min(1000));
    let wanted = (u32::from(total) * share + 500) / 1000;
    let first = u16::try_from(wanted)
        .unwrap_or(u16::MAX)
        .clamp(first_min, total - 1 - second_min);
    Some((first, total - 1 - first))
}

/// List and reader rows for a reader below the list.
pub fn split_rows(main_rows: u16, permille: u16) -> Option<(u16, u16)> {
    split_by_share(main_rows, permille, LIST_MIN_ROWS, READER_MIN_ROWS)
}

/// List and reader columns for a reader right of the list.
pub fn split_cols(main_cols: u16, permille: u16) -> Option<(u16, u16)> {
    split_by_share(main_cols, permille, LIST_MIN_COLS, READER_MIN_COLS)
}

/// Rows for a section whose preference is `preferred` (0 = `content`),
/// never under the section minimum.
pub fn section_rows(preferred: u16, content: u16) -> u16 {
    let wanted = if preferred == 0 { content } else { preferred };
    wanted.max(SECTION_MIN_ROWS)
}

#[cfg(test)]
include!("clamp.tests.rs");
