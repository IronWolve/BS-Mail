//! What the server said it can do in its answer to EHLO.

use super::reply::Reply;

/// The extensions this program looks at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerCaps {
    pub starttls: bool,
    /// The sign-in methods offered, upper-cased. Empty: none offered.
    pub auth: Vec<String>,
    pub eight_bit: bool,
    /// The largest message the server says it takes; `Some(0)` means it
    /// named no limit.
    pub size: Option<u64>,
}

impl ServerCaps {
    /// Reads the capability lines of an EHLO answer (all but the first line,
    /// which is the server's greeting).
    pub fn from_ehlo(reply: &Reply) -> Self {
        let mut caps = Self::default();
        for line in reply.lines.iter().skip(1) {
            let mut words = line.split_whitespace();
            let keyword = words.next().unwrap_or("").to_ascii_uppercase();
            match keyword.as_str() {
                "STARTTLS" => caps.starttls = true,
                "8BITMIME" => caps.eight_bit = true,
                "SIZE" => caps.size = Some(words.next().and_then(|n| n.parse().ok()).unwrap_or(0)),
                // Some servers write `AUTH=PLAIN LOGIN` as well.
                "AUTH" | "AUTH=" => add_methods(&mut caps.auth, words),
                other if other.starts_with("AUTH=") => {
                    let first = other["AUTH=".len()..].to_owned();
                    add_methods(&mut caps.auth, std::iter::once(first.as_str()).chain(words));
                }
                _ => {}
            }
        }
        caps
    }

    pub fn offers_auth(&self, method: &str) -> bool {
        self.auth.iter().any(|offered| offered == method)
    }
}

/// Adds sign-in method names, upper-cased, each once.
fn add_methods<'a>(known: &mut Vec<String>, names: impl Iterator<Item = &'a str>) {
    for name in names {
        let name = name.to_ascii_uppercase();
        if !known.contains(&name) {
            known.push(name);
        }
    }
}

#[cfg(test)]
include!("caps.tests.rs");
