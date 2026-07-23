use dolfin_datetime::{parse_temporal, DateField, DateLocale, TemporalContext, TemporalExpr};

fn dt_xsd(input: &str, ctx: &TemporalContext) -> String {
    match parse_temporal(input, ctx).unwrap() {
        TemporalExpr::DateTime(dt) => dt.to_xsd(),
        other => panic!("expected DateTime, got {other:?} for {input:?}"),
    }
}

#[test]
fn natural_comma_separated() {
    let ctx = TemporalContext::strict();
    assert_eq!(
        dt_xsd("June 1st 2026, 2:30 PM", &ctx),
        "2026-06-01T14:30:00"
    );
}

#[test]
fn natural_space_separated() {
    let ctx = TemporalContext::strict();
    assert_eq!(dt_xsd("June 1st 2026 14:30", &ctx), "2026-06-01T14:30:00");
}

#[test]
fn numeric_with_trailing_mask() {
    let ctx = TemporalContext::strict();
    // The `as` mask trails the whole literal but applies to the date part.
    assert_eq!(
        dt_xsd("01/06/2026 14:30 as d/m/y", &ctx),
        "2026-06-01T14:30:00"
    );
}

#[test]
fn numeric_with_locale() {
    let ctx = TemporalContext {
        locale: Some(DateLocale {
            order: [DateField::Day, DateField::Month, DateField::Year],
            separator: '/',
        }),
        timezone: None,
    };
    assert_eq!(dt_xsd("01/06/2026 14:30", &ctx), "2026-06-01T14:30:00");
}

#[test]
fn datetime_with_timezone() {
    let ctx = TemporalContext::strict();
    assert_eq!(
        dt_xsd("June 1st 2026, 2:30 PM UTC", &ctx),
        "2026-06-01T14:30:00+00:00"
    );
}
