# dolfin-datetime

[![crates.io](https://img.shields.io/crates/v/dolfin-datetime.svg)](https://crates.io/crates/dolfin-datetime)
[![docs.rs](https://docs.rs/dolfin-datetime/badge.svg)](https://docs.rs/dolfin-datetime)
[![license](https://img.shields.io/crates/l/dolfin-datetime.svg)](https://github.com/dolfin-team/dolfin-datetime)

Date, time, datetime and duration parsing for the [Dolfin Langugage](https://github.com/dolfin-team).

The Dolfin parser identifies a parenthesised smart literal `(...)` as temporal and hands its raw content to `parse_temporal`, which returns a typed `TemporalExpr` ready for Turtle serialisation via `to_xsd()`. The XSD type (`xsd:date`, `xsd:time`, `xsd:dateTime`, `xsd:duration`) is inferred from the literal content — the author never writes it explicitly.

## Quick start

```rust
use dolfin_datetime::{parse_temporal, TemporalContext, TemporalExpr};

let ctx = TemporalContext::strict();
let expr = parse_temporal("June 1st 2026", &ctx).unwrap();
assert_eq!(expr.to_xsd(), "2026-06-01");
assert_eq!(expr.xsd_type(), "xsd:date");
assert!(matches!(expr, TemporalExpr::Date(_)));
```

## Supported literal forms

| Kind | Examples | XSD output |
|---|---|---|
| Natural date | `June 1st 2026`, `1st June 2026`, `Dec 25 2030` | `2026-06-01` |
| Numeric date | `01/06/2026 as d/m/y`, `2026-06-01 as y-m-d` | `2026-06-01` |
| Time | `14:30`, `2:30 pm`, `14:30:15 UTC`, `09:00 +02:00` | `14:30:00+02:00` |
| Datetime | `June 1st 2026 14:30`, `01/06/2026, 2:30 pm CET as d/m/y` | `2026-06-01T14:30:00+01:00` |
| Duration | `3d`, `1y 6mo`, `2w 3d 4h 30min` | `P1Y6M`, `P2W3DT4H30M` |

Duration units: `y`, `mo`, `w`, `d`, `h`, `min`, `s` — any order, values preserved as written (no normalisation).

### Disambiguating numeric dates

Ambiguous numeric dates require either an inline `as` mask (`01/06/2026 as d/m/y`) or a file-level locale default:

```rust
use dolfin_datetime::{parse_temporal, TemporalContext};

// Locale d/m/y with '/' separator, default timezone Europe/Brussels.
let ctx = TemporalContext::with_locale_and_tz("d/m/y", '/', "Europe/Brussels").unwrap();
let expr = parse_temporal("01/06/2026 14:30", &ctx).unwrap();
assert_eq!(expr.to_xsd(), "2026-06-01T14:30:00+01:00");
```

With `TemporalContext::strict()` (no locale, no timezone), an ambiguous numeric date without a mask is a parse error, and times without an inline offset stay timezone-naive.

### Timezones

Inline timezone beats the file-level default. Accepted forms:

- Fixed offsets: `+02:00`, `-05:00`
- Unambiguous abbreviations: `UTC`, `Z`, `GMT`, `CET`, `CEST`, `EST`, `EDT`, `CST`, `CDT`, `MST`, `MDT`, `PST`, `PDT`
- IANA names such as `Europe/Brussels` (with the default `iana-tz` feature)

Ambiguous abbreviations (e.g. `IST`) are deliberately rejected — use a full IANA name or an explicit offset.

## Compile-time arithmetic

`apply` and `apply_chain` evaluate `+`/`-` chains over temporal values at parse time, with type rules, month clamping and calendar-style differences:

```rust
use dolfin_datetime::{apply, parse_duration, parse_temporal, Op, TemporalContext, TemporalExpr};

let ctx = TemporalContext::strict();
let start = parse_temporal("January 31st 2026", &ctx).unwrap();
let month = TemporalExpr::Duration(parse_duration("1mo").unwrap());

let result = apply(&start, Op::Add, &month).unwrap();
assert_eq!(result.expr.to_xsd(), "2026-02-28"); // day clamped
assert!(!result.warnings.is_empty());            // MonthOverflow warning
```

Supported combinations: `Date ± Duration`, `DateTime ± Duration`, `Time ± Duration` (wraps at 24 h), `Duration ± Duration`, `Date - Date` and `DateTime - DateTime` (always a positive calendar-style duration). Anything else is a `TypeMismatch` error. Month clamping (`Jan 31 + 1mo → Feb 28`) is a non-fatal warning, never a hard error.

## Feature flags

| Feature | Default | Effect |
|---|---|---|
| `iana-tz` | yes | Resolve IANA timezone names (`Europe/Brussels`) via [`tzdb`](https://crates.io/crates/tzdb) |
| `serde` | no | `Serialize`/`Deserialize` on all public types |

Without `iana-tz`, only fixed offsets and the built-in abbreviation table are accepted.

## Error handling

All failures are a single `TemporalError` enum with precise diagnostics: `AmbiguousDate`, `UnknownMonth`, `InvalidDate`, `InvalidTime`, `UnknownTimezone`, `SeparatorMismatch`, `TypeMismatch`, `MonthOverflow` (warning only) and `ParseError`.

## License

MIT
