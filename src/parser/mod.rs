//! Top-level `parse_temporal` dispatcher (§3.1, §5).

pub(crate) mod date;
mod datetime;
pub mod duration;
pub(crate) mod time;

use crate::context::TemporalContext;
use crate::error::TemporalError;
use crate::types::TemporalExpr;

/// Parse the raw content of a temporal smart literal.
///
/// The Dolfin parser hands the content of a `(...)` literal here, e.g.
/// `"June 1st 2026"` or `"01/06/2026 as d/m/y"`. Type is inferred by trying the
/// productions in §5 order: DateTime → Date → Time → Duration.
pub fn parse_temporal(
    input: &str,
    context: &TemporalContext,
) -> Result<TemporalExpr, TemporalError> {
    let (value, mask) = split_mask(input.trim());

    // 1. DateTime — needs both a date part and a time part.
    if let Some(res) = datetime::try_datetime(value, mask, context) {
        return res.map(TemporalExpr::DateTime);
    }

    // 2. Date. Remember a "clearly a date but malformed" error to surface last.
    let mut date_err: Option<TemporalError> = None;
    match date::try_date(value, mask, context) {
        Some(Ok(d)) => return Ok(TemporalExpr::Date(d)),
        Some(Err(e)) => date_err = Some(e),
        None => {}
    }

    // 3. Time.
    if let Some(res) = time::try_time(value, context) {
        return res.map(TemporalExpr::Time);
    }

    // 4. Duration. A mask makes no sense for a duration, so require none.
    if mask.is_none()
        && let Some(res) = duration::try_duration(value)
    {
        return res.map(TemporalExpr::Duration);
    }

    Err(date_err.unwrap_or_else(|| TemporalError::ParseError {
        token: input.to_string(),
        reason: "unrecognised temporal literal".into(),
    }))
}

/// Split a trailing/embedded ` as <mask>` off the literal. The mask always
/// describes the numeric date, even in a datetime where it trails the time
/// (`01/06/2026 14:30 as d/m/y`, §4.4). Returns (value, mask?).
fn split_mask(input: &str) -> (&str, Option<&str>) {
    // Match the last " as " so a value like "01/06/2026 14:30 as d/m/y" splits
    // cleanly at the mask keyword.
    if let Some(pos) = input.rfind(" as ") {
        let value = input[..pos].trim();
        let mask = input[pos + 4..].trim();
        if !mask.is_empty() {
            return (value, Some(mask));
        }
    }
    (input, None)
}
