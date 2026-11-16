//! The keys that edit a one-line box (the search box, the find box): the
//! caret moves by letter, word and line, and the text loses a letter or a
//! word. What Enter and Escape mean is for each box to say.

use surface::{Modifiers, NamedKey};

use crate::composer::EditorBuffer;

/// Does what `named` does to the text of `bar`; false for a key that is not an editing key.
pub(crate) fn edit_line(bar: &mut EditorBuffer, named: NamedKey, held: Modifiers) -> bool {
    match named {
        NamedKey::Left if held.ctrl => bar.move_word_left(held.shift),
        NamedKey::Right if held.ctrl => bar.move_word_right(held.shift),
        NamedKey::Left => bar.move_left(held.shift),
        NamedKey::Right => bar.move_right(held.shift),
        NamedKey::Home => bar.move_line_start(held.shift),
        NamedKey::End => bar.move_line_end(held.shift),
        NamedKey::Backspace if held.ctrl => {
            bar.delete_word_back();
        }
        NamedKey::Backspace => {
            bar.backspace();
        }
        NamedKey::Delete => {
            bar.delete_forward();
        }
        _ => return false,
    }
    true
}
