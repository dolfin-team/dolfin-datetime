//! Datetime-expression combinator (§4.4): a date part, a comma-or-space
//! separator, and a time part. The `as` mask (already split off upstream)
//! applies to the date part only.

use crate::context::TemporalContext;
use crate::error::TemporalError;
use crate::parser::date::try_date;
use crate::parser::time::try_time;
use crate::types::DateTime;

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
    let time = match try_time(time_str, ctx) {
        Some(Ok(t)) => t,
        Some(Err(e)) => return Some(Err(e)),
        None => return None,
    };
    Some(Ok(DateTime { date, time }))
}
