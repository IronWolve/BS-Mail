//! The size of the terminal the program was started from, in cells.

/// Columns and rows of the terminal, or `None` when the output is not a
/// terminal (a pipe or a file) and the size cannot be asked for.
pub fn terminal_size() -> Option<(u16, u16)> {
    crossterm::terminal::size()
        .ok()
        .filter(|(cols, rows)| *cols > 0 && *rows > 0)
}
