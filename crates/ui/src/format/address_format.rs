//! Mail addresses as text: the From column's styles and the reader's header
//! form (`Name <address>`, lists joined with a comma).

use model::Address;
use model::config::FromStyle;

/// The address as the From column shows it. A missing display name falls
/// back to the bare address in every style.
pub fn format_address(address: &Address, style: FromStyle) -> String {
    let name = address.name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    match (style, name) {
        (FromStyle::NameAddress, Some(name)) => format!("{name} <{}>", address.email),
        (FromStyle::Name, Some(name)) => name.to_string(),
        _ => address.email.clone(),
    }
}

/// The address in header form: `Name <address>`, the name quoted when it
/// holds a character that would end it early.
pub fn header_address(address: &Address) -> String {
    let name = address.name.as_deref().map(str::trim).filter(|name| !name.is_empty());
    match name {
        Some(name) if needs_quotes(name) => {
            format!("\"{}\" <{}>", name.replace('\\', "\\\\").replace('"', "\\\""), address.email)
        }
        Some(name) => format!("{name} <{}>", address.email),
        None => address.email.clone(),
    }
}

/// Addresses in header form joined with `, `.
pub fn header_address_list(addresses: &[Address]) -> String {
    let parts: Vec<String> = addresses.iter().map(header_address).collect();
    parts.join(", ")
}

fn needs_quotes(name: &str) -> bool {
    name.chars().any(|c| matches!(c, ',' | ';' | ':' | '<' | '>' | '"' | '@'))
}

#[cfg(test)]
include!("address_format.tests.rs");
