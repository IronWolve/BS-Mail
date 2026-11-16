//! What a press on an Options control does: a request for the program, a
//! change of the view's own settings, or a change of what the screen shows.

use model::AccountId;

use crate::app_view::AppView;
use crate::options::appearance_values::{
    cycle_icons, cycle_theme, spacing_differs, step_font_size, step_line_spacing,
};
use crate::options::{Control, OptionsAction, OptionsState, Part};

const NOT_READY: &str = "That account is not ready yet; try again in a moment.";

impl AppView {
    /// The id of the account the highlight is on.
    fn highlighted_id(&self, state: &OptionsState) -> Option<String> {
        self.options_view().highlighted(state.cursor).map(|account| account.id.clone())
    }

    pub(crate) fn options_press(&mut self, state: &mut OptionsState, control: Control, part: Part) {
        let forward = matches!(part, Part::Next | Part::Plus);
        match control {
            Control::AddAccount => self.queue_options_action(OptionsAction::AddAccount),
            Control::EditAccount => {
                if let Some(id) = self.highlighted_id(state) {
                    self.queue_options_action(OptionsAction::EditAccount(id));
                }
            }
            Control::RemoveAccount => {
                if self.highlighted_id(state).is_some() {
                    state.begin_confirm();
                }
            }
            Control::ConfirmRemove => {
                if let Some(id) = self.highlighted_id(state) {
                    self.queue_options_action(OptionsAction::RemoveAccount(id));
                }
                state.cancel_confirm();
            }
            Control::KeepAccount => state.cancel_confirm(),
            Control::SwitchAccount => self.switch_to_highlighted(state),
            Control::Theme => self.press_theme(forward),
            Control::Icons => self.press_icons(forward),
            Control::TextSize => self.press_text_size(forward),
            Control::LineSpacing => self.press_line_spacing(forward),
            Control::ReaderPosition | Control::Density | Control::Threaded | Control::Column(_) | Control::Panel(_) => {
                self.press_layout(control, forward);
            }
            Control::Reload => self.queue_options_action(OptionsAction::ReloadSettings),
            Control::AccountList => {}
        }
    }

    /// Switches the mail to the highlighted account, as a click in the
    /// sidebar does.
    fn switch_to_highlighted(&mut self, state: &OptionsState) {
        let Some(id) = self.highlighted_id(state) else {
            return;
        };
        let id = AccountId(id);
        if self.mail.account(&id).is_none() {
            self.options_problem(NOT_READY);
            return;
        }
        self.switch_account(id);
    }

    /// The theme itself is the program's to load: ask, and the program calls
    /// `set_theme` with the result.
    fn press_theme(&mut self, forward: bool) {
        let names = &self.options_host.theme_names;
        if let Some(next) = cycle_theme(names, &self.config.appearance.theme, forward) {
            self.queue_options_action(OptionsAction::SetTheme(next));
        }
    }

    fn press_icons(&mut self, forward: bool) {
        let current = self.config.appearance.icons;
        let next = cycle_icons(current, forward);
        if next != current {
            self.set_icon_mode(next);
            self.queue_options_action(OptionsAction::SetIcons(next));
        }
    }

    fn press_text_size(&mut self, plus: bool) {
        let current = self.config.font.size;
        let next = step_font_size(current, plus);
        if next != current {
            self.config.font.size = next;
            self.queue_options_action(OptionsAction::SetFontSize(next));
        }
    }

    fn press_line_spacing(&mut self, plus: bool) {
        let current = self.config.font.line_spacing;
        let next = step_line_spacing(current, plus);
        if spacing_differs(next, current) {
            self.config.font.line_spacing = next;
            self.queue_options_action(OptionsAction::SetLineSpacing(next));
        }
    }
}
