//! The styles one row's cells use. On the selected row the bar's background
//! sits under every cell; plain text takes the bar's own style, and text
//! that carries meaning (the unread dot, the star, a subject prefix) keeps
//! its foreground and gains the bar's weight.

use surface::Style;

use crate::theme::roles;
use crate::widgets::DrawContext;

#[derive(Clone, Copy)]
pub struct RowStyle<'a> {
    ctx: &'a DrawContext<'a>,
    selected: bool,
}

impl<'a> RowStyle<'a> {
    pub fn new(ctx: &'a DrawContext<'a>, selected: bool) -> Self {
        Self { ctx, selected }
    }

    /// Ordinary row text: the role's style, or the bar's style on the
    /// selected row.
    pub fn plain(&self, role: &str) -> Style {
        if self.selected {
            self.ctx.full(roles::LIST_SELECTED)
        } else {
            self.ctx.fg(role)
        }
    }

    /// Text with a meaning of its own: the role's foreground; on the selected
    /// row also the bar's background and weight.
    pub fn accent(&self, role: &str) -> Style {
        let style = self.ctx.fg(role);
        if !self.selected {
            return style;
        }
        let bar = self.ctx.full(roles::LIST_SELECTED);
        let mut on_bar = style.add_modifier(bar.add_modifier);
        on_bar.bg = bar.bg;
        on_bar
    }
}

#[cfg(test)]
include!("row_style.tests.rs");
