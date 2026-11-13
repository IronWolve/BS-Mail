//! Byte counts for the Size column and the storage line: `42K`, `1.2M`,
//! `1.9G / 10G (19%)`.

use model::Quota;
use model::config::SizeStyle;

const SHORT_UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "P"];
const IEC_UNITS: [&str; 6] = [" B", " KiB", " MiB", " GiB", " TiB", " PiB"];

/// A size in the chosen style. `Short` gives `512B`, `42K`, `1.2M`, `10G`:
/// whole numbers up to kilobytes, one decimal below ten of a larger unit.
/// `Iec` gives the same numbers as `42 KiB`; `Bytes` the plain count.
pub fn format_size(bytes: u64, style: SizeStyle) -> String {
    match style {
        SizeStyle::Bytes => bytes.to_string(),
        SizeStyle::Short => with_units(bytes, &SHORT_UNITS),
        SizeStyle::Iec => with_units(bytes, &IEC_UNITS),
    }
}

/// The storage line: used and limit in short sizes, then the percent used,
/// as `1.9G / 10G (19%)`. With no limit only the used size is shown.
pub fn format_quota(quota: Quota) -> String {
    let used = with_units(quota.used, &SHORT_UNITS);
    if quota.limit == 0 {
        return used;
    }
    let limit = with_units(quota.limit, &SHORT_UNITS);
    let half = u128::from(quota.limit) / 2;
    let percent = (u128::from(quota.used) * 100 + half) / u128::from(quota.limit);
    format!("{used} / {limit} ({percent}%)")
}

fn with_units(bytes: u64, units: &[&str; 6]) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while unit + 1 < units.len() && value.round() >= 1024.0 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{}{}", number_text(value, unit), units[unit])
}

/// Whole numbers for bytes and kilobytes and for ten or more of a larger
/// unit; one decimal for under ten of a larger unit (`1.2`, `1.0`).
fn number_text(value: f64, unit: usize) -> String {
    if unit >= 2 {
        let tenths = (value * 10.0).round() / 10.0;
        if tenths < 10.0 {
            return format!("{tenths:.1}");
        }
    }
    format!("{:.0}", value.round())
}

#[cfg(test)]
include!("size_format.tests.rs");
