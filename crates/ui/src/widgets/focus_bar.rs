//! The cue in a selection bar: the bar of a pane that does not have the keys
//! is drawn dimmer than the bar of the one that has them, so the selection
//! stays in view and the eye goes to the pane the keys act on.

use super::draw_context::DrawContext;

/// The role a pane's selection bar is drawn in: `own` when the pane has the
/// keys, `idle` (the dimmer look) when another has them.
///
/// The focused look is kept where the dim one would lose the bar: a theme
/// written before the dim role existed has no `idle`, and a terminal with no
/// color shows the bar only by its weight, which the dim look gives up.
pub fn bar_role(ctx: &DrawContext<'_>, focused: bool, colourless: bool, own: &'static str, idle: &'static str) -> &'static str {
    if focused || colourless || !ctx.theme.has_role(idle) { own } else { idle }
}

#[cfg(test)]
include!("focus_bar.tests.rs");
