//! HTML bodies as plain text, with the links collected as numbered
//! references at the end.

use mail_parser::decoders::html::html_to_text as basic_html_to_text;

use super::limits::MAX_HTML_DEPTH;

/// Where paragraphs are wrapped. The reader wraps again to its own width, so
/// this is a ceiling that keeps tables and rules from growing without limit.
const WRAP: usize = 100;

/// Tags that have no end tag, or whose end tag may be left out, so they do
/// not count as one more level of nesting.
const NO_NESTING: [&str; 24] = [
    "area", "base", "br", "col", "colgroup", "dd", "dt", "embed", "hr", "img", "input", "li", "link", "meta",
    "option", "p", "param", "source", "td", "th", "tr", "track", "wbr", "tbody",
];

/// The text of an HTML document. If the converter cannot read it, or the tags
/// are nested so deep that it should not be asked to, a simpler tag stripper
/// does the job.
pub fn html_to_text(html: &str) -> String {
    if too_nested(html) {
        return basic_html_to_text(html);
    }
    html2text::config::plain_no_decorate()
        .link_footnotes(true)
        .string_from_read(html.as_bytes(), WRAP)
        .unwrap_or_else(|_| basic_html_to_text(html))
}

/// Whether the tags nest deeper than the converter is trusted with. A rough
/// count in one pass over the text: each start tag is a level down and each
/// end tag a level up, except for the tags that need no end tag.
pub fn too_nested(html: &str) -> bool {
    let bytes = html.as_bytes();
    let (mut depth, mut at) = (0usize, 0usize);
    while let Some(found) = bytes[at..].iter().position(|&b| b == b'<') {
        at += found + 1;
        let closing = bytes.get(at) == Some(&b'/');
        let start = at + usize::from(closing);
        let name_len = bytes[start.min(bytes.len())..].iter().take_while(|b| b.is_ascii_alphanumeric()).count();
        if name_len == 0 {
            continue;
        }
        let name = String::from_utf8_lossy(&bytes[start..start + name_len]).to_ascii_lowercase();
        at = start + name_len;
        if NO_NESTING.contains(&name.as_str()) {
            continue;
        }
        if closing {
            depth = depth.saturating_sub(1);
        } else {
            depth += 1;
            if depth > MAX_HTML_DEPTH {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
include!("html_to_text.tests.rs");
