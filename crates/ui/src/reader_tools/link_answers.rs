//! The last answers to "which link is under the pointer", kept for the pointer's
//! moves. The pointer moves often and asking costs a hash of the whole body (to
//! be sure the wrapped lines are the body's), so an answer is kept with what it
//! was made from and given again while all of that is the same: the cell, the
//! scroll, the pane, the message and the identity of its lines. The last two are
//! kept, because one move is asked about at both ends (the cell the pointer left
//! and the one it came to) and by the pointer's shape before the screen sees the
//! move. A key, a click or an event from the backend forgets them, so a body that
//! changed under the same cell is never answered from the old one. Drawing never
//! uses them.

use std::cell::RefCell;

use model::MessageId;
use surface::Rect;

/// What an answer was made from: a change in any part makes it stale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Basis {
    /// The screen cell asked about.
    pub cell: (u16, u16),
    /// The first row of the body shown, and the rectangle it is drawn in.
    pub first: usize,
    pub body: Rect,
    /// The message shown, and where and how long its lines are.
    pub message: Option<MessageId>,
    pub lines: (usize, usize),
    /// How many lines come before the body (the attachments).
    pub lead: usize,
    /// The settings that wrap and highlight: the wrap column and the option
    /// that colours the body.
    pub settings: (u16, bool),
}

/// How many answers are kept.
const KEPT: usize = 2;

#[derive(Default)]
struct Kept {
    /// The newest first.
    recent: Vec<(Basis, Option<usize>)>,
    worked_out: usize,
}

/// The kept answers. They live in a cell because the pointer's questions are
/// asked of a view that is only borrowed.
#[derive(Default)]
pub struct LinkAnswers {
    kept: RefCell<Kept>,
}

impl LinkAnswers {
    /// The answer for `basis`: a kept one when it was made from the same, else
    /// what `work` finds, which is then kept in place of the oldest.
    pub(crate) fn answer(&self, basis: Basis, work: impl FnOnce() -> Option<usize>) -> Option<usize> {
        if let Some((_, answer)) = self.kept.borrow().recent.iter().find(|(known, _)| *known == basis) {
            return *answer;
        }
        let answer = work();
        let mut kept = self.kept.borrow_mut();
        kept.worked_out += 1;
        kept.recent.insert(0, (basis, answer));
        kept.recent.truncate(KEPT);
        answer
    }

    /// Lets go of the kept answers: something they may have come from changed.
    pub fn forget(&self) {
        self.kept.borrow_mut().recent.clear();
    }

    /// How many times an answer was worked out rather than given again.
    pub fn worked_out(&self) -> usize {
        self.kept.borrow().worked_out
    }
}

#[cfg(test)]
include!("link_answers.tests.rs");
