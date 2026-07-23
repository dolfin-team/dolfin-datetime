use dolfin_datetime::{
    apply, apply_chain, parse_duration, parse_temporal, Op, TemporalContext, TemporalError,
    TemporalExpr,
};

fn parse(input: &str) -> TemporalExpr {
    parse_temporal(input, &TemporalContext::strict()).unwrap()
}

fn dur(input: &str) -> TemporalExpr {
    TemporalExpr::Duration(parse_duration(input).unwrap())
}

#[test]
fn date_plus_duration() {
    // 2000-01-01 + 7d
    let d = parse("2000-01-01 as y-m-d");
    let r = apply(&d, Op::Add, &dur("7d")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2000-01-08");
}

#[test]
fn date_minus_duration() {
    let d = parse("2000-01-08 as y-m-d");
    let r = apply(&d, Op::Sub, &dur("7d")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2000-01-01");
}

#[test]
fn date_minus_date_is_duration() {
    // June 1st 2026 - Jan 1st 2025 = P1Y5M
    let a = parse("June 1st 2026");
    let b = parse("Jan 1st 2025");
    let r = apply(&a, Op::Sub, &b).unwrap();
    assert_eq!(r.expr.to_xsd(), "P1Y5M");
}

#[test]
fn date_minus_date_reversed_still_positive() {
    // §9.5: operand order does not change the magnitude or sign.
    let a = parse("June 1st 2026");
    let b = parse("Jan 1st 2025");
    let r = apply(&b, Op::Sub, &a).unwrap();
    assert_eq!(r.expr.to_xsd(), "P1Y5M");
}

#[test]
fn date_minus_subday_duration_keeps_day() {
    // Subtracting an hour from a bare Date must not roll back a whole day.
    let d = parse("2000-06-15 as y-m-d");
    let r = apply(&d, Op::Sub, &dur("1h")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2000-06-15");
}

#[test]
fn datetime_minus_datetime_no_borrow() {
    let a = parse("June 1st 2026, 14:30");
    let b = parse("June 1st 2026, 12:00");
    let r = apply(&a, Op::Sub, &b).unwrap();
    assert_eq!(r.expr.to_xsd(), "PT2H30M");
}

#[test]
fn datetime_minus_datetime_with_day_borrow() {
    let a = parse("June 2nd 2026, 1:00");
    let b = parse("June 1st 2026, 23:00");
    let r = apply(&a, Op::Sub, &b).unwrap();
    assert_eq!(r.expr.to_xsd(), "PT2H");
}

#[test]
fn duration_plus_duration() {
    let r = apply(&dur("7d"), Op::Add, &dur("1h 40min")).unwrap();
    assert_eq!(r.expr.to_xsd(), "P7DT1H40M");
}

#[test]
fn duration_minus_duration_negative() {
    let r = apply(&dur("1h"), Op::Sub, &dur("2h")).unwrap();
    assert_eq!(r.expr.to_xsd(), "-PT1H");
}

#[test]
fn time_plus_duration_wraps() {
    let t = parse("23:00");
    let r = apply(&t, Op::Add, &dur("2h")).unwrap();
    assert_eq!(r.expr.to_xsd(), "01:00:00");
}

#[test]
fn datetime_plus_duration() {
    let dt = parse("June 1st 2026, 2:30 PM");
    let r = apply(&dt, Op::Add, &dur("7d")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2026-06-08T14:30:00");
}

#[test]
fn chaining_left_to_right() {
    // (1st Jan 2000, 12:54 AM UTC) - 7d - 1h + 40min
    let start = parse("1st Jan 2000, 12:54 AM UTC");
    let rest = [
        (Op::Sub, dur("7d")),
        (Op::Sub, dur("1h")),
        (Op::Add, dur("40min")),
    ];
    let r = apply_chain(&start, &rest).unwrap();
    // 00:54 - 1h + 40min = 00:34, date -7d = 1999-12-25
    assert_eq!(r.expr.to_xsd(), "1999-12-25T00:34:00+00:00");
}

#[test]
fn type_mismatch_date_plus_date() {
    let a = parse("June 1st 2026");
    let b = parse("June 1st 2025");
    match apply(&a, Op::Add, &b) {
        Err(TemporalError::TypeMismatch { .. }) => {}
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn type_mismatch_time_minus_date() {
    let t = parse("14:30");
    let d = parse("June 1st 2026");
    match apply(&t, Op::Sub, &d) {
        Err(TemporalError::TypeMismatch { .. }) => {}
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}

#[test]
fn month_overflow_clamps_non_leap() {
    // Jan 31 2026 + 1mo -> Feb 28 2026 with warning
    let d = parse("Jan 31st 2026");
    let r = apply(&d, Op::Add, &dur("1mo")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2026-02-28");
    assert!(matches!(
        r.warnings.as_slice(),
        [TemporalError::MonthOverflow(_, 31)]
    ));
}

#[test]
fn month_overflow_clamps_leap() {
    let d = parse("Jan 31st 2024");
    let r = apply(&d, Op::Add, &dur("1mo")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2024-02-29");
}

#[test]
fn month_subtract_clamps() {
    let d = parse("Mar 31st 2026");
    let r = apply(&d, Op::Sub, &dur("1mo")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2026-02-28");
}

#[test]
fn year_boundary_add_day() {
    let d = parse("Dec 31st 2025");
    let r = apply(&d, Op::Add, &dur("1d")).unwrap();
    assert_eq!(r.expr.to_xsd(), "2026-01-01");
}
