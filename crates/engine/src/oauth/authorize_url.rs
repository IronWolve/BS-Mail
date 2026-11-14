//! The link the person opens in a browser, and the percent-encoding that
//! builds it (and the form bodies sent to the token endpoint).

use crate::account_oauth::OAuthSettings;

/// What goes into one authorization link.
pub struct AuthorizeRequest<'a> {
    pub settings: &'a OAuthSettings,
    /// The address the browser comes back to, `http://<host>:<port>` (no path).
    pub redirect_uri: &'a str,
    pub state: &'a str,
    /// The PKCE S256 challenge.
    pub challenge: &'a str,
    /// The account's address, offered to the provider as a hint.
    pub login_hint: &'a str,
}

/// Percent-encodes `text`; the RFC 3986 unreserved characters stay as they
/// are, every other byte becomes `%XX`.
pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// `name=value` pairs joined by `&`, each part percent-encoded.
pub fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// The authorization link: the provider's address with the code flow's
/// parameters, the provider's extra ones, and a login hint.
pub fn build(request: &AuthorizeRequest<'_>) -> String {
    let settings = request.settings;
    let scope = settings.scopes.join(" ");
    let mut pairs: Vec<(&str, &str)> = vec![
        ("response_type", "code"),
        ("client_id", &settings.client_id),
        ("redirect_uri", request.redirect_uri),
        ("scope", &scope),
        ("state", request.state),
        ("code_challenge", request.challenge),
        ("code_challenge_method", "S256"),
    ];
    pairs.extend(settings.extra_params.iter().map(|(n, v)| (n.as_str(), v.as_str())));
    pairs.push(("login_hint", request.login_hint));
    let separator = if settings.auth_url.contains('?') { '&' } else { '?' };
    format!("{}{separator}{}", settings.auth_url, form_body(&pairs))
}

#[cfg(test)]
include!("authorize_url.tests.rs");
