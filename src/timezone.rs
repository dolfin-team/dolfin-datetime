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
/// the `iana-tz` feature) an IANA name like `Europe/Brussels`. With no instant
/// to go by, an IANA name resolves to its offset at the Unix epoch (standard
/// time); use [`resolve_named_at`] when the wall-clock instant is known.
pub fn resolve_named(name: &str) -> Result<UtcOffset, TemporalError> {
    resolve_named_at(name, 0)
}

/// Like [`resolve_named`], but an IANA name resolves to the offset in force at
/// the wall-clock instant `local_secs` (seconds since 1970-01-01T00:00 local
/// time), so daylight saving time is honoured.
pub fn resolve_named_at(name: &str, local_secs: i64) -> Result<UtcOffset, TemporalError> {
    let trimmed = name.trim();

    if let Some(off) = parse_fixed_offset(trimmed) {
        return Ok(off);
    }
    if let Some(off) = resolve_abbrev(trimmed) {
        return Ok(off);
    }
    resolve_iana(trimmed, local_secs)
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
fn resolve_iana(name: &str, local_secs: i64) -> Result<UtcOffset, TemporalError> {
    let tz = tzdb::tz_by_name(name)
        .ok_or_else(|| TemporalError::UnknownTimezone(name.to_string()))?;
    let offset_at = |unix: i64| {
        tz.find_local_time_type(unix)
            .map(|t| i64::from(t.ut_offset()))
            .map_err(|_| TemporalError::UnknownTimezone(name.to_string()))
    };
    // Wall clock -> UTC needs the offset we are looking for: guess with the
    // offset at the wall clock read as UTC, then take the offset at the
    // resulting instant (exact except inside a DST gap or overlap).
    let secs = offset_at(local_secs - offset_at(local_secs)?)?;
    Ok(UtcOffset {
        total_minutes: (secs / 60) as i32,
    })
}

#[cfg(not(feature = "iana-tz"))]
fn resolve_iana(name: &str, _local_secs: i64) -> Result<UtcOffset, TemporalError> {
    Err(TemporalError::UnknownTimezone(name.to_string()))
}
