//! Recipient lists as people type them: `a@b.org, Name <c@d.org>, "Last,
//! First" <e@f.org>`.

use model::Address;

/// Reads a comma- or semicolon-separated list. An entry that is not an
/// address is an error naming it, so the user can fix that one.
pub fn parse(text: &str) -> Result<Vec<Address>, String> {
    let mut out = Vec::new();
    for entry in split_entries(text) {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        out.push(parse_entry(entry).ok_or_else(|| format!("not an address: {entry}"))?);
    }
    Ok(out)
}

/// The list as it is shown in a field: `Name <a@b.org>` or just the address.
pub fn format(addresses: &[Address]) -> String {
    addresses.iter().map(format_one).collect::<Vec<_>>().join(", ")
}

fn format_one(address: &Address) -> String {
    match address.name.as_deref().filter(|name| !name.is_empty()) {
        Some(name) if needs_quotes(name) => format!("\"{}\" <{}>", name.replace('"', "\\\""), address.email),
        Some(name) => format!("{name} <{}>", address.email),
        None => address.email.clone(),
    }
}

fn needs_quotes(name: &str) -> bool {
    name.chars().any(|ch| matches!(ch, ',' | ';' | '"' | '<' | '>' | '@' | '(' | ')'))
}

/// Cuts at commas and semicolons that are not inside quotes or angle
/// brackets.
fn split_entries(text: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let (mut quoted, mut escaped, mut angle) = (false, false, false);
    for ch in text.chars() {
        match ch {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '<' if !quoted => angle = true,
            '>' if !quoted => angle = false,
            ',' | ';' if !quoted && !angle => {
                entries.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    entries.push(current);
    entries
}

fn parse_entry(entry: &str) -> Option<Address> {
    if let Some(open) = entry.rfind('<') {
        let close = entry.rfind('>')?;
        if close < open || !entry[close + 1..].trim().is_empty() {
            return None;
        }
        let email = entry[open + 1..close].trim();
        let name = unquote(entry[..open].trim());
        return valid_email(email).then(|| Address {
            name: (!name.is_empty()).then_some(name),
            email: email.to_string(),
        });
    }
    valid_email(entry).then(|| Address::bare(entry))
}

fn unquote(name: &str) -> String {
    let inner = name.strip_prefix('"').and_then(|rest| rest.strip_suffix('"')).unwrap_or(name);
    inner.replace("\\\"", "\"").replace("\\\\", "\\")
}

/// One `@`, something before it and something after, and nothing in the
/// address that could break a header.
fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && !email.chars().any(|ch| ch.is_whitespace() || ch.is_control() || matches!(ch, '<' | '>' | ',' | ';' | '"'))
}

#[cfg(test)]
include!("address_list.tests.rs");
