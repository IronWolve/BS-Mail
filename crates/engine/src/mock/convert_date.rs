//! Turns a TOML date-time with an offset into the model's date type.

use model::MailDate;
use toml::value::{Datetime, Offset};

use super::fixture_error::FieldProblem;

const SECONDS_PER_DAY: i64 = 86_400;

/// A moment with its date, time and offset all present. The offset is kept so
/// the reader can show the sender's zone; fractions of a second are dropped.
pub(crate) fn mail_date(value: &Datetime, field: &str) -> Result<MailDate, FieldProblem> {
    let (Some(date), Some(time), Some(offset)) = (value.date, value.time, value.offset) else {
        return Err(FieldProblem::new(
            field,
            "needs a date, a time and a UTC offset, such as 2025-01-29T21:58:14-05:00",
        ));
    };
    let offset_minutes = match offset {
        Offset::Z => 0,
        Offset::Custom { minutes } => minutes,
    };
    let days = days_from_civil(
        i64::from(date.year),
        i64::from(date.month),
        i64::from(date.day),
    );
    let local = days * SECONDS_PER_DAY
        + i64::from(time.hour) * 3600
        + i64::from(time.minute) * 60
        + i64::from(time.second);
    Ok(MailDate {
        unix: local - i64::from(offset_minutes) * 60,
        offset_minutes,
    })
}

/// Days since 1970-01-01 for a proleptic Gregorian date.
pub(super) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_from_march = (month + 9) % 12;
    let day_of_year = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
include!("convert_date.tests.rs");
