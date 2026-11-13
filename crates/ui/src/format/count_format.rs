//! Counts with thousands grouping (`1,286`) and the sidebar's count template.

use model::config::AppearanceConfig;

/// The number with a comma between groups of three digits.
pub fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Fills `{n}` in a template such as `[{n}]` with the grouped number.
pub fn format_count(template: &str, n: u64) -> String {
    template.replace("{n}", &group_thousands(n))
}

/// The sidebar's unread count text for a folder, or `None` when it is hidden:
/// the template is empty, or the count is zero and zero counts are hidden.
pub fn sidebar_count(appearance: &AppearanceConfig, n: u32) -> Option<String> {
    if appearance.count_format.is_empty() || (n == 0 && appearance.hide_zero_counts) {
        return None;
    }
    Some(format_count(&appearance.count_format, u64::from(n)))
}

#[cfg(test)]
include!("count_format.tests.rs");
