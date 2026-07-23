use dolfin_datetime::{
    parse_temporal, DateLocale, DateField, TemporalContext, TemporalExpr, Timezone, UtcOffset,
};

fn time_xsd(input: &str) -> String {
    time_xsd_ctx(input, &TemporalContext::strict())
}

fn time_xsd_ctx(input: &str, ctx: &TemporalContext) -> String {
    match parse_temporal(input, ctx).unwrap() {
        TemporalExpr::Time(t) => t.to_xsd(),
        other => panic!("expected Time, got {other:?} for {input:?}"),
    }
}

#[test]
fn twenty_four_hour() {
    assert_eq!(time_xsd("14:30"), "14:30:00");
    assert_eq!(time_xsd("00:00"), "00:00:00");
    assert_eq!(time_xsd("23:59:59"), "23:59:59");
}

#[test]
fn twelve_hour() {
    assert_eq!(time_xsd("2:30 PM"), "14:30:00");
    assert_eq!(time_xsd("12:00 AM"), "00:00:00");
    assert_eq!(time_xsd("12:00 PM"), "12:00:00");
    assert_eq!(time_xsd("2:30 pm"), "14:30:00");
}

#[test]
fn inline_timezone() {
    assert_eq!(time_xsd("08:46:00 UTC"), "08:46:00+00:00");
    assert_eq!(time_xsd("14:30 +02:00"), "14:30:00+02:00");
    assert_eq!(time_xsd("14:30 -05:00"), "14:30:00-05:00");
    assert_eq!(time_xsd("14:30 Z"), "14:30:00+00:00");
    assert_eq!(time_xsd("14:30 CET"), "14:30:00+01:00");
}

#[test]
fn timezone_fallback_from_context() {
    let ctx = TemporalContext {
        locale: Some(DateLocale {
            order: [DateField::Day, DateField::Month, DateField::Year],
            separator: '/',
        }),
        timezone: Some(Timezone::Offset(UtcOffset { total_minutes: 120 })),
    };
    assert_eq!(time_xsd_ctx("14:30", &ctx), "14:30:00+02:00");
}

#[test]
fn inline_beats_context() {
    let ctx = TemporalContext {
        locale: None,
        timezone: Some(Timezone::Offset(UtcOffset { total_minutes: 120 })),
    };
    assert_eq!(time_xsd_ctx("14:30 -05:00", &ctx), "14:30:00-05:00");
}
