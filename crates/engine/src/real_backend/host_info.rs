//! The machine the engine runs on, for the account strip's host segment.

use std::fs;

/// Short host name and one line naming the system.
pub fn host_info() -> (String, String) {
    (host_name(), system_line())
}

fn host_name() -> String {
    let candidates = [
        fs::read_to_string("/proc/sys/kernel/hostname").ok(),
        fs::read_to_string("/etc/hostname").ok(),
        std::env::var("HOSTNAME").ok(),
        std::env::var("COMPUTERNAME").ok(),
    ];
    candidates
        .into_iter()
        .flatten()
        .map(|raw| short_host_name(&raw))
        .find(|name| !name.is_empty())
        .unwrap_or_else(|| "localhost".to_owned())
}

/// The name up to its first dot, trimmed.
pub fn short_host_name(raw: &str) -> String {
    raw.trim().split('.').next().unwrap_or("").trim().to_owned()
}

fn system_line() -> String {
    let kernel = |name: &str| {
        fs::read_to_string(format!("/proc/sys/kernel/{name}"))
            .ok()
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty())
    };
    match (kernel("ostype"), kernel("osrelease")) {
        (Some(kind), Some(release)) => format!("{kind} {release}"),
        _ => format!("{} {}", os_label(std::env::consts::OS), std::env::consts::ARCH),
    }
}

fn os_label(os: &str) -> &str {
    match os {
        "macos" => "macOS",
        "windows" => "Windows",
        "linux" => "Linux",
        other => other,
    }
}

#[cfg(test)]
include!("host_info.tests.rs");
