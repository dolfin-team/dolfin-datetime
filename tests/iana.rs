#![cfg(feature = "iana-tz")]

use dolfin_datetime::{parse_temporal, resolve_named, TemporalContext, TemporalExpr};

#[test]
fn resolve_iana_name() {
    // Brussels is CET (+01:00) at the epoch representative instant.
    let off = resolve_named("Europe/Brussels").unwrap();
    assert_eq!(off.total_minutes, 60);
}

#[test]
fn context_applies_iana_default() {
    let ctx = TemporalContext::with_locale_and_tz("d/m/y", '/', "Europe/Brussels").unwrap();
    match parse_temporal("14:30", &ctx).unwrap() {
        TemporalExpr::Time(t) => assert_eq!(t.to_xsd(), "14:30:00+01:00"),
        other => panic!("expected Time, got {other:?}"),
    }
}

#[test]
fn unknown_iana_name_errors() {
    assert!(resolve_named("Mars/Olympus").is_err());
}
