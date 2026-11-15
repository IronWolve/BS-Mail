//! Taking secrets out of words that came from a server. A server that echoes
//! a command back would otherwise put the password in a line shown on screen.

use crate::send::base64;

/// `text` with every copy of each secret replaced, in each spelling a server
/// or a library may repeat it in: as it is, in base64, quoted the IMAP way (a
/// backslash before a quotation mark or a backslash), and in Rust's debug
/// spelling of the first and the quoted one. The longest spellings go first,
/// so one that holds a shorter one is removed whole. Nothing is replaced for
/// an empty secret.
pub fn scrub(text: &str, secrets: &[&str]) -> String {
    let mut forms: Vec<String> = secrets
        .iter()
        .filter(|secret| !secret.is_empty())
        .flat_map(|secret| spellings(secret))
        .collect();
    forms.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    forms.dedup();
    let mut clean = text.to_owned();
    for form in &forms {
        clean = clean.replace(form.as_str(), "***");
    }
    clean
}

/// Every spelling of a non-empty `secret` that is worth removing.
fn spellings(secret: &str) -> Vec<String> {
    let quoted = secret.replace('\\', "\\\\").replace('"', "\\\"");
    let mut forms = vec![
        secret.to_owned(),
        base64::encode(secret.as_bytes()),
        quoted.clone(),
        debug_inside(secret),
        debug_inside(&quoted),
    ];
    forms.retain(|form| !form.is_empty());
    forms
}

/// What Rust's debug form of `text` holds between its quotation marks.
fn debug_inside(text: &str) -> String {
    let shown = format!("{text:?}");
    shown[1..shown.len() - 1].to_owned()
}

/// The base64 of the PLAIN sign-in blob for `user` and `secret`, which is
/// what goes over the wire after `AUTH PLAIN`.
pub fn plain_blob(user: &str, secret: &str) -> String {
    let mut blob = vec![0u8];
    blob.extend_from_slice(user.as_bytes());
    blob.push(0);
    blob.extend_from_slice(secret.as_bytes());
    base64::encode(&blob)
}

#[cfg(test)]
include!("scrub.tests.rs");
