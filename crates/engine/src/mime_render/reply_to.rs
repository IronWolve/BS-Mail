//! The Reply-To header as the addresses an answer should go to.

use mail_parser::Message;
use model::Address;

use crate::text_clean::single_line;

/// Most addresses taken from one header; a longer list is cut.
const MOST: usize = 10;

/// The usable addresses of the Reply-To header: a name with an `@` in it, one
/// run of printable characters, no spaces. A name is cleaned to one line.
pub fn reply_to_of(message: &Message<'_>) -> Vec<Address> {
    let Some(header) = message.reply_to() else {
        return Vec::new();
    };
    header
        .iter()
        .filter_map(|addr| {
            let email = addr.address.as_deref()?.trim();
            usable(email).then(|| Address {
                name: addr.name.as_deref().map(single_line).map(|name| name.trim().to_owned()).filter(|name| !name.is_empty()),
                email: email.to_owned(),
            })
        })
        .take(MOST)
        .collect()
}

fn usable(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.contains('@')
        && !email.chars().any(|ch| ch.is_whitespace() || ch.is_control() || matches!(ch, '<' | '>' | ',' | ';' | '"'))
}

#[cfg(test)]
include!("reply_to.tests.rs");
