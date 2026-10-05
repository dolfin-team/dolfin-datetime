//! File-level defaults passed to every `parse_temporal` call (§3.5, §7).

use crate::error::TemporalError;
use crate::timezone::{resolve_named, resolve_named_at};
use crate::types::UtcOffset;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Which calendar field a numeric-date position holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DateField {
    Year,
    Month,
    Day,
}

/// Default field order and separator for ambiguous numeric dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DateLocale {
    /// The three fields in the order they appear in numeric dates.
    pub order: [DateField; 3],
    /// The separator character expected in numeric dates.
    pub separator: char,
}

/// A file-level timezone default.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Timezone {
    /// A fixed offset, e.g. `+02:00`.
    Offset(UtcOffset),
    /// An IANA timezone name, e.g. `"Europe/Brussels"`, resolved to a
    /// `UtcOffset` at parse time.
    Named(String),
}

impl Timezone {
    /// Resolve this timezone to a concrete UTC offset.
    pub fn resolve(&self) -> Result<UtcOffset, TemporalError> {
        match self {
            Timezone::Offset(o) => Ok(*o),
            Timezone::Named(name) => resolve_named(name),
        }
    }

    /// Resolve to the offset in force at the wall-clock instant `local_secs`
    /// (seconds since 1970-01-01T00:00 local time); DST-aware for IANA names.
    pub fn resolve_at(&self, local_secs: i64) -> Result<UtcOffset, TemporalError> {
        match self {
            Timezone::Offset(o) => Ok(*o),
            Timezone::Named(name) => resolve_named_at(name, local_secs),
        }
    }
}

/// File-level defaults. Constructed once per Dolfin source file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TemporalContext {
    /// Default field order for ambiguous numeric dates. `None` means ambiguous
    /// numerics without an `as` mask are a parse error.
    pub locale: Option<DateLocale>,

    /// Default timezone applied to times/datetimes with no inline offset.
    /// `None` means timezone-naive output.
    pub timezone: Option<Timezone>,
}

impl TemporalContext {
    /// No locale, no timezone — strictest mode.
    pub fn strict() -> Self {
        TemporalContext {
            locale: None,
            timezone: None,
        }
    }

    /// Convenience: a locale like `"d/m/y"` (separator taken from `sep`) and a
    /// named IANA timezone.
    pub fn with_locale_and_tz(order: &str, sep: char, tz: &str) -> Result<Self, TemporalError> {
        let locale = parse_locale_order(order, sep)?;
        // Validate the timezone name eagerly so callers fail fast.
        let timezone = Timezone::Named(tz.to_string());
        timezone.resolve()?;
        Ok(TemporalContext {
            locale: Some(locale),
            timezone: Some(timezone),
        })
    }
}

/// Parse a `d/m/y`-style order string into a `DateLocale`. The separator inside
/// `order` (if any) is ignored in favour of the explicit `sep`; only the field
/// letters are read.
pub fn parse_locale_order(order: &str, sep: char) -> Result<DateLocale, TemporalError> {
    let fields: Vec<DateField> = order
        .chars()
        .filter_map(|c| match c.to_ascii_lowercase() {
            'y' => Some(Some(DateField::Year)),
            'm' => Some(Some(DateField::Month)),
            'd' => Some(Some(DateField::Day)),
            c if c.is_alphanumeric() => Some(None), // unknown letter → error below
            _ => None,                              // separators skipped
        })
        .map(|opt| opt.ok_or(()))
        .collect::<Result<Vec<_>, ()>>()
        .map_err(|_| TemporalError::ParseError {
            token: order.to_string(),
            reason: "locale order must use only y, m, d".into(),
        })?;

    if fields.len() != 3 {
        return Err(TemporalError::ParseError {
            token: order.to_string(),
            reason: "locale order must have exactly three fields (y, m, d)".into(),
        });
    }
    Ok(DateLocale {
        order: [fields[0], fields[1], fields[2]],
        separator: sep,
    })
}
