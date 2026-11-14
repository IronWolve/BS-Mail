//! What the Options screen shows about an account, and what the form starts
//! with when the account is edited. Both are read from the account file's
//! TOML table, leniently, so an account whose file is not usable still has a
//! row to select, edit or remove.

use model::AccountId;
use ui::setup::providers::preset_for;
use ui::{AccountPrefill, AccountSummary};

use super::scan::{Found, text_of};

/// Said when an account signs in through a provider the form has no row for.
const CUSTOM_PROVIDER: &str =
    "This account signs in through a custom provider; change its account.toml by hand.";
/// Said when the account file could not be read as TOML at all.
const UNREADABLE: &str =
    "This account's file cannot be read; remove the account and add it again, or fix its account.toml by hand.";

/// The Options page's facts for one account.
pub fn summary_of(found: &Found) -> AccountSummary {
    let table = found.table.as_ref();
    let get = |key: &str| table.and_then(|table| text_of(table, key));
    let security = get("security").unwrap_or_else(|| "implicit".to_string());
    let incoming = match get("imap_host") {
        Some(host) => format!("{host}:{}, {}", port_of(table, "imap_port", default_imap_port(&security)), security_label(&security)),
        None => "not known".to_string(),
    };
    let outgoing = match get("smtp_host") {
        Some(host) => {
            let how = get("smtp_security").unwrap_or_else(|| "starttls".to_string());
            format!("{host}:{}, {}", port_of(table, "smtp_port", default_smtp_port(&how)), security_label(&how))
        }
        None => "none".to_string(),
    };
    AccountSummary {
        id: found.id.clone(),
        name: found.name.clone(),
        address: found.address.clone(),
        incoming,
        outgoing,
        sign_in: sign_in_label(table),
        folder: found.folder.display().to_string(),
        changes_allowed: table.and_then(|table| table.get("read_only")?.as_bool()) == Some(false),
    }
}

/// Every account's summary, in the order the running accounts are in; any
/// account on disk the order does not name follows, in the order found.
pub fn summaries_in_order(found: &[Found], order: &[AccountId]) -> Vec<AccountSummary> {
    let mut rest: Vec<&Found> = found.iter().collect();
    let mut sorted = Vec::with_capacity(found.len());
    for id in order {
        if let Some(at) = rest.iter().position(|account| account.id == id.0) {
            sorted.push(rest.remove(at));
        }
    }
    sorted.extend(rest);
    sorted.into_iter().map(summary_of).collect()
}

/// The form's starting point for an account. The password and the client
/// secret are part of it; they travel only into the form's hidden fields.
pub fn prefill_of(found: &Found) -> Result<AccountPrefill, String> {
    let table = found.table.as_ref().ok_or(UNREADABLE)?;
    let get = |key: &str| text_of(table, key);
    let security = get("security").unwrap_or_else(|| "implicit".to_string());
    let smtp_security = get("smtp_security").unwrap_or_else(|| "starttls".to_string());
    let address = get("address").unwrap_or_else(|| found.address.clone());
    let browser = browser_provider(table, &address)?;
    Ok(AccountPrefill {
        id: found.id.clone(),
        username: get("username").unwrap_or_else(|| address.clone()),
        address,
        name: get("name").unwrap_or_else(|| found.name.clone()),
        password: get("password").unwrap_or_default(),
        imap_host: get("imap_host").unwrap_or_default(),
        imap_port: port_of(Some(table), "imap_port", default_imap_port(&security)),
        smtp_host: get("smtp_host").unwrap_or_default(),
        smtp_port: port_of(Some(table), "smtp_port", default_smtp_port(&smtp_security)),
        imap_security: security,
        smtp_security,
        browser_provider: browser,
        client_id: get("oauth_client_id").unwrap_or_default(),
        client_secret: get("oauth_client_secret").unwrap_or_default(),
        allow_changes: table.get("read_only").and_then(toml::Value::as_bool) == Some(false),
    })
}

/// The provider the form signs in through for an account that uses the
/// browser, `None` for a password account, and an error for a provider the
/// form cannot take.
fn browser_provider(table: &toml::Table, address: &str) -> Result<Option<String>, String> {
    if text_of(table, "auth").as_deref() != Some("oauth2") {
        return Ok(None);
    }
    let named = text_of(table, "oauth_provider").map(|name| name.to_lowercase());
    let guessed = || preset_for(address).and_then(|preset| preset.oauth).map(str::to_string);
    let provider = match named {
        Some(name) if name == "google" || name == "microsoft" => Some(name),
        Some(_) => None,
        None => guessed(),
    };
    provider.map(Some).ok_or_else(|| CUSTOM_PROVIDER.to_string())
}

fn sign_in_label(table: Option<&toml::Table>) -> String {
    let Some(table) = table else {
        return "not known".to_string();
    };
    if text_of(table, "auth").as_deref() != Some("oauth2") {
        return "password".to_string();
    }
    match text_of(table, "oauth_provider") {
        Some(provider) => format!("web browser ({provider})"),
        None => "web browser".to_string(),
    }
}

fn security_label(word: &str) -> &'static str {
    match word {
        "starttls" => "STARTTLS",
        "none" => "no encryption",
        _ => "TLS",
    }
}

fn default_imap_port(security: &str) -> u16 {
    if security == "implicit" { 993 } else { 143 }
}

fn default_smtp_port(security: &str) -> u16 {
    match security {
        "implicit" => 465,
        "none" => 25,
        _ => 587,
    }
}

fn port_of(table: Option<&toml::Table>, key: &str, default: u16) -> u16 {
    table
        .and_then(|table| table.get(key)?.as_integer())
        .and_then(|number| u16::try_from(number).ok())
        .filter(|port| *port > 0)
        .unwrap_or(default)
}

#[cfg(test)]
include!("summary.tests.rs");
