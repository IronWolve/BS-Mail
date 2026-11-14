//! Editing an account keeps what the form cannot say.
//!
//! The form writes only the keys it has a row for. An account file can hold
//! more (custom sign-in endpoints, a poll interval, separate outgoing
//! credentials), and an edit must not throw those away: they are carried over
//! from the old file, and the form's text decides everything else.

/// The keys the form writes or leaves out on purpose. Whatever it does not
/// write for one of these is gone after an edit; any other key of the old
/// file stays.
const FORM_KEYS: [&str; 19] = [
    "id",
    "name",
    "address",
    "imap_host",
    "imap_port",
    "security",
    "username",
    "password",
    "auth",
    "oauth_provider",
    "oauth_client_id",
    "oauth_client_secret",
    "trust",
    "read_only",
    "smtp_host",
    "smtp_port",
    "smtp_security",
    "smtp_trust",
    "save_sent",
];

/// `new_text` with the keys of `old_text` that the form cannot express added.
/// Text that cannot be read as TOML on either side is returned as the form
/// made it.
pub fn carry_over(old_text: &str, new_text: &str) -> String {
    let (Ok(old), Ok(mut new)) = (toml::from_str::<toml::Table>(old_text), toml::from_str::<toml::Table>(new_text)) else {
        return new_text.to_string();
    };
    let mut carried = false;
    for (key, value) in old {
        if FORM_KEYS.contains(&key.as_str()) || new.contains_key(&key) {
            continue;
        }
        new.insert(key, value);
        carried = true;
    }
    if !carried {
        return new_text.to_string();
    }
    toml::to_string(&new).unwrap_or_else(|_| new_text.to_string())
}

#[cfg(test)]
include!("merge.tests.rs");
