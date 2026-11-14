//! The id an account gets from its address: a short name safe to use as a
//! folder name and in the account file.

/// The longest id made.
pub const MAX_ID_LEN: usize = 40;

/// Lower case ASCII letters and digits of `address`; every other run of
/// characters becomes one `-`, none at either end; at most `MAX_ID_LEN`
/// characters; `account` when nothing is left. The program, not the form,
/// makes sure no two accounts end up with the same id.
pub fn account_id_for(address: &str) -> String {
    let mut id = String::new();
    let mut gap = false;
    for ch in address.chars() {
        if ch.is_ascii_alphanumeric() {
            if gap && !id.is_empty() {
                id.push('-');
            }
            gap = false;
            id.push(ch.to_ascii_lowercase());
        } else {
            gap = true;
        }
    }
    id.truncate(MAX_ID_LEN);
    let id = id.trim_end_matches('-');
    if id.is_empty() { "account".to_string() } else { id.to_string() }
}

#[cfg(test)]
include!("account_id.tests.rs");
