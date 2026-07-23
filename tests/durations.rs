use dolfin_datetime::{parse_duration, parse_temporal, TemporalContext, TemporalExpr};

fn dur_xsd(input: &str) -> String {
    let ctx = TemporalContext::strict();
    match parse_temporal(input, &ctx).unwrap() {
        TemporalExpr::Duration(d) => d.to_xsd(),
        other => panic!("expected Duration, got {other:?} for {input:?}"),
    }
}

#[test]
fn single_units() {
    assert_eq!(dur_xsd("7d"), "P7D");
    assert_eq!(dur_xsd("1y"), "P1Y");
    assert_eq!(dur_xsd("6mo"), "P6M");
    assert_eq!(dur_xsd("2w"), "P2W");
    assert_eq!(dur_xsd("3h"), "PT3H");
    assert_eq!(dur_xsd("40min"), "PT40M");
    assert_eq!(dur_xsd("15s"), "PT15S");
}

#[test]
fn combined() {
    assert_eq!(dur_xsd("1h 40min"), "PT1H40M");
    assert_eq!(dur_xsd("1y 6mo"), "P1Y6M");
    assert_eq!(dur_xsd("7d 1h 40min"), "P7DT1H40M");
    assert_eq!(dur_xsd("2h 30min 15s"), "PT2H30M15S");
}

#[test]
fn any_order_serialises_iso() {
    // Source order is free; XSD output follows ISO 8601 order.
    assert_eq!(dur_xsd("40min 1h"), "PT1H40M");
    assert_eq!(dur_xsd("6mo 1y"), "P1Y6M");
    assert_eq!(dur_xsd("1h 7d"), "P7DT1H");
}

#[test]
fn weeks_preserved_not_converted() {
    // Q3: xsd:duration keeps nW rather than folding to days.
    assert_eq!(dur_xsd("1w"), "P1W");
}

#[test]
fn bare_duration_parse() {
    let d = parse_duration("7d 1h 40min").unwrap();
    assert_eq!(d.to_xsd(), "P7DT1H40M");
}
