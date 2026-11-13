//! HTML bodies as plain text, with the links collected as numbered
//! references at the end.

use mail_parser::decoders::html::html_to_text as basic_html_to_text;

/// Where paragraphs are wrapped. The reader wraps again to its own width, so
/// this is a ceiling that keeps tables and rules from growing without limit.
const WRAP: usize = 100;

/// The text of an HTML document. If the converter cannot read it, a simpler
/// tag stripper does the job.
pub fn html_to_text(html: &str) -> String {
    html2text::config::plain_no_decorate()
        .link_footnotes(true)
        .string_from_read(html.as_bytes(), WRAP)
        .unwrap_or_else(|_| basic_html_to_text(html))
}

#[cfg(test)]
include!("html_to_text.tests.rs");
