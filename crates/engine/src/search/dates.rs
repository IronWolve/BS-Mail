//! The days `before:` and `after:` take: `YYYY-MM-DD`.

use super::ast::Date;
use super::error::SearchError;

/// Month names as the server writes them in a search date.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Date {
    /// The day as a search key takes it: `5-Feb-2025`.
    pub fn imap(&self) -> String {
        let month = MONTHS[usize::from(self.month.clamp(1, 12)) - 1];
        format!("{}-{month}-{}", self.day, self.year)
    }
}

/// Reads `YYYY-MM-DD`. `field` is the name used in the sentence when the
/// text is not a day.
pub fn parse_date(field: &str, text: &str) -> Result<Date, SearchError> {
    let wrong = || {
        SearchError::new(format!(
            "{field}: needs a day written YYYY-MM-DD, such as {field}:2025-02-05; \"{text}\" is not one"
        ))
    };
    let parts: Vec<&str> = text.split('-').collect();
    let [year, month, day] = parts[..] else {
        return Err(wrong());
    };
    let digits = |part: &str, len: usize| part.len() == len && part.bytes().all(|b| b.is_ascii_digit());
    if !digits(year, 4) || !digits(month, 2) || !digits(day, 2) {
        return Err(wrong());
    }
    let (year, month, day) = (
        year.parse::<u16>().map_err(|_| wrong())?,
        month.parse::<u8>().map_err(|_| wrong())?,
        day.parse::<u8>().map_err(|_| wrong())?,
    );
    if year < 1900 || !(1..=12).contains(&month) || day == 0 || day > days_in(year, month) {
        return Err(SearchError::new(format!("{field}: {text} is not a day on the calendar")));
    }
    Ok(Date { year, month, day })
}

fn days_in(year: u16, month: u8) -> u8 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
include!("dates.tests.rs");
