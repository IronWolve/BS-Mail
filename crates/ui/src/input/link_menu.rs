//! The menu keys on the link the keys are on. `.`, the Menu key and Shift+F10
//! open the menu of the selected item; while the keys' link is in play (the
//! reader has the focus and `l` or `L` has chosen a link) the item is that link,
//! and the menu is the one a right click on the link opens, with the same
//! entries (Open and Copy link) for the same address. It opens just under the
//! link so the link stays in view, and closes as any menu does: the link stays
//! chosen. With no link chosen the menu keys do what they did.

use super::menu_tables::MenuKind;
use super::menus::MenuTarget;
use crate::app_view::AppView;
use crate::layout::Layout;

impl AppView {
    /// Opens the menu of the link the keys are on, when it is in play. Returns
    /// whether a menu opened.
    pub(crate) fn open_link_menu(&mut self, layout: &Layout) -> bool {
        let Some(reader) = layout.reader.as_ref().filter(|_| self.link_mode()) else {
            return false;
        };
        let Some((index, address)) = self.reading.link_cursor.as_ref().map(|cursor| (cursor.index, cursor.address.clone())) else {
            return false;
        };
        // The row just under the link's last row in view; the menu is pushed up
        // to fit when there is no room below.
        let (first, last) = self.rows_in_view(layout);
        let bottom = self.reader_links(layout).iter().find(|link| link.index == index).map_or(first, |link| link.rows.1);
        let line = u16::try_from(bottom.clamp(first, last) - first).unwrap_or(0);
        let anchor = (reader.body.x.saturating_add(2), reader.body.y.saturating_add(line).saturating_add(1));
        self.open_menu(MenuKind::Link, MenuTarget::Link(address), anchor);
        true
    }
}

#[cfg(test)]
include!("link_menu.tests.rs");
