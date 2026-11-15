//! `compute_layout`: one pure function from preferences, options and the grid
//! size to a rectangle for every region and the lines between them.

use surface::Rect;

use super::chrome_rows::{ChromeRows, ChromeShape, chrome_rows};
use super::clamp::sidebar_width;
use super::main_split::{Divider, list_region, reader_region, split_main};
use super::options::LayoutOptions;
use super::prefs::LayoutPrefs;
use super::regions::{GridSize, Layout, LineKind, LineSpec};
use super::sidebar_layout::{SidebarInput, layout_sidebar};

/// Computes the layout of a grid. A grid under 40 x 16 gets
/// `Layout::too_small` and the caller draws one message line.
pub fn compute_layout(prefs: &LayoutPrefs, options: &LayoutOptions, grid: GridSize) -> Layout {
    if !grid.is_usable() {
        return Layout::too_small(grid);
    }
    let shape = ChromeShape {
        banner_rows: options.banner_rows,
        strip: options.strip,
        key_bar: options.key_bar,
        footer: options.footer,
    };
    let rows = chrome_rows(&shape, grid.rows);
    let mut layout = Layout::empty(grid);
    layout.lines = frame_lines(grid, &rows);
    place_bands(&mut layout, &rows);
    place_main(&mut layout, &rows, prefs, options);
    layout
}

/// Borders and the rules between bands, as frame lines.
fn frame_lines(grid: GridSize, rows: &ChromeRows) -> Vec<LineSpec> {
    let last_col = grid.cols - 1;
    let mut lines = vec![LineSpec::horizontal(LineKind::Frame, 0, 0, last_col)];
    for rule in &rows.rules {
        lines.push(LineSpec::horizontal(LineKind::Frame, *rule, 0, last_col));
    }
    lines.push(LineSpec::horizontal(
        LineKind::Frame,
        rows.bottom_border,
        0,
        last_col,
    ));
    for col in [0, last_col] {
        lines.push(LineSpec::vertical(
            LineKind::Frame,
            col,
            0,
            rows.bottom_border,
        ));
    }
    lines
}

/// The banner, strip, key bar and footer span the inside of the frame.
fn place_bands(layout: &mut Layout, rows: &ChromeRows) {
    let width = layout.grid.cols - 2;
    let band = |y: u16, count: u16| Rect::new(1, y, width, count);
    layout.banner = rows.banner.map(|span| band(span.y, span.rows));
    layout.strip = rows.strip.map(|y| band(y, 1));
    layout.key_bar = rows.key_bar.map(|y| band(y, 1));
    layout.footer = rows.footer.map(|y| band(y, 1));
}

fn place_main(
    layout: &mut Layout,
    rows: &ChromeRows,
    prefs: &LayoutPrefs,
    options: &LayoutOptions,
) {
    let cols = layout.grid.cols;
    let (top_line, bottom_line) = (rows.main_top_line(), rows.main_bottom_line());
    let sidebar_on = options.sidebar && !options.sections.is_empty();
    let divider_col = sidebar_on.then(|| sidebar_width(prefs.sidebar_width, cols));
    let main_x = divider_col.map_or(1, |col| col + 1);
    layout.main = Rect::new(main_x, rows.main.y, cols - 1 - main_x, rows.main.rows);
    if let Some(col) = divider_col {
        layout
            .lines
            .push(LineSpec::vertical(LineKind::Frame, col, top_line, bottom_line));
        let area = Rect::new(1, rows.main.y, col - 1, rows.main.rows);
        place_sidebar(layout, area, prefs, options);
    }
    place_panes(layout, divider_col, (top_line, bottom_line), prefs, options);
}

fn place_sidebar(layout: &mut Layout, area: Rect, prefs: &LayoutPrefs, options: &LayoutOptions) {
    let input = SidebarInput {
        sections: options.sections.clone(),
        density: options.density,
        content: options.content,
        accounts_rows: prefs.accounts_rows,
        status_rows: prefs.status_rows,
    };
    let sidebar = layout_sidebar(area, &input);
    for section in &sidebar.sections {
        layout.lines.extend(section.rule.map(LineSpec::rule_in));
    }
    layout
        .lines
        .extend(sidebar.dividers.iter().copied().map(LineSpec::rule_in));
    layout.sidebar = Some(sidebar);
}

/// The list and the reader, the divider between them, and their rules.
fn place_panes(
    layout: &mut Layout,
    sidebar_divider: Option<u16>,
    (top_line, bottom_line): (u16, u16),
    prefs: &LayoutPrefs,
    options: &LayoutOptions,
) {
    let last_col = layout.grid.cols - 1;
    let split = split_main(layout.main, options.reader, prefs.list_share);
    match split.divider {
        Some(Divider::Row(row)) => layout.lines.push(LineSpec::horizontal(
            LineKind::Frame,
            row,
            sidebar_divider.unwrap_or(0),
            last_col,
        )),
        Some(Divider::Column(col)) => layout
            .lines
            .push(LineSpec::vertical(LineKind::Frame, col, top_line, bottom_line)),
        None => {}
    }
    if split.reader.is_none() && options.reader_full {
        // No room beside the list: the message covers it.
        place_reader(layout, split.list, options);
        return;
    }
    let list = list_region(split.list, &options.columns, &prefs.widths);
    layout.lines.extend(list.rule.map(LineSpec::rule_in));
    layout.list = Some(list);
    if let Some(rect) = split.reader {
        place_reader(layout, rect, options);
    }
}

fn place_reader(layout: &mut Layout, rect: Rect, options: &LayoutOptions) {
    let reader = reader_region(rect, options.reader_header_rows);
    layout.lines.extend(reader.rule.map(LineSpec::rule_in));
    layout.lines.extend(reader.header_rule.map(LineSpec::rule_in));
    layout.reader = Some(reader);
}

#[cfg(test)]
include!("compute.tests.rs");
