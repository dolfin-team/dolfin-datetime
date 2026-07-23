//! Public output types and their XSD mapping (§2, §3.2, §3.3).

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One parsed temporal expression. Type is inferred from the literal content
/// (§5); the author never writes the XSD type explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TemporalExpr {
    Date(Date),
    Time(Time),
    DateTime(DateTime),
    Duration(Duration),
}

impl TemporalExpr {
    /// The XSD type name, e.g. `"xsd:date"`. Handy for Turtle serialisation.
    pub fn xsd_type(&self) -> &'static str {
        match self {
            TemporalExpr::Date(_) => "xsd:date",
            TemporalExpr::Time(_) => "xsd:time",
            TemporalExpr::DateTime(_) => "xsd:dateTime",
            TemporalExpr::Duration(_) => "xsd:duration",
        }
    }

    /// The lexical value to embed in a Turtle typed literal.
    pub fn to_xsd(&self) -> String {
        match self {
            TemporalExpr::Date(d) => d.to_xsd(),
            TemporalExpr::Time(t) => t.to_xsd(),
            TemporalExpr::DateTime(dt) => dt.to_xsd(),
            TemporalExpr::Duration(du) => du.to_xsd(),
        }
    }

    /// Human-readable kind name, used in `TypeMismatch` error messages.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            TemporalExpr::Date(_) => "Date",
            TemporalExpr::Time(_) => "Time",
            TemporalExpr::DateTime(_) => "DateTime",
            TemporalExpr::Duration(_) => "Duration",
        }
    }
}

/// A calendar date, always normalised to (year, month, day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Date {
    /// Negative values for BCE (proleptic Gregorian).
    pub year: i32,
    /// 1–12.
    pub month: u8,
    /// 1–31.
    pub day: u8,
}

/// A wall-clock time with optional timezone offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    /// 0 if not specified.
    pub second: u8,
    pub offset: Option<UtcOffset>,
}

/// A combined date and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

/// An ISO 8601 duration with all components. Negative durations are expressed
/// via the `negative` flag (§9.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Duration {
    pub negative: bool,
    pub years: u32,
    pub months: u32,
    pub weeks: u32,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

/// A timezone offset from UTC, in total minutes (may be negative).
/// E.g. CET = +60, EST = -300.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UtcOffset {
    pub total_minutes: i32,
}
