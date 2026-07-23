//! Compile-time temporal arithmetic (§6): type rules, month clamping, and
//! calendar-difference computation. All evaluation is at parse time.

use crate::error::TemporalError;
use crate::parser::date::days_in_month;
use crate::types::{Date, DateTime, Duration, TemporalExpr, Time};

/// The single additive operator (§6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
}

/// The result of an arithmetic step: the value plus any non-fatal warnings
/// (currently only `MonthOverflow`, §6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluated {
    pub expr: TemporalExpr,
    pub warnings: Vec<TemporalError>,
}

impl Evaluated {
    fn plain(expr: TemporalExpr) -> Self {
        Evaluated {
            expr,
            warnings: Vec::new(),
        }
    }
}

/// Apply one binary operator (§6.2 type rules). Any combination not in the
/// table is a `TypeMismatch`.
pub fn apply(lhs: &TemporalExpr, op: Op, rhs: &TemporalExpr) -> Result<Evaluated, TemporalError> {
    use TemporalExpr::*;
    match (lhs, rhs) {
        (Date(a), Duration(d)) => {
            let (date, warns) = date_plus_duration(*a, op, d);
            Ok(Evaluated {
                expr: Date(date),
                warnings: warns,
            })
        }
        (Date(a), Date(b)) if op == Op::Sub => Ok(Evaluated::plain(Duration(date_diff(*a, *b)))),
        (DateTime(a), Duration(d)) => {
            let (dt, warns) = datetime_plus_duration(*a, op, d);
            Ok(Evaluated {
                expr: DateTime(dt),
                warnings: warns,
            })
        }
        (DateTime(a), DateTime(b)) if op == Op::Sub => {
            Ok(Evaluated::plain(Duration(datetime_diff(*a, *b))))
        }
        (Time(a), Duration(d)) => Ok(Evaluated::plain(Time(time_plus_duration(*a, op, d)))),
        (Duration(a), Duration(b)) => Ok(Evaluated::plain(Duration(duration_op(a, op, b)))),
        _ => Err(TemporalError::TypeMismatch {
            op: match op {
                Op::Add => "add".into(),
                Op::Sub => "subtract".into(),
            },
            lhs: lhs.kind().into(),
            rhs: rhs.kind().into(),
        }),
    }
}

/// Evaluate a left-to-right chain of equal-precedence operators (§6.3).
/// `terms` is `[t0, (op1, t1), (op2, t2), …]`.
pub fn apply_chain(
    first: &TemporalExpr,
    rest: &[(Op, TemporalExpr)],
) -> Result<Evaluated, TemporalError> {
    let mut acc = first.clone();
    let mut warnings = Vec::new();
    for (op, term) in rest {
        let step = apply(&acc, *op, term)?;
        warnings.extend(step.warnings);
        acc = step.expr;
    }
    Ok(Evaluated {
        expr: acc,
        warnings,
    })
}

// --- duration decomposition -------------------------------------------------

/// A duration as two commensurable axes: whole months, and seconds. Weeks/days
/// fold into seconds; years/months into months. Sign from the `negative` flag.
fn axes(d: &Duration) -> (i64, i64) {
    let months = d.years as i64 * 12 + d.months as i64;
    let secs = (d.weeks as i64 * 7 + d.days as i64) * 86_400
        + d.hours as i64 * 3600
        + d.minutes as i64 * 60
        + d.seconds as i64;
    if d.negative {
        (-months, -secs)
    } else {
        (months, secs)
    }
}

fn from_axes(mut months: i64, mut secs: i64) -> Duration {
    // Choose an overall sign; for well-formed operands both axes agree.
    let sign = if months != 0 {
        months.signum()
    } else {
        secs.signum()
    };
    let negative = sign < 0;
    if negative {
        months = -months;
        secs = -secs;
    }
    let months = months.max(0);
    let secs = secs.max(0);
    Duration {
        negative,
        years: (months / 12) as u32,
        months: (months % 12) as u32,
        weeks: 0,
        days: (secs / 86_400) as u32,
        hours: ((secs % 86_400) / 3600) as u32,
        minutes: ((secs % 3600) / 60) as u32,
        seconds: (secs % 60) as u32,
    }
}

// --- month / day helpers ----------------------------------------------------

/// Add a signed number of months to a date, clamping the day to the last valid
/// day of the target month (§6.4). Returns the date and any overflow warning.
fn add_months(date: Date, delta_months: i64) -> (Date, Vec<TemporalError>) {
    let total = date.year as i64 * 12 + (date.month as i64 - 1) + delta_months;
    let new_year = total.div_euclid(12) as i32;
    let new_month = (total.rem_euclid(12) + 1) as u8;
    let max_day = days_in_month(new_year, new_month);
    let (day, warnings) = if date.day > max_day {
        (
            max_day,
            vec![TemporalError::MonthOverflow(
                format!("{new_year:04}-{new_month:02}"),
                date.day,
            )],
        )
    } else {
        (date.day, Vec::new())
    };
    (
        Date {
            year: new_year,
            month: new_month,
            day,
        },
        warnings,
    )
}

