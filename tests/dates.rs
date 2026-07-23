use dolfin_datetime::{parse_temporal, TemporalContext, TemporalExpr};

fn date_xsd(input: &str) -> String {
    let ctx = TemporalContext::strict();
    match parse_temporal(input, &ctx).unwrap() {
        TemporalExpr::Date(d) => d.to_xsd(),
        other => panic!("expected Date, got {other:?} for {input:?}"),
    }
}

#[test]
fn natural_month_first() {
    assert_eq!(date_xsd("June 1st 2026"), "2026-06-01");
}

#[test]
fn natural_day_first() {
    assert_eq!(date_xsd("1st June 2026"), "2026-06-01");
}

#[test]
fn natural_abbrev_no_ordinal() {
    assert_eq!(date_xsd("Jun 1 2026"), "2026-06-01");
}

#[test]
fn natural_case_insensitive() {
    assert_eq!(date_xsd("JUNE 1ST 2026"), "2026-06-01");
}

#[test]
fn all_months() {
    let cases = [
        ("January 5 2020", "2020-01-05"),
        ("Feb 5 2020", "2020-02-05"),
        ("March 5 2020", "2020-03-05"),
        ("Apr 5 2020", "2020-04-05"),
        ("May 5 2020", "2020-05-05"),
        ("Jun 5 2020", "2020-06-05"),
        ("Jul 5 2020", "2020-07-05"),
        ("August 5 2020", "2020-08-05"),
        ("Sept 5 2020", "2020-09-05"),
        ("Sep 5 2020", "2020-09-05"),
        ("Oct 5 2020", "2020-10-05"),
        ("Nov 5 2020", "2020-11-05"),
        ("December 5 2020", "2020-12-05"),
    ];
    for (input, expected) in cases {
        assert_eq!(date_xsd(input), expected, "for {input}");
    }
}

#[test]
fn ordinal_suffixes_stripped() {
    for (input, expected) in [
        ("Jan 1st 2020", "2020-01-01"),
        ("Jan 2nd 2020", "2020-01-02"),
        ("Jan 3rd 2020", "2020-01-03"),
        ("Jan 4th 2020", "2020-01-04"),
        ("Jan 11th 2020", "2020-01-11"),
        ("Jan 21st 2020", "2020-01-21"),
    ] {
        assert_eq!(date_xsd(input), expected, "for {input}");
    }
}

#[test]
fn leap_day_valid() {
    assert_eq!(date_xsd("Feb 29 2024"), "2024-02-29");
}

#[test]
fn leap_day_invalid() {
    let ctx = TemporalContext::strict();
    assert!(parse_temporal("Feb 29 2026", &ctx).is_err());
}
