//! The SASL XOAUTH2 initial response: `user=<user>^Aauth=Bearer <token>^A^A`.
//! SMTP sends it in base64 (`blob`); the IMAP library base64-encodes whatever
//! its authenticator returns, so IMAP hands it the bytes as they are (`raw`).

use crate::send::base64;

/// The response for `user` and `token`, before base64. A user name or token
/// with a control separator in it cannot be told apart from the format, so it
/// is refused.
pub fn raw(user: &str, token: &str) -> Result<String, String> {
    if user.contains(['\u{1}', '\0']) || token.contains(['\u{1}', '\0']) {
        return Err("the user name or token holds a control character that the sign-in format cannot carry".to_owned());
    }
    Ok(format!("user={user}\u{1}auth=Bearer {token}\u{1}\u{1}"))
}

/// The same response in base64.
pub fn blob(user: &str, token: &str) -> Result<String, String> {
    raw(user, token).map(|text| base64::encode(text.as_bytes()))
}

#[cfg(test)]
include!("xoauth2.tests.rs");
