use dolfin_datetime::{
    parse_temporal, DateField, DateLocale, TemporalContext, TemporalError, TemporalExpr,
};

fn date_xsd(input: &str, ctx: &TemporalContext) -> String {
    match parse_temporal(input, ctx).unwrap() {
        TemporalExpr::Date(d) => d.to_xsd(),
        other => panic!("expected Date, got {other:?}"),
    }
}

fn locale(order: [DateField; 3], sep: char) -> TemporalContext {
    TemporalContext {
        locale: Some(DateLocale { order, separator: sep }),
        timezone: None,
    }
}

#[test]
fn mask_dmy() {
    let ctx = TemporalContext::strict();
    assert_eq!(date_xsd("01/06/2026 as d/m/y", &ctx), "2026-06-01");
}

#[test]
fn mask_ymd() {
    let ctx = TemporalContext::strict();
    assert_eq!(date_xsd("2001-09-11 as y-m-d", &ctx), "2001-09-11");
}

#[test]
fn mask_dmy_dots() {
    let ctx = TemporalContext::strict();
    assert_eq!(date_xsd("11.09.2001 as d.m.y", &ctx), "2001-09-11");
}

#[test]
fn locale_fallback_resolves() {
    let ctx = locale([DateField::Day, DateField::Month, DateField::Year], '/');
    assert_eq!(date_xsd("01/06/2026", &ctx), "2026-06-01");
}

#[test]
fn locale_mdy() {
    let ctx = locale([DateField::Month, DateField::Day, DateField::Year], '/');
    assert_eq!(date_xsd("06/01/2026", &ctx), "2026-06-01");
}

#[test]
fn no_locale_no_mask_is_ambiguous() {
    let ctx = TemporalContext::strict();
    match parse_temporal("01/06/2026", &ctx) {
        Err(TemporalError::AmbiguousDate(_)) => {}
        other => panic!("expected AmbiguousDate, got {other:?}"),
    }
}

#[test]
fn separator_mismatch_mask() {
    let ctx = TemporalContext::strict();
    match parse_temporal("01/06/2026 as d-m-y", &ctx) {
        Err(TemporalError::SeparatorMismatch {
            value_sep: '/',
            mask_sep: '-',
        }) => {}
        other => panic!("expected SeparatorMismatch, got {other:?}"),
    }
}

#[test]
fn inline_mask_beats_locale() {
    // Locale says m/d/y, but the inline mask says d/m/y and wins.
    let ctx = locale([DateField::Month, DateField::Day, DateField::Year], '/');
    assert_eq!(date_xsd("01/06/2026 as d/m/y", &ctx), "2026-06-01");
}
