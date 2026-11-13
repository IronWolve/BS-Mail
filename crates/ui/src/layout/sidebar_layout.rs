//! The sidebar's sections stacked top to bottom: how many rows each gets,
//! where the dividers fall, and whether folders are spaced roomy or compact.

use model::config::Density;
use surface::Rect;

use super::clamp::{SECTION_MIN_ROWS, section_rows};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Folders,
    Accounts,
    Status,
}

impl SectionKind {
    /// The section named in `[sidebar] sections`; `None` for an unknown name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "folders" => Some(Self::Folders),
            "accounts" => Some(Self::Accounts),
            "status" => Some(Self::Status),
            _ => None,
        }
    }

    /// The configured sections in order, unknown names and repeats dropped.
    pub fn list_from_names(names: &[String]) -> Vec<Self> {
        let mut kinds: Vec<Self> = Vec::new();
        for kind in names.iter().filter_map(|name| Self::from_name(name)) {
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        kinds
    }
}

/// Spacing between top-level folders once density is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderSpacing {
    /// A blank row between top-level entries.
    Roomy,
    Compact,
}

/// How tall each section's content is; the caller counts from its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SidebarContent {
    pub folder_rows_roomy: u16,
    pub folder_rows_compact: u16,
    pub account_rows: u16,
    pub status_lines: u16,
}

/// What the sidebar layout reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarInput {
    pub sections: Vec<SectionKind>,
    pub density: Density,
    pub content: SidebarContent,
    /// Preferred rows for Accounts and Status; 0 fits their content.
    pub accounts_rows: u16,
    pub status_rows: u16,
}

/// One section: its title row, the rule under it and the body below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarSection {
    pub kind: SectionKind,
    pub rect: Rect,
    pub title: Rect,
    pub rule: Option<Rect>,
    pub body: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarLayout {
    /// The whole sidebar between its border and the divider.
    pub rect: Rect,
    pub sections: Vec<SidebarSection>,
    /// One divider row between each pair of sections.
    pub dividers: Vec<Rect>,
    pub spacing: FolderSpacing,
}

impl SidebarLayout {
    pub fn section(&self, kind: SectionKind) -> Option<&SidebarSection> {
        self.sections.iter().find(|section| section.kind == kind)
    }
}

fn desired(kind: SectionKind, spacing: FolderSpacing, input: &SidebarInput) -> u16 {
    let content = &input.content;
    match kind {
        SectionKind::Folders => {
            2 + match spacing {
                FolderSpacing::Roomy => content.folder_rows_roomy,
                FolderSpacing::Compact => content.folder_rows_compact,
            }
        }
        SectionKind::Accounts => section_rows(input.accounts_rows, 2 + content.account_rows),
        SectionKind::Status => section_rows(input.status_rows, 2 + content.status_lines),
    }
}

fn dividers_for(count: usize) -> u16 {
    u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX)
}

/// Roomy when every section fits at its content height with blank rows
/// between folders (and `auto` asks for that); otherwise compact.
fn pick_spacing(input: &SidebarInput, kinds: &[SectionKind], available: u16) -> FolderSpacing {
    let roomy = match input.density {
        Density::Roomy => true,
        Density::Compact => false,
        Density::Auto => {
            let total: u32 = kinds
                .iter()
                .map(|kind| u32::from(desired(*kind, FolderSpacing::Roomy, input)))
                .sum::<u32>()
                + u32::from(dividers_for(kinds.len()));
            total <= u32::from(available)
        }
    };
    if roomy {
        FolderSpacing::Roomy
    } else {
        FolderSpacing::Compact
    }
}

/// The section that takes the rows the others leave: Folders, else the last.
fn flex_kind(kinds: &[SectionKind]) -> Option<SectionKind> {
    if kinds.contains(&SectionKind::Folders) {
        Some(SectionKind::Folders)
    } else {
        kinds.last().copied()
    }
}

/// Drops the last non-flex section until every remaining one can have its
/// minimum rows.
fn keep_what_fits(kinds: &mut Vec<SectionKind>, available: u16) {
    while kinds.len() > 1 {
        let minimum = u32::from(SECTION_MIN_ROWS) * kinds.len() as u32
            + u32::from(dividers_for(kinds.len()));
        if minimum <= u32::from(available) {
            return;
        }
        let flex = flex_kind(kinds);
        match kinds.iter().rposition(|kind| Some(*kind) != flex) {
            Some(index) => {
                kinds.remove(index);
            }
            None => return,
        }
    }
}

/// Row counts per section: content height for the fixed ones (shrunk from
/// the bottom up toward the minimum when short of rows), the rest for flex.
fn heights(
    kinds: &[SectionKind],
    spacing: FolderSpacing,
    input: &SidebarInput,
    available: u16,
) -> Vec<u16> {
    let flex = flex_kind(kinds);
    let mut rows: Vec<u16> = kinds
        .iter()
        .map(|kind| {
            if Some(*kind) == flex {
                SECTION_MIN_ROWS
            } else {
                desired(*kind, spacing, input)
            }
        })
        .collect();
    let space = available.saturating_sub(dividers_for(kinds.len()));
    let mut excess = rows.iter().map(|r| u32::from(*r)).sum::<u32>() as i64 - i64::from(space);
    for (index, kind) in kinds.iter().enumerate().rev() {
        let room = i64::from(rows[index].saturating_sub(SECTION_MIN_ROWS));
        if Some(*kind) != flex && excess > 0 && room > 0 {
            let cut = room.min(excess);
            rows[index] -= cut as u16;
            excess -= cut;
        }
    }
    let others: u16 = kinds
        .iter()
        .zip(&rows)
        .filter(|(kind, _)| Some(**kind) != flex)
        .map(|(_, r)| *r)
        .sum();
    if let Some(at) = kinds.iter().position(|kind| Some(*kind) == flex) {
        rows[at] = space.saturating_sub(others);
    }
    rows
}

fn section_at(kind: SectionKind, x: u16, y: u16, width: u16, rows: u16) -> SidebarSection {
    let row = |dy: u16, count: u16| Rect::new(x, y.saturating_add(dy), width, count);
    SidebarSection {
        kind,
        rect: Rect::new(x, y, width, rows),
        title: row(0, rows.min(1)),
        rule: (rows >= 2).then(|| row(1, 1)),
        body: row(2, rows.saturating_sub(2)),
    }
}

/// Lays the sections out inside `area`, the sidebar between its border and
/// the divider. Sections use the text columns: one cell in on each side.
pub fn layout_sidebar(area: Rect, input: &SidebarInput) -> SidebarLayout {
    let mut kinds = input.sections.clone();
    keep_what_fits(&mut kinds, area.height);
    let spacing = pick_spacing(input, &kinds, area.height);
    let rows = heights(&kinds, spacing, input, area.height);
    let x = area.x.saturating_add(1);
    let width = area.width.saturating_sub(2);
    let mut y = area.y;
    let mut sections = Vec::with_capacity(kinds.len());
    let mut dividers = Vec::new();
    for (index, (kind, count)) in kinds.iter().zip(&rows).enumerate() {
        sections.push(section_at(*kind, x, y, width, *count));
        y = y.saturating_add(*count);
        if index + 1 < kinds.len() {
            dividers.push(Rect::new(x, y, width, 1));
            y = y.saturating_add(1);
        }
    }
    SidebarLayout {
        rect: area,
        sections,
        dividers,
        spacing,
    }
}

#[cfg(test)]
include!("sidebar_layout.tests.rs");
