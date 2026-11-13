//! The fixed rows of the screen: borders, bands (banner, strip, key bar,
//! footer), the rules between them, and what is left for the main area.

/// Which bands are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChromeShape {
    /// Rows of the banner band; 0 hides it.
    pub banner_rows: u16,
    pub strip: bool,
    pub key_bar: bool,
    pub footer: bool,
}

/// A run of rows: the first row and how many.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowSpan {
    pub y: u16,
    pub rows: u16,
}

/// Where each band sits. Row 0 is the top border; every band is followed by
/// a rule except the last, which is followed by the bottom border.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeRows {
    pub banner: Option<RowSpan>,
    pub strip: Option<u16>,
    pub main: RowSpan,
    pub key_bar: Option<u16>,
    pub footer: Option<u16>,
    /// The rule rows between bands, top to bottom.
    pub rules: Vec<u16>,
    pub bottom_border: u16,
}

impl ChromeRows {
    /// The line above the main area: a rule, or the top border when no band
    /// sits above it.
    pub fn main_top_line(&self) -> u16 {
        self.main.y.saturating_sub(1)
    }

    /// The line below the main area: a rule, or the bottom border.
    pub fn main_bottom_line(&self) -> u16 {
        self.main.y.saturating_add(self.main.rows)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Band {
    Banner,
    Strip,
    Main,
    KeyBar,
    Footer,
}

fn bands(shape: &ChromeShape) -> Vec<(Band, u16)> {
    let mut list = Vec::with_capacity(5);
    if shape.banner_rows > 0 {
        list.push((Band::Banner, shape.banner_rows));
    }
    if shape.strip {
        list.push((Band::Strip, 1));
    }
    list.push((Band::Main, 0));
    if shape.key_bar {
        list.push((Band::KeyBar, 1));
    }
    if shape.footer {
        list.push((Band::Footer, 1));
    }
    list
}

/// Lays the bands out in a grid `grid_rows` high. The main area gets every
/// row the fixed chrome leaves; on a grid too short for the chrome it gets
/// none and later bands run past the bottom, so callers check the size first.
pub fn chrome_rows(shape: &ChromeShape, grid_rows: u16) -> ChromeRows {
    let list = bands(shape);
    let fixed: u16 = list.iter().map(|(_, rows)| *rows).sum();
    let rule_count = u16::try_from(list.len() - 1).unwrap_or(0);
    let main_rows = grid_rows.saturating_sub(fixed + rule_count + 2);
    let mut out = ChromeRows {
        banner: None,
        strip: None,
        main: RowSpan { y: 0, rows: 0 },
        key_bar: None,
        footer: None,
        rules: Vec::new(),
        bottom_border: 0,
    };
    let mut y = 1u16;
    for (index, (band, fixed_rows)) in list.iter().enumerate() {
        let rows = if *band == Band::Main { main_rows } else { *fixed_rows };
        match band {
            Band::Banner => out.banner = Some(RowSpan { y, rows }),
            Band::Strip => out.strip = Some(y),
            Band::Main => out.main = RowSpan { y, rows },
            Band::KeyBar => out.key_bar = Some(y),
            Band::Footer => out.footer = Some(y),
        }
        y = y.saturating_add(rows);
        if index + 1 < list.len() {
            out.rules.push(y);
            y = y.saturating_add(1);
        }
    }
    out.bottom_border = y;
    out
}

#[cfg(test)]
include!("chrome_rows.tests.rs");
