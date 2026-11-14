//! The Appearance page: theme, icons, text size and line spacing.

use super::appearance_values::{icon_name, spacing_text};
use super::control::Control;
use super::rows::{Item, Row, Tone};
use super::view::OptionsView;

pub fn rows(view: &OptionsView<'_>) -> Vec<Row> {
    let appearance = &view.config.appearance;
    let font = &view.config.font;
    vec![
        Row::field("Theme", Item::Choice { control: Control::Theme, text: appearance.theme.clone() }),
        Row::field("Icons", Item::Choice { control: Control::Icons, text: icon_name(appearance.icons).to_string() }),
        Row {
            label: Some("Text size".to_string()),
            items: vec![
                Item::Stepper { control: Control::TextSize, text: font.size.to_string() },
                Item::text("window only", Tone::Dim),
            ],
        },
        Row::field("Line spacing", Item::Stepper { control: Control::LineSpacing, text: spacing_text(font.line_spacing) }),
    ]
}

#[cfg(test)]
include!("appearance_page.tests.rs");
