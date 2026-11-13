//! The moment the screen treats as now, for Today, Yesterday and clock times.

use chrono::Local;
use model::MailDate;

/// The system clock with this machine's time zone, so Today and Yesterday
/// turn over at local midnight.
pub fn system_now() -> MailDate {
    let now = Local::now();
    let minutes = now.offset().local_minus_utc() / 60;
    MailDate {
        unix: now.timestamp(),
        offset_minutes: i16::try_from(minutes).unwrap_or(0),
    }
}

#[cfg(test)]
include!("clock.tests.rs");
