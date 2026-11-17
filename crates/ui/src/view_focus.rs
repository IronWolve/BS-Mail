//! The cue for which pane has the keys, drawn over the main screen: the
//! pane's title in the accent color and underlined, and the selection bar of
//! each pane that does not have the keys drawn dimmer. The widgets know
//! nothing of focus: the title cue is a pass over the cells they wrote
//! (`widgets/focus_mark.rs`), and the bar cue is the role each is told to use
//! (`widgets/focus_bar.rs`).

use surface::{ColorDepth, Frame};

use crate::app_view::AppView;
use crate::layout::{Layout, SectionKind};
use crate::theme::roles;
use crate::view_state::Focus;
use crate::widgets::{DrawContext, Heading, bar_role, list_heading_extent, mark_focused, reader_heading_extent};

/// Which pane the screen shows as having the keys, and what the terminal can
/// show of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FocusLook {
    /// The pane that has the keys and is on screen.
    pub pane: Focus,
    /// The terminal shows no color at all, only attributes.
    pub colourless: bool,
}

impl FocusLook {
    /// The role the selection bar of a pane is drawn in: `own` while the
    /// pane has the keys, `idle` while another has them.
    pub(crate) fn bar(&self, ctx: &DrawContext<'_>, focused: bool, own: &'static str, idle: &'static str) -> &'static str {
        bar_role(ctx, focused, self.colourless, own, idle)
    }
}

impl AppView {
    /// The pane the screen shows as having the keys. It is the focused one;
    /// when that pane is not on screen (the sidebar switched off), the nearest
    /// pane that is, so the screen never shows no pane as the one with the keys.
    pub(crate) fn focus_look(&self, layout: &Layout) -> FocusLook {
        let on_screen = |pane: Focus| match pane {
            Focus::Sidebar => layout.sidebar.is_some(),
            Focus::List => layout.list.is_some(),
            Focus::Reader => layout.reader.is_some(),
        };
        let pane = [self.focus, Focus::List, Focus::Reader, Focus::Sidebar].into_iter().find(|pane| on_screen(*pane)).unwrap_or(self.focus);
        FocusLook { pane, colourless: self.capabilities().color_depth == ColorDepth::None }
    }

    /// The title of the pane that has the keys: underlined, and in the accent
    /// color.
    pub(crate) fn mark_focus(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout, look: FocusLook) {
        let heading = match look.pane {
            Focus::Sidebar => {
                let kind = if self.account_cursor_row().is_some() { SectionKind::Accounts } else { SectionKind::Folders };
                let title = layout.sidebar.as_ref().and_then(|sidebar| sidebar.section(kind)).map(|section| section.title);
                title.map(|row| Heading { row, text: row, role: roles::SECTION_TITLE })
            }
            Focus::List => layout.list.as_ref().map(|list| {
                let text = list_heading_extent(list.title, self.list_summary().as_ref(), self.order_note().as_deref());
                Heading { row: list.title, text, role: roles::LIST_TITLE }
            }),
            Focus::Reader => layout.reader.as_ref().map(|reader| {
                let data = self.shown_page().and_then(|page| self.reader_title_data(page));
                Heading { row: reader.title, text: reader_heading_extent(ctx, reader.title, data.as_ref()), role: roles::READER_TITLE }
            }),
        };
        if let Some(heading) = heading {
            mark_focused(frame, ctx, &heading, look.colourless);
        }
    }
}

#[cfg(test)]
include!("view_focus.tests.rs");
