//! Timezone abbreviation registry and IANA name resolution (§9.3).

use crate::error::TemporalError;
use crate::types::UtcOffset;

/// Fixed-offset abbreviations, resolved to total minutes from UTC.
///
/// Ambiguous abbreviations (e.g. `IST`) are deliberately absent — they are
/// rejected with `UnknownTimezone` so the author uses a full IANA name or an
/// explicit offset (§9.3).
const ABBREVS: &[(&str, i32)] = &[
    ("UTC", 0),
    ("Z", 0),
    ("GMT", 0),
    ("CET", 60),
    ("CEST", 120),
    ("EST", -300),
    ("EDT", -240),
    ("CST", -360),
    ("CDT", -300),
    ("MST", -420),
    ("MDT", -360),
    ("PST", -480),
    ("PDT", -420),
];

/// Resolve a bare timezone token (`UTC`, `Z`, `CET`, …) to a `UtcOffset`.
/// Case-insensitive. Returns `None` if the token is not a known abbreviation
/// (it may still be an inline `±HH:MM` offset handled by the time parser).
pub fn resolve_abbrev(token: &str) -> Option<UtcOffset> {
    let up = token.to_ascii_uppercase();
    ABBREVS
        .iter()
        .find(|(name, _)| *name == up)
        .map(|(_, mins)| UtcOffset {
            total_minutes: *mins,
        })
}

/// Resolve a file-level timezone name to a `UtcOffset`.
///
/// Accepts, in order: a fixed offset `±HH:MM`, a known abbreviation, or (with
/// the `iana-tz` feature) an IANA name like `Europe/Brussels`. IANA names are
/// resolved to their *current* offset; DST for historical instants is out of
/// scope for compile-time literal serialisation.
pub fn resolve_named(name: &str) -> Result<UtcOffset, TemporalError> {
    let trimmed = name.trim();

    if let Some(off) = parse_fixed_offset(trimmed) {
        return Ok(off);
    }
    if let Some(off) = resolve_abbrev(trimmed) {
        return Ok(off);
    }
    resolve_iana(trimmed)
}

/// Parse a `±HH:MM` fixed offset. Returns `None` if the shape does not match.
fn parse_fixed_offset(s: &str) -> Option<UtcOffset> {
    let bytes = s.as_bytes();
    let sign = match bytes.first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &s[1..];
    let (h, m) = rest.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    let hh: i32 = h.parse().ok()?;
    let mm: i32 = m.parse().ok()?;
    if hh > 23 || mm > 59 {
        return None;
    }
    Some(UtcOffset {
        total_minutes: sign * (hh * 60 + mm),
    })
}

#[cfg(feature = "iana-tz")]
fn resolve_iana(name: &str) -> Result<UtcOffset, TemporalError> {
    let tz = tzdb::tz_by_name(name)
        .ok_or_else(|| TemporalError::UnknownTimezone(name.to_string()))?;
    // Offset at the Unix epoch is a stable, deterministic representative for a
    // compile-time literal that carries no instant of its own.
    let secs = tz
        .find_local_time_type(0)
        .map_err(|_| TemporalError::UnknownTimezone(name.to_string()))?
        .ut_offset();
    Ok(UtcOffset {
        total_minutes: secs / 60,
    })
}

#[cfg(not(feature = "iana-tz"))]
fn resolve_iana(name: &str) -> Result<UtcOffset, TemporalError> {
    Err(TemporalError::UnknownTimezone(name.to_string()))
}
