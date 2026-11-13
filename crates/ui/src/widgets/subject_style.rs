//! A subject as styled runs: a leading `[Tag]` colored by the tag table in
//! the `[list]` config, the warning mark and orange text for an alert tag,
//! and a `#123:` number right after the prefix.

use surface::Style;

use super::cell_text::Run;
use super::draw_context::DrawContext;
use crate::icons::IconId;
use crate::theme::roles;

/// Longest tag name, in characters, that still counts as a prefix.
const MAX_TAG_CHARS: usize = 32;

/// A leading `[Tag]`: the name inside, the bracketed text and what follows.
struct Prefix<'a> {
    name: &'a str,
    bracketed: &'a str,
    rest: &'a str,
}

fn split_prefix(subject: &str) -> Option<Prefix<'_>> {
    let inner = subject.strip_prefix('[')?;
    let close = inner.find(']')?;
    let name = &inner[..close];
    let valid = !name.trim().is_empty()
        && name.chars().count() <= MAX_TAG_CHARS
        && !name.contains('[');
    valid.then(|| Prefix {
        name,
        bracketed: &subject[..close + 2],
        rest: &subject[close + 2..],
    })
}

/// Splits ` #123: more` into the blanks before the number, the number with
/// its colon, and the rest. `None` when no number follows the prefix.
fn split_number(rest: &str) -> Option<(&str, &str, &str)> {
    let body = rest.trim_start_matches(' ');
    let blanks = &rest[..rest.len() - body.len()];
    let digits = body.strip_prefix('#')?;
    let count = digits.chars().take_while(char::is_ascii_digit).count();
    if count == 0 || !digits[count..].starts_with(':') {
        return None;
    }
    let end = 1 + count + 1;
    Some((blanks, &body[..end], &body[end..]))
}

/// The subject as runs. `base` styles ordinary text; `accent` gives the
/// style of an accent role (so the caller can put it on a selection bar).
/// With `prefix_colors` off, or when the tag table does not name the tag,
/// the whole subject is one run in `base`.
pub fn subject_runs(
    ctx: &DrawContext<'_>,
    subject: &str,
    base: Style,
    accent: &dyn Fn(&str) -> Style,
    prefix_colors: bool,
) -> Vec<Run> {
    let plain = || vec![Run::new(subject, base)];
    if !prefix_colors || ctx.config.list.subject_tags.is_empty() {
        return plain();
    }
    let Some(prefix) = split_prefix(subject) else {
        return plain();
    };
    // A tag the table does not name is left alone: the picture colors the
    // tags it knows and shows the rest as ordinary subject text.
    let Some(role) = ctx.config.list.subject_tags.get(prefix.name) else {
        return plain();
    };
    let mut runs = prefixed_runs(ctx, &prefix, role, base, accent);
    runs.retain(|run| !run.text.is_empty());
    runs
}

fn prefixed_runs(
    ctx: &DrawContext<'_>,
    prefix: &Prefix<'_>,
    role: &str,
    base: Style,
    accent: &dyn Fn(&str) -> Style,
) -> Vec<Run> {
    if role == roles::SUBJECT_TAG_ALERT {
        return vec![
            Run::new(ctx.icon_inline(IconId::Warning), accent(roles::SUBJECT_WARNING)),
            Run::new(" ", base),
            Run::new(prefix.bracketed, accent(role)),
            Run::new(prefix.rest, accent(roles::SUBJECT_ALERT)),
        ];
    }
    let mut runs = vec![Run::new(prefix.bracketed, accent(role))];
    match split_number(prefix.rest) {
        Some((blanks, number, rest)) => {
            runs.push(Run::new(blanks, base));
            runs.push(Run::new(number, accent(roles::SUBJECT_NUMBER)));
            runs.push(Run::new(rest, base));
        }
        None => runs.push(Run::new(prefix.rest, base)),
    }
    runs
}

#[cfg(test)]
include!("subject_style.tests.rs");
