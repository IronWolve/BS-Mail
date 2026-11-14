//! The sizes `size:` takes: `>1M` or `<500K`.

use super::error::SearchError;

/// Which side of the size the message must be on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Larger(u32),
    Smaller(u32),
}

/// Reads `>` or `<` followed by a whole number and an optional unit: `B`,
/// `K`/`KB`, `M`/`MB` or `G`/`GB`, in any letter case. A unit is 1024 times
/// the one before.
pub fn parse_size(text: &str) -> Result<Bound, SearchError> {
    let wrong = || {
        SearchError::new(format!(
            "size: needs > or < and a size, such as size:>1M or size:<500K; \"{text}\" is not one"
        ))
    };
    let larger = match text.chars().next() {
        Some('>') => true,
        Some('<') => false,
        _ => return Err(wrong()),
    };
    let rest = &text[1..];
    let split = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    let (digits, unit) = rest.split_at(split);
    let count: u64 = digits.parse().map_err(|_| wrong())?;
    let factor: u64 = match unit.to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" => 1 << 10,
        "m" | "mb" => 1 << 20,
        "g" | "gb" => 1 << 30,
        _ => return Err(wrong()),
    };
    let bytes = count
        .checked_mul(factor)
        .and_then(|total| u32::try_from(total).ok())
        .ok_or_else(|| SearchError::new(format!("size: {} is too large to search for", &text[1..])))?;
    Ok(if larger { Bound::Larger(bytes) } else { Bound::Smaller(bytes) })
}

#[cfg(test)]
include!("sizes.tests.rs");
