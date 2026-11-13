//! Widths and positions of the message list's columns. The flex column takes
//! what the fixed columns leave; when it would fall under its minimum, fixed
//! columns give up width in the order of plan 4.4.

use model::config::ListWidths;

/// Gap between two columns.
pub const GUTTER: u16 = 1;
/// The flex column never gets less than this while anything can still give.
pub const FLEX_MIN: u16 = 10;
/// Width of the Date column once it switches to its short format.
pub const DATE_SHORT_WIDTH: u16 = 11;
/// Width given to a fixed column that is not the flex column and has no
/// configured width (a Subject column that is not flex).
const UNSET_WIDTH: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnId {
    Num,
    Flag,
    Star,
    From,
    Subject,
    Date,
    Size,
}

impl ColumnId {
    /// The column named in `[list] columns`; `None` for an unknown name.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "num" => Self::Num,
            "flag" => Self::Flag,
            "star" => Self::Star,
            "from" => Self::From,
            "subject" => Self::Subject,
            "date" => Self::Date,
            "size" => Self::Size,
            _ => return None,
        })
    }

    /// Narrowest width the column may be dragged to: icon columns are 3,
    /// text columns at least 6 where the default allows it.
    pub fn min_width(self) -> u16 {
        match self {
            Self::Num | Self::Flag | Self::Star => 3,
            Self::From | Self::Date => 6,
            Self::Subject => FLEX_MIN,
            Self::Size => 4,
        }
    }

    fn fixed_width(self, widths: &ListWidths) -> u16 {
        let wanted = match self {
            Self::Num => widths.num,
            Self::Flag => widths.flag,
            Self::Star => widths.star,
            Self::From => widths.from,
            Self::Date => widths.date,
            Self::Size => widths.size,
            Self::Subject => UNSET_WIDTH,
        };
        match self {
            Self::Flag | Self::Star => 3,
            _ => wanted.max(self.min_width()),
        }
    }
}

/// One column's place: absolute column and width in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSlot {
    pub id: ColumnId,
    pub x: u16,
    pub width: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnLayout {
    pub slots: Vec<ColumnSlot>,
    /// The Date column switched to its short format.
    pub short_date: bool,
}

impl ColumnLayout {
    pub fn slot(&self, id: ColumnId) -> Option<&ColumnSlot> {
        self.slots.iter().find(|slot| slot.id == id)
    }
}

/// Which columns and in what order, and which one flexes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSpec {
    pub order: Vec<ColumnId>,
    pub flex: ColumnId,
}

impl ColumnSpec {
    /// From the `[list]` names; unknown names and repeats are ignored.
    pub fn from_names(order: &[String], flex: &str) -> Self {
        let mut ids: Vec<ColumnId> = Vec::new();
        for id in order.iter().filter_map(|name| ColumnId::from_name(name)) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        Self {
            order: ids,
            flex: ColumnId::from_name(flex).unwrap_or(ColumnId::Subject),
        }
    }
}

/// Cells between two columns that sit next to each other. The flag and star
/// columns form one group, like their shared header bar, so nothing comes
/// between them.
pub fn gutter_between(left: ColumnId, right: ColumnId) -> u16 {
    match (left, right) {
        (ColumnId::Flag, ColumnId::Star) | (ColumnId::Star, ColumnId::Flag) => 0,
        _ => GUTTER,
    }
}

/// Working set while columns give up width.
struct Cols {
    items: Vec<(ColumnId, u16)>,
    flex: ColumnId,
    short_date: bool,
}

impl Cols {
    fn flex_width(&self, width: u16) -> i32 {
        let gutters: i32 = self
            .items
            .windows(2)
            .map(|pair| i32::from(gutter_between(pair[0].0, pair[1].0)))
            .sum();
        let fixed: i32 = self
            .items
            .iter()
            .filter(|(id, _)| *id != self.flex)
            .map(|(_, w)| i32::from(*w))
            .sum();
        i32::from(width) - gutters - fixed
    }

    fn width_of(&mut self, id: ColumnId) -> Option<&mut u16> {
        self.items
            .iter_mut()
            .find(|(item, _)| *item == id)
            .map(|(_, w)| w)
    }

    fn shrink_from(&mut self, deficit: i32) {
        if let Some(w) = self.width_of(ColumnId::From) {
            let room = i32::from(w.saturating_sub(ColumnId::From.min_width()));
            *w -= deficit.clamp(0, room) as u16;
        }
    }

    fn hide(&mut self, id: ColumnId) {
        self.items.retain(|(item, _)| *item != id);
    }

    fn shorten_date(&mut self) {
        if let Some(w) = self.width_of(ColumnId::Date) {
            *w = (*w).min(DATE_SHORT_WIDTH);
            self.short_date = true;
        }
    }
}

/// Lays the columns out across `width` cells starting at column `x`.
pub fn layout_columns(spec: &ColumnSpec, widths: &ListWidths, x: u16, width: u16) -> ColumnLayout {
    let mut cols = Cols {
        items: spec
            .order
            .iter()
            .map(|id| (*id, id.fixed_width(widths)))
            .collect(),
        flex: spec.flex,
        short_date: false,
    };
    let need = i32::from(FLEX_MIN);
    let deficit = |cols: &Cols| need - cols.flex_width(width);
    if deficit(&cols) > 0 {
        let wanted = deficit(&cols);
        cols.shrink_from(wanted);
    }
    if deficit(&cols) > 0 {
        cols.hide(ColumnId::Num);
    }
    if deficit(&cols) > 0 {
        cols.hide(ColumnId::Size);
    }
    if deficit(&cols) > 0 {
        cols.shorten_date();
    }
    place(&cols, x, width)
}

fn place(cols: &Cols, x: u16, width: u16) -> ColumnLayout {
    let flex = cols.flex_width(width).max(0) as u16;
    let mut at = x;
    let mut slots = Vec::with_capacity(cols.items.len());
    for (index, (id, fixed)) in cols.items.iter().enumerate() {
        let slot_width = if *id == cols.flex { flex } else { *fixed };
        if slot_width > 0 {
            slots.push(ColumnSlot {
                id: *id,
                x: at,
                width: slot_width,
            });
            let gutter = cols
                .items
                .get(index + 1)
                .map_or(0, |(next, _)| gutter_between(*id, *next));
            at = at.saturating_add(slot_width).saturating_add(gutter);
        }
    }
    ColumnLayout {
        slots,
        short_date: cols.short_date,
    }
}

#[cfg(test)]
include!("columns.tests.rs");
