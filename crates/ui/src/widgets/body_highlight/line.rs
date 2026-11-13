//! The rules applied to one line of spans or one plain-text span.

use model::{BodyRole, StyledLine, StyledSpan};

use super::bullets::find_bullet;
use super::found::Found;
use super::links::{find_links, find_paren_paths};
use super::names::{find_app_name, find_signature};
use super::rules::HighlightRules;
use super::tokens::{find_search_tokens, find_versions};

/// The sub-spans of one plain-text span. `at_line_start` says that nothing
/// but blanks comes before the span on its line, which the bullet and
/// signature rules need. Rules run in a fixed order and an earlier match
/// keeps its text. Text no rule matches comes back as one `Text` span.
pub fn highlight_text(
    text: &str,
    at_line_start: bool,
    rules: &HighlightRules,
    app_name: &str,
) -> Vec<StyledSpan> {
    let mut found = Found::new();
    if at_line_start && rules.bullets {
        find_bullet(text, rules.lead, &mut found);
    }
    if at_line_start && rules.signature {
        find_signature(text, &mut found);
    }
    if rules.links {
        find_links(text, &mut found);
    }
    if rules.paths {
        find_paren_paths(text, &mut found);
    }
    if rules.name {
        find_app_name(text, app_name, &mut found);
    }
    if rules.tokens {
        find_search_tokens(text, &mut found);
    }
    if rules.versions {
        find_versions(text, &mut found);
    }
    found.into_spans(text)
}

/// True when the first thing on the line, past blanks, is a plain `>`.
fn is_quote(line: &StyledLine) -> bool {
    line.spans
        .iter()
        .find(|span| !span.text.trim().is_empty())
        .is_some_and(|span| span.role == BodyRole::Text && span.text.trim_start().starts_with('>'))
}

/// A line with its plain spans highlighted. Spans that already carry a role
/// are left alone. A quoted line turns all its plain spans into `Quote`.
pub fn highlight_line(line: &StyledLine, rules: &HighlightRules, app_name: &str) -> Vec<StyledSpan> {
    let quoted = rules.quote && is_quote(line);
    let mut out = Vec::with_capacity(line.spans.len());
    let mut at_start = true;
    for span in &line.spans {
        let plain = span.role == BodyRole::Text;
        if plain && quoted {
            out.push(StyledSpan {
                text: span.text.clone(),
                role: BodyRole::Quote,
            });
        } else if plain {
            out.extend(highlight_text(&span.text, at_start, rules, app_name));
        } else {
            out.push(span.clone());
        }
        at_start &= span.text.trim().is_empty();
    }
    out
}

#[cfg(test)]
include!("line.tests.rs");
