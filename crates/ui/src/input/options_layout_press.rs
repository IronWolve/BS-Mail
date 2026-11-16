//! What a press on a layout control of the Options page does: the view's own
//! settings change at once, so the screen shows the change when the page is
//! closed, and the program is asked to save it.

use crate::app_view::AppView;
use crate::options::layout_choices::{LayoutChange, cycle_density, cycle_reader, toggled_columns};
use crate::options::{Control, OptionsAction};

impl AppView {
    /// A press on the reader's place, the spacing, a column or a panel.
    /// `forward` is the direction of a choice that goes round; a column or a
    /// panel is shown or hidden by either.
    pub(crate) fn press_layout(&mut self, control: Control, forward: bool) {
        let change = match control {
            Control::ReaderPosition => {
                let next = cycle_reader(self.config.reader.position, forward);
                self.config.reader.position = next;
                LayoutChange::ReaderPosition(next)
            }
            Control::Density => {
                let next = cycle_density(self.config.appearance.density, forward);
                self.config.appearance.density = next;
                LayoutChange::Density(next)
            }
            Control::Column(column) => {
                let shown = !column.shown(&self.config);
                self.config.list.columns = toggled_columns(&self.config.list.columns, column, shown);
                LayoutChange::Columns(self.config.list.columns.clone())
            }
            Control::Panel(panel) => {
                let shown = !panel.shown(&self.config);
                panel.set(&mut self.config, shown);
                LayoutChange::Panel(panel, shown)
            }
            _ => return,
        };
        self.queue_options_action(OptionsAction::SetLayout(change));
    }
}

#[cfg(test)]
include!("options_layout_press.tests.rs");
