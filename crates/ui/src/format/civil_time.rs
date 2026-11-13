//! Calendar fields of a point in time, worked out by hand so no date library
//! is needed. Years are proleptic Gregorian.

const SECONDS_PER_DAY: i64 = 86_400;

/// The calendar fields of one instant in one UTC offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilTime {
    pub year: i64,
    /// 1 to 12.
    pub month: u8,
    /// 1 to 31.
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// 0 is Sunday.
    pub weekday: u8,
    /// The offset these fields are in, minutes east of UTC.
    pub offset_minutes: i16,
}

impl CivilTime {
    /// The fields of UTC second `unix` as seen in `offset_minutes` east of UTC.
    pub fn from_unix(unix: i64, offset_minutes: i16) -> Self {
        let local = unix + i64::from(offset_minutes) * 60;
        let days = local.div_euclid(SECONDS_PER_DAY);
        let seconds = local.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            hour: (seconds / 3600) as u8,
            minute: (seconds % 3600 / 60) as u8,
            second: (seconds % 60) as u8,
            // 1970-01-01 was a Thursday.
            weekday: (days + 4).rem_euclid(7) as u8,
            offset_minutes,
        }
    }

    /// Whole days from 1970-01-01 to this local date; two times fall on the
    /// same local day exactly when their day numbers are equal.
    pub fn day_number(&self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }
}

/// Days from 1970-01-01 to the given date (negative before it).
pub fn days_from_civil(year: i64, month: u8, day: u8) -> i64 {
    let month = i64::from(month);
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_from_march = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_from_march + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The date `days` after 1970-01-01 (before it when negative).
pub fn civil_from_days(days: i64) -> (i64, u8, u8) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month as u8, day as u8)
}

/// UTC seconds of a local date and time in `offset_minutes` east of UTC.
pub fn unix_from_civil(
    year: i64,
    month: u8,
    day: u8,
    (hour, minute, second): (u8, u8, u8),
    offset_minutes: i16,
) -> i64 {
    let local = days_from_civil(year, month, day) * SECONDS_PER_DAY
        + i64::from(hour) * 3600
        + i64::from(minute) * 60
        + i64::from(second);
    local - i64::from(offset_minutes) * 60
}

#[cfg(test)]
include!("civil_time.tests.rs");
