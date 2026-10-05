//! Datetime-expression combinator (§4.4): a date part, a comma-or-space
//! separator, and a time part. The `as` mask (already split off upstream)
//! applies to the date part only.

use crate::context::TemporalContext;
use crate::error::TemporalError;
use crate::parser::date::try_date;
use crate::parser::time::try_time;
use crate::types::{Date, DateTime, Time};

/// Split `input` into (date-part, time-part). The time part is recognised by
/// the first whitespace-separated token that contains a `:`. Returns `None`
/// when there is no time token or no preceding date part.
fn split_datetime(input: &str) -> Option<(&str, &str)> {
    // Prefer an explicit comma separator (§4.4: "comma-space or plain space").
    if let Some(idx) = input.find(',') {
        let (date, rest) = input.split_at(idx);
        let time = rest[1..].trim();
        if !date.trim().is_empty() && time.contains(':') {
            return Some((date.trim(), time));
        }
    }

    // Otherwise: find the first token bearing a ':' and split there.
    let mut offset = 0;
    for tok in input.split_inclusive(char::is_whitespace) {
        if tok.contains(':') {
            let date = input[..offset].trim();
            let time = input[offset..].trim();
            if date.is_empty() {
                return None;
            }
            return Some((date, time));
        }
        offset += tok.len();
    }
    None
}

/// Try to parse the whole `input` as a datetime. `None` when it does not have
/// both a date and a time part.
pub(crate) fn try_datetime(
    input: &str,
    mask: Option<&str>,
    ctx: &TemporalContext,
) -> Option<Result<DateTime, TemporalError>> {
    let (date_str, time_str) = split_datetime(input.trim())?;

    let date = match try_date(date_str, mask, ctx) {
        Some(Ok(d)) => d,
        Some(Err(e)) => return Some(Err(e)),
        None => return None,
    };
    // Parse the time without the file-level zone: a named zone's offset
    // depends on the date (DST), so it is applied below with the full instant.
    let mut time = match try_time(time_str, &TemporalContext::strict()) {
        Some(Ok(t)) => t,
        Some(Err(e)) => return Some(Err(e)),
        None => return None,
    };
    if let (None, Some(tz)) = (time.offset, &ctx.timezone) {
        match tz.resolve_at(local_secs(date, time)) {
            Ok(off) => time.offset = Some(off),
            Err(e) => return Some(Err(e)),
        }
    }
    Some(Ok(DateTime { date, time }))
}

/// Seconds from 1970-01-01T00:00 to this wall-clock date and time.
fn local_secs(date: Date, time: Time) -> i64 {
    let days = time::Month::try_from(date.month)
        .and_then(|m| time::Date::from_calendar_date(date.year, m, date.day))
        .map(|d| i64::from(d.to_julian_day()) - 2_440_588) // Julian day of 1970-01-01
        .unwrap_or(0); // unreachable: only validated dates get here
    days * 86_400 + i64::from(time.hour) * 3600 + i64::from(time.minute) * 60 + i64::from(time.second)
}
