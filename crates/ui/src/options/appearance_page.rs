//! The Appearance page: theme, icons, text size and line spacing, then the
//! layout settings (`layout_page.rs`).

use super::appearance_values::{icon_name, spacing_text};
use super::control::Control;
use super::layout_page;
use super::rows::{Item, Row, Tone};
use super::view::OptionsView;

pub fn rows(view: &OptionsView<'_>) -> Vec<Row> {
    let appearance = &view.config.appearance;
    let font = &view.config.font;
    let mut rows = vec![
        Row::field("Theme", Item::Choice { control: Control::Theme, text: appearance.theme.clone() }),
        Row::field("Icons", Item::Choice { control: Control::Icons, text: icon_name(appearance.icons).to_string() }),
        window_only("Text size", Control::TextSize, font.size.to_string()),
        window_only("Line spacing", Control::LineSpacing, spacing_text(font.line_spacing)),
    ];
    rows.extend(layout_page::rows(view));
    rows
}

/// A stepper whose value only the window shows: a terminal keeps its own font
/// and rows.
fn window_only(label: &str, control: Control, text: String) -> Row {
    Row {
        label: Some(label.to_string()),
        items: vec![Item::Stepper { control, text }, Item::text("window only", Tone::Dim)],
    }
}

#[cfg(test)]
include!("appearance_page.tests.rs");
