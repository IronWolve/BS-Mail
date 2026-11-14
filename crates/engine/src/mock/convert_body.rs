//! Turns the fixture's span lists into the model's styled body lines.

use std::collections::BTreeMap;

use model::{Body, BodyRole, StyledLine, StyledSpan};

use super::fixture_error::FieldProblem;
use super::fixture_schema::SpanRow;

/// The body of the open message. `field` is the path of the body in the file
/// (for error messages); a problem names the line and span at fault.
pub(crate) fn body_from_rows(rows: &[Vec<SpanRow>], field: &str) -> Result<Body, FieldProblem> {
    let mut lines = Vec::with_capacity(rows.len());
    for (line_index, row) in rows.iter().enumerate() {
        let mut spans = Vec::with_capacity(row.len());
        for (span_index, span) in row.iter().enumerate() {
            let at = format!("{field}[{line_index}][{span_index}]");
            spans.push(span_from_row(span, &at)?);
        }
        lines.push(StyledLine { spans });
    }
    Ok(Body {
        lines,
        attachments: Vec::new(),
        reply_to: Vec::new(),
    })
}

fn span_from_row(row: &SpanRow, at: &str) -> Result<StyledSpan, FieldProblem> {
    match row {
        SpanRow::Plain(text) => Ok(StyledSpan {
            text: text.clone(),
            role: BodyRole::Text,
        }),
        SpanRow::Styled(table) => styled_span(table, at),
    }
}

fn styled_span(table: &BTreeMap<String, String>, at: &str) -> Result<StyledSpan, FieldProblem> {
    let mut entries = table.iter();
    let (Some((role, text)), None) = (entries.next(), entries.next()) else {
        return Err(FieldProblem::new(
            at,
            "a styled span is a table with exactly one key, the role",
        ));
    };
    let role = role_from_name(role)
        .ok_or_else(|| FieldProblem::new(at, format!("unknown span role `{role}`")))?;
    Ok(StyledSpan {
        text: text.clone(),
        role,
    })
}

fn role_from_name(name: &str) -> Option<BodyRole> {
    Some(match name {
        "text" => BodyRole::Text,
        "quote" => BodyRole::Quote,
        "name" => BodyRole::Name,
        "bullet_done" => BodyRole::BulletDone,
        "bullet_open" => BodyRole::BulletOpen,
        "lead" => BodyRole::Lead,
        "lead_next" => BodyRole::LeadNext,
        "token" => BodyRole::Token,
        "path" => BodyRole::Path,
        "link" => BodyRole::Link,
        "signature" => BodyRole::Signature,
        _ => return None,
    })
}
