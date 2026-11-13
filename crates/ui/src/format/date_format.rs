//! The list's date column: "Today 21:58", "Yesterday 22:14", then older dates,
//! from the `[list]` date options. Works from fields and an injected "now".

use model::MailDate;
use model::config::{DateStyle, ListConfig};

use super::civil_time::CivilTime;
use super::time_pattern::format_pattern;

const SECONDS_PER_DAY: i64 = 86_400;

/// The date patterns of the `[list]` table (section 8.3 of the plan).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateFormat<'a> {
    pub style: DateStyle,
    pub fixed: &'a str,
    pub today: &'a str,
    pub yesterday: &'a str,
    pub older: &'a str,
    /// Used for dates that are not today when the column is narrowed.
    pub short: &'a str,
}

impl<'a> DateFormat<'a> {
    pub fn from_config(list: &'a ListConfig) -> Self {
        Self {
            style: list.date_format,
            fixed: &list.date_fixed,
            today: &list.date_today,
            yesterday: &list.date_yesterday,
            older: &list.date_older,
            short: &list.date_short,
        }
    }
}

/// The text of the date column for a message.
///
/// `now.offset_minutes` is the viewer's own zone: both times are shown and
/// compared in it, not in the sender's zone. With `DateStyle::Relative` a
/// date on the same local day as `now` uses `today`, the day before uses
/// `yesterday`, anything else (including a date in the future) uses `older`.
/// With `DateStyle::Fixed` every date uses `fixed`. When `narrow` is set the
/// column is too tight for the long forms: `short` replaces `yesterday`,
/// `older` and `fixed`.
pub fn format_list_date(
    date: MailDate,
    now: MailDate,
    format: &DateFormat<'_>,
    narrow: bool,
) -> String {
    let local = CivilTime::from_unix(date.unix, now.offset_minutes);
    let today = CivilTime::from_unix(now.unix, now.offset_minutes);
    let days_ago = today.day_number() - local.day_number();
    format_pattern(pick_pattern(format, days_ago, narrow), &local)
}

fn pick_pattern<'a>(format: &DateFormat<'a>, days_ago: i64, narrow: bool) -> &'a str {
    match (format.style, days_ago, narrow) {
        (DateStyle::Relative, 0, _) => format.today,
        (_, _, true) => format.short,
        (DateStyle::Fixed, _, false) => format.fixed,
        (DateStyle::Relative, 1, false) => format.yesterday,
        (DateStyle::Relative, _, false) => format.older,
    }
}

/// True for a message from the last 24 hours, which the list may color as a
/// recent date. A date in the future is not recent.
pub fn is_recent(date: MailDate, now: MailDate) -> bool {
    (0..SECONDS_PER_DAY).contains(&(now.unix - date.unix))
}

#[cfg(test)]
include!("date_format.tests.rs");
