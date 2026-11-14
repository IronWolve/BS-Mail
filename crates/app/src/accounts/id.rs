//! Account ids: what a folder name may be, and how a taken one is made
//! unique.

/// The longest id the engine takes.
pub const MAX_ID_LEN: usize = 64;

/// Whether `id` is something the engine takes as an account id: 1 to 64
/// letters, digits, `-` or `_`. The same words name the account's folder, so
/// nothing that could point elsewhere (a separator, `.`, `..`) gets through.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

/// `base` when nothing has it, else `base-2`, `base-3` and so on: the first
/// that `taken` does not claim. `None` when a thousand of them are taken.
pub fn unique_id(base: &str, taken: impl Fn(&str) -> bool) -> Option<String> {
    if !taken(base) {
        return Some(base.to_string());
    }
    for number in 2..1000u32 {
        let suffix = format!("-{number}");
        let keep = MAX_ID_LEN - suffix.len();
        let stem: String = base.chars().take(keep).collect();
        let candidate = format!("{stem}{suffix}");
        if !taken(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
include!("id.tests.rs");
