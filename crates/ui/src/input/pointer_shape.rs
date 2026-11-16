//! The pointer to show at a cell: the hand over everything a click acts on,
//! the text pointer over a text field, the resize pointers over the bars, and
//! the arrow everywhere else. What is open over the mail decides what is
//! asked about: a menu, a question, the move picker, the search box or the
//! composer each answer for themselves.

use surface::{PointerShape, Rect};

use super::drag::divider_at;
use super::hit::{Hit, account_at};
use crate::app_view::AppView;
use crate::composer::mouse::pointer_shape as composer_shape;
use crate::ask::Ask;
use crate::delete_prompt::draw::{button_at as prompt_button_at, confirm_label, prompt_rects, prompt_rects_for};
use crate::layout::Layout;

fn hand_if(over: bool) -> PointerShape {
    if over { PointerShape::Hand } else { PointerShape::Arrow }
}

impl AppView {
    /// The pointer to show at column `col`, row `row` of a `grid` grid: the
    /// resize pointer on a bar, and the one for the bar being dragged for as
    /// long as the drag lasts, even when the pointer has run ahead of it.
    pub fn pointer_shape(&self, col: u16, row: u16, grid: (u16, u16)) -> PointerShape {
        if self.setup.is_some() {
            return PointerShape::Arrow;
        }
        if self.options.is_some() {
            return self.options_pointer_shape(col, row, grid);
        }
        if let Some(form) = &self.composer {
            return composer_shape(form, self.layout(grid.0, grid.1).main, (col, row));
        }
        if self.menu.is_some() {
            return hand_if(self.menu_entry_under(col, row, grid));
        }
        if let Some(prompt) = &self.delete_prompt {
            let rects = prompt_rects(Rect::new(0, 0, grid.0, grid.1), confirm_label(prompt.for_good));
            return hand_if(rects.and_then(|rects| prompt_button_at(&rects, col, row)).is_some());
        }
        if self.picker.is_some() {
            return hand_if(self.picker_row_under(col, row, grid));
        }
        if let Some(ask) = &self.ask {
            return match ask {
                Ask::Name(_) => self.name_pointer_shape(col, row, grid),
                Ask::Tag(_) => hand_if(self.tag_row_under(col, row, grid)),
                Ask::Path(prompt) => prompt.pointer_shape(Rect::new(0, 0, grid.0, grid.1), (col, row)),
                Ask::Confirm(prompt) => {
                    let rects = prompt_rects_for(Rect::new(0, 0, grid.0, grid.1), prompt.yes, prompt.no);
                    hand_if(rects.and_then(|rects| prompt_button_at(&rects, col, row)).is_some())
                }
            };
        }
        if self.search_bar.is_some() {
            return self.search_pointer_shape(col, row, grid);
        }
        if self.overlays.is_open() {
            return PointerShape::Arrow;
        }
        if let Some(drag) = self.drag {
            return drag.shape();
        }
        if self.dragging_rows() {
            return hand_if(self.drop_target_at((col, row), grid).is_some());
        }
        let layout = self.layout(grid.0, grid.1);
        if let Some(drag) = divider_at(&layout, col, row) {
            return drag.shape();
        }
        hand_if(self.clickable_at(&layout, col, row))
    }

    /// Whether a click at the cell does something.
    fn clickable_at(&self, layout: &Layout, col: u16, row: u16) -> bool {
        if self.scrollbars(layout).iter().any(|bar| bar.holds(col, row)) {
            return true;
        }
        match self.hit(layout, col, row) {
            Hit::Message(_) | Hit::Folder(_) | Hit::KeyBar(_) => true,
            Hit::Account(_) => self.account_row_switches(layout, col, row),
            Hit::Reader => {
                self.link_under(layout, col, row).is_some() || self.over_find_word(layout, col, row) || self.attachment_at(layout, col, row).is_some()
            }
            Hit::Nothing => self.footer_span_at(layout, col, row).is_some(),
            Hit::Sidebar | Hit::ListChrome => false,
        }
    }

    /// Whether a click here would switch to another account: on a row of the
    /// sidebar's accounts that is not the one showing.
    fn account_row_switches(&self, layout: &Layout, col: u16, row: u16) -> bool {
        let Some(index) = account_at(layout, self.mail.accounts.len(), col, row) else {
            return false;
        };
        self.mail.accounts.get(index).is_some_and(|account| self.selection.account.as_ref() != Some(&account.id))
    }
}

#[cfg(test)]
include!("pointer_shape.tests.rs");
