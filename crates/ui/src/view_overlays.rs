//! The boxes drawn over the mail, from the bottom up: the composer, the move
//! picker, the question before a delete, the folder and tag boxes, the search
//! box, the context menu and the help. Moved out of `AppView::render` so that
//! file stays small.

use surface::{Capabilities, Frame, Rect};

use crate::app_view::AppView;
use crate::ask::Ask;
use crate::ask::confirm_draw::draw_confirm;
use crate::ask::name_draw::draw_name_prompt;
use crate::ask::tags_draw::draw_tag_picker;
use crate::composer::draw::draw_composer;
use crate::delete_prompt::draw::draw_delete_prompt;
use crate::layout::Layout;
use crate::move_picker::draw::draw_picker;
use crate::path_entry::draw_path_prompt;
use crate::search::draw::draw_search_bar;
use crate::view_state::OverlayKind;
use crate::widgets::{DrawContext, draw_help, draw_menu};

impl AppView {
    /// Draws whatever is open over the mail, in the order it stacks.
    pub(crate) fn draw_modals(&self, frame: &mut Frame, ctx: &DrawContext<'_>, layout: &Layout, capabilities: &Capabilities) {
        let area: Rect = frame.area();
        if let Some(form) = &self.composer {
            draw_composer(frame, ctx, layout.main, form);
        }
        if let Some(picker) = &self.picker {
            draw_picker(frame, ctx, area, picker, &self.mail.folders);
        }
        if let Some(prompt) = &self.delete_prompt {
            draw_delete_prompt(frame, ctx, area, prompt);
        }
        match &self.ask {
            Some(Ask::Name(prompt)) => draw_name_prompt(frame, ctx, area, prompt),
            Some(Ask::Confirm(prompt)) => draw_confirm(frame, ctx, area, prompt),
            Some(Ask::Tag(picker)) => draw_tag_picker(frame, ctx, area, picker),
            Some(Ask::Path(prompt)) => draw_path_prompt(frame, ctx, area, prompt),
            None => {}
        }
        if let Some(bar) = &self.search_bar {
            draw_search_bar(frame, ctx, area, bar);
        }
        if let Some(open) = &self.menu {
            draw_menu(frame, ctx, area, &open.menu);
        }
        if self.overlays.open == Some(OverlayKind::Help) {
            draw_help(frame, ctx, area, capabilities, self.overlays.help_scroll);
        }
    }
}
