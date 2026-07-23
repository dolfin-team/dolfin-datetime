//! Error type for the `dolfin-datetime` crate.

/// All error and warning conditions produced while parsing or evaluating
/// temporal literals. `MonthOverflow` is a non-fatal warning (see §6.4 of the
/// spec) — it is returned in the `warnings` list of an evaluation, never as the
/// hard error of a `Result::Err`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TemporalError {
    #[error("Ambiguous numeric date '{0}': no locale default set and no 'as' mask provided")]
    AmbiguousDate(String),

    #[error("Unknown month name: '{0}'")]
    UnknownMonth(String),

    #[error("Invalid date: {reason}")]
    InvalidDate { reason: String },

    #[error("Invalid time: {reason}")]
    InvalidTime { reason: String },

    #[error("Unknown timezone: '{0}'")]
    UnknownTimezone(String),

    #[error("Separator mismatch: value uses '{value_sep}' but mask uses '{mask_sep}'")]
    SeparatorMismatch { value_sep: char, mask_sep: char },

    #[error("Type error: cannot {op} {lhs} and {rhs}")]
    TypeMismatch { op: String, lhs: String, rhs: String },

    #[error("Ambiguous month arithmetic: {0} has no day {1}; clamped to last day of month")]
    MonthOverflow(String, u8),

    #[error("Parse error at '{token}': {reason}")]
    ParseError { token: String, reason: String },
}
