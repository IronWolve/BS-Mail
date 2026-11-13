//! Fills a `%`-pattern such as `Today %H:%M` from calendar fields. Supports
//! `%Y %y %m %d %e %H %I %M %S %p %b %B %a %A %z %%`; month and day names are
//! English. Any other code is copied through unchanged.

use std::fmt::Write;

use super::civil_time::CivilTime;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// Renders `pattern` for `time`.
pub fn format_pattern(pattern: &str, time: &CivilTime) -> String {
    let mut out = String::with_capacity(pattern.len() + 8);
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some(code) => push_field(&mut out, code, time),
            None => out.push('%'),
        }
    }
    out
}

fn push_field(out: &mut String, code: char, t: &CivilTime) {
    // Writing into a String cannot fail.
    let _ = match code {
        'Y' => write!(out, "{:04}", t.year),
        'y' => write!(out, "{:02}", t.year.rem_euclid(100)),
        'm' => write!(out, "{:02}", t.month),
        'd' => write!(out, "{:02}", t.day),
        'e' => write!(out, "{:>2}", t.day),
        'H' => write!(out, "{:02}", t.hour),
        'I' => write!(out, "{:02}", twelve_hour(t.hour)),
        'M' => write!(out, "{:02}", t.minute),
        'S' => write!(out, "{:02}", t.second),
        'p' => write!(out, "{}", if t.hour < 12 { "AM" } else { "PM" }),
        'b' => write!(out, "{}", &name(&MONTHS, usize::from(t.month).wrapping_sub(1))[..3]),
        'B' => write!(out, "{}", name(&MONTHS, usize::from(t.month).wrapping_sub(1))),
        'a' => write!(out, "{}", &name(&WEEKDAYS, usize::from(t.weekday))[..3]),
        'A' => write!(out, "{}", name(&WEEKDAYS, usize::from(t.weekday))),
        'z' => write!(out, "{}", numeric_offset(t.offset_minutes)),
        '%' => write!(out, "%"),
        other => write!(out, "%{other}"),
    };
}

fn twelve_hour(hour: u8) -> u8 {
    match hour % 12 {
        0 => 12,
        h => h,
    }
}

/// A name from a fixed table; a bad index (not reachable from `CivilTime`)
/// gives `"???"`.
fn name<'a>(table: &[&'a str], index: usize) -> &'a str {
    table.get(index).copied().unwrap_or("???")
}

/// `+hhmm` or `-hhmm`.
fn numeric_offset(offset_minutes: i16) -> String {
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let magnitude = offset_minutes.unsigned_abs();
    format!("{sign}{:02}{:02}", magnitude / 60, magnitude % 60)
}

#[cfg(test)]
include!("time_pattern.tests.rs");
