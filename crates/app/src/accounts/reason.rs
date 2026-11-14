//! Why an account file was refused, in words that are safe to show.
//!
//! The engine's message for text that is not valid TOML quotes the line that
//! was wrong, and that line can hold the password. The footer and the error
//! line never get more than the place and the kind of the fault: the quoted
//! line and the offending value are cut.

use engine::ConfigError;

/// Words in a key that mean its value is a secret.
const SECRET_KEYS: [&str; 3] = ["password", "secret", "token"];

/// The reason for `error`, with nothing quoted from the file.
pub fn reason_of(error: &ConfigError) -> String {
    match error {
        ConfigError::Parse(text) => format!("the account file is not valid: {}", short_parse(text)),
        other => other.to_string(),
    }
}

/// The location line and the closing message of a TOML error, which sit
/// around the quoted source line.
fn short_parse(text: &str) -> String {
    let lines: Vec<&str> = text.lines().map(str::trim_end).filter(|line| !line.trim().is_empty()).collect();
    let (Some(first), Some(last)) = (lines.first(), lines.last()) else {
        return "it could not be read".to_string();
    };
    if lines.len() == 1 {
        return (*first).to_string();
    }
    if quotes_a_value(last) && secret_line(&lines) {
        return format!("{first}: the value is not usable (it is not shown here)");
    }
    format!("{first}: {last}")
}

/// Whether a message repeats part of the file: serde's wording for a value of
/// the wrong type or out of range puts it in quotes or backticks.
fn quotes_a_value(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    (lower.starts_with("invalid") || lower.starts_with("unknown")) && (message.contains('`') || message.contains('"'))
}

/// Whether the quoted source line of the error belongs to a secret key.
fn secret_line(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let Some((_, source)) = line.split_once('|') else {
            return false;
        };
        let key = source.split('=').next().unwrap_or("").trim().to_ascii_lowercase();
        !source.trim().is_empty() && source.contains('=') && SECRET_KEYS.iter().any(|word| key.contains(word))
    })
}

#[cfg(test)]
include!("reason.tests.rs");
