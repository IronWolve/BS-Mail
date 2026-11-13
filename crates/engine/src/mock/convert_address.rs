//! Reads an address written as a mail header does: `Name <user@host>` or a
//! bare `user@host`.

use model::Address;

/// Parses one address; the error says what is wrong with the text.
pub(crate) fn parse_address(text: &str) -> Result<Address, String> {
    let text = text.trim();
    let (name, email) = match text.split_once('<') {
        Some((name, rest)) => {
            let email = rest
                .strip_suffix('>')
                .ok_or_else(|| format!("`{text}` has `<` but does not end with `>`"))?;
            (name.trim().trim_matches('"').trim(), email.trim())
        }
        None => ("", text),
    };
    check_email(email, text)?;
    Ok(if name.is_empty() {
        Address::bare(email)
    } else {
        Address::named(name, email)
    })
}

fn check_email(email: &str, whole: &str) -> Result<(), String> {
    let valid = match email.split_once('@') {
        Some((user, host)) => {
            !user.is_empty()
                && !host.is_empty()
                && !email.contains(char::is_whitespace)
                && !email.contains(['<', '>', ','])
        }
        None => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!("`{whole}` has no usable user@host address"))
    }
}

#[cfg(test)]
include!("convert_address.tests.rs");
