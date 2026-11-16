//! The layout part of the Appearance page: where the reader sits, how the
//! sidebar is spaced, which columns and which panels show. Every one of them
//! is drawn from the settings in force and applies at once, which each row
//! says.

use super::control::Control;
use super::layout_choices::{Column, Panel, density_name, reader_name};
use super::rows::{Item, Row, Tone};
use super::view::OptionsView;

/// What a row says about when its setting applies.
const NOW: &str = "now";

pub fn rows(view: &OptionsView<'_>) -> Vec<Row> {
    let config = view.config;
    let mut rows = vec![
        Row::blank(),
        Row::text("Layout", Tone::Dim),
        applies_now("Reader", Control::ReaderPosition, reader_name(config.reader.position)),
        applies_now("Folder spacing", Control::Density, density_name(config.appearance.density)),
        applies_now("Threads", Control::Threaded, if config.list.threaded { "on" } else { "off" }),
        Row::blank(),
        Row::text("Columns of the list (the subject always shows)", Tone::Dim),
    ];
    rows.extend(Column::ALL.iter().map(|column| applies_now(column.label(), Control::Column(*column), shown_word(column.shown(config)))));
    rows.push(Row::blank());
    rows.push(Row::text("Panels", Tone::Dim));
    rows.extend(Panel::ALL.iter().map(|panel| applies_now(panel.label(), Control::Panel(*panel), shown_word(panel.shown(config)))));
    rows
}

fn shown_word(shown: bool) -> &'static str {
    if shown { "shown" } else { "hidden" }
}

fn applies_now(label: &str, control: Control, text: &str) -> Row {
    Row {
        label: Some(label.to_string()),
        items: vec![Item::Choice { control, text: text.to_string() }, Item::text(NOW, Tone::Dim)],
    }
}

#[cfg(test)]
include!("layout_page.tests.rs");