fn to_time_date(d: Date) -> time::Date {
    time::Date::from_calendar_date(d.year, time::Month::try_from(d.month).unwrap(), d.day)
        .expect("date invariant: only validated dates reach arithmetic")
}

fn from_time_date(d: time::Date) -> Date {
    Date {
        year: d.year(),
        month: u8::from(d.month()),
        day: d.day(),
    }
}

fn add_days(date: Date, days: i64) -> Date {
    from_time_date(to_time_date(date) + time::Duration::days(days))
}

fn sign_of(op: Op) -> i64 {
    match op {
        Op::Add => 1,
        Op::Sub => -1,
    }
}

// --- the individual rules ---------------------------------------------------

fn date_plus_duration(date: Date, op: Op, d: &Duration) -> (Date, Vec<TemporalError>) {
    let s = sign_of(op);
    let (dm, _dsecs) = axes(d);
    let (date, warns) = add_months(date, s * dm);
    // A bare Date has no clock, so only whole day/week components move it;
    // sub-day components (h/m/s) are ignored rather than floored into a day.
    let day_units = d.weeks as i64 * 7 + d.days as i64;
    let days = s * if d.negative { -day_units } else { day_units };
    (add_days(date, days), warns)
}

fn datetime_plus_duration(dt: DateTime, op: Op, d: &Duration) -> (DateTime, Vec<TemporalError>) {
    let s = sign_of(op);
    let (dm, dsecs) = axes(d);
    let (date, warns) = add_months(dt.date, s * dm);

    let base = secs_of_day(dt.time);
    let grand = base + s * dsecs;
    let carry = grand.div_euclid(86_400);
    let secs = grand.rem_euclid(86_400);
    let date = add_days(date, carry);
    (
        DateTime {
            date,
            time: time_from_secs(secs, dt.time),
        },
        warns,
    )
}

fn time_plus_duration(t: Time, op: Op, d: &Duration) -> Time {
    let s = sign_of(op);
    // Only clock components move a Time; it wraps at the 24h boundary (§6, Q4).
    let dsecs = d.hours as i64 * 3600 + d.minutes as i64 * 60 + d.seconds as i64;
    let dsecs = if d.negative { -dsecs } else { dsecs };
    let total = (secs_of_day(t) + s * dsecs).rem_euclid(86_400);
    time_from_secs(total, t)
}

fn duration_op(a: &Duration, op: Op, b: &Duration) -> Duration {
    let (m1, s1) = axes(a);
    let (m2, s2) = axes(b);
    let s = sign_of(op);
    from_axes(m1 + s * m2, s1 + s * s2)
}

fn date_diff(a: Date, b: Date) -> Duration {
    // Always a positive duration: subtract the smaller from the larger. §6.2's
    // table note ("if lhs < rhs, result is negative") conflicts with §9.5's
    // explicit "the result is always a positive duration"; we follow §9.5, the
    // more explicit statement, so operand order does not affect the magnitude.
    let (later, earlier) = if a >= b { (a, b) } else { (b, a) };
    age(later, earlier)
}

fn datetime_diff(a: DateTime, b: DateTime) -> Duration {
    let a_key = (a.date, secs_of_day(a.time));
    let b_key = (b.date, secs_of_day(b.time));
    let (later, earlier) = if a_key >= b_key { (a, b) } else { (b, a) };

    let mut later_secs = secs_of_day(later.time);
    let earlier_secs = secs_of_day(earlier.time);
    let mut later_date = later.date;
    if later_secs < earlier_secs {
        later_secs += 86_400;
        later_date = add_days(later_date, -1);
    }
    let tdiff = later_secs - earlier_secs;

    let mut dur = age(later_date, earlier.date);
    dur.hours = (tdiff / 3600) as u32;
    dur.minutes = ((tdiff % 3600) / 60) as u32;
    dur.seconds = (tdiff % 60) as u32;
    dur
}

/// Calendar difference `later - earlier` in years/months/days (Postgres
/// `age()`-style borrow), both dates assumed `later >= earlier` (§6.5).
fn age(later: Date, earlier: Date) -> Duration {
    let mut years = later.year - earlier.year;
    let mut months = later.month as i32 - earlier.month as i32;
    let mut days = later.day as i32 - earlier.day as i32;

    if days < 0 {
        months -= 1;
        let (by, bm) = if later.month == 1 {
            (later.year - 1, 12u8)
        } else {
            (later.year, later.month - 1)
        };
        days += days_in_month(by, bm) as i32;
    }
    if months < 0 {
        years -= 1;
        months += 12;
    }
    Duration {
        negative: false,
        years: years as u32,
        months: months as u32,
        weeks: 0,
        days: days as u32,
        hours: 0,
        minutes: 0,
        seconds: 0,
    }
}

fn secs_of_day(t: Time) -> i64 {
    t.hour as i64 * 3600 + t.minute as i64 * 60 + t.second as i64
}

fn time_from_secs(secs: i64, template: Time) -> Time {
    let secs = secs.rem_euclid(86_400);
    Time {
        hour: (secs / 3600) as u8,
        minute: ((secs % 3600) / 60) as u8,
        second: (secs % 60) as u8,
        offset: template.offset,
    }
}
