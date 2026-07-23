use dolfin_datetime::{parse_temporal, TemporalContext, TemporalError};

fn err(input: &str) -> TemporalError {
    parse_temporal(input, &TemporalContext::strict()).unwrap_err()
}

#[test]
fn unknown_month() {
    match err("Jono 1st 2026") {
        TemporalError::UnknownMonth(m) => assert_eq!(m, "Jono"),
        other => panic!("expected UnknownMonth, got {other:?}"),
    }
}

#[test]
fn ambiguous_numeric_date() {
    assert!(matches!(err("01/06/2026"), TemporalError::AmbiguousDate(_)));
}

#[test]
fn separator_mismatch() {
    match err("01/06/2026 as d-m-y") {
        TemporalError::SeparatorMismatch {
            value_sep: '/',
            mask_sep: '-',
        } => {}
        other => panic!("expected SeparatorMismatch, got {other:?}"),
    }
}

#[test]
fn invalid_date() {
    assert!(matches!(
        err("Feb 30 2026"),
        TemporalError::InvalidDate { .. }
    ));
}

#[test]
fn invalid_time_hour() {
    assert!(matches!(err("25:00"), TemporalError::InvalidTime { .. }));
}

#[test]
fn invalid_12h_hour() {
    assert!(matches!(err("13:00 PM"), TemporalError::InvalidTime { .. }));
}

#[test]
fn unknown_timezone_abbrev() {
    // IST is deliberately ambiguous → rejected (§9.3).
    match err("14:30 IST") {
        TemporalError::UnknownTimezone(tz) => assert_eq!(tz, "IST"),
        other => panic!("expected UnknownTimezone, got {other:?}"),
    }
}

#[test]
fn garbage_is_error() {
    assert!(parse_temporal("hello world", &TemporalContext::strict()).is_err());
}
