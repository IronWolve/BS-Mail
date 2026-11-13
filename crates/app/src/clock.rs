//! The moment the screen treats as now, for Today, Yesterday and clock times.

use std::time::{SystemTime, UNIX_EPOCH};

use model::MailDate;

/// The system clock, in UTC. The offset is zero until the local time zone is
/// read, so Today and Yesterday turn over at midnight UTC for now.
pub fn system_now() -> MailDate {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX));
    MailDate {
        unix: seconds,
        offset_minutes: 0,
    }
}
