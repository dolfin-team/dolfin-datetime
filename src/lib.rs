//! `dolfin-datetime` — date, time, datetime and duration parsing for the Dolfin
//! DSL.
//!
//! The Dolfin parser identifies a parenthesised smart literal `(...)` as
//! temporal and hands its raw content to [`parse_temporal`], which returns a
//! typed [`TemporalExpr`] ready for Turtle serialisation via `to_xsd()`.
//!
//! ```
//! use dolfin_datetime::{parse_temporal, TemporalContext, TemporalExpr};
//!
//! let ctx = TemporalContext::strict();
//! let expr = parse_temporal("June 1st 2026", &ctx).unwrap();
//! assert_eq!(expr.to_xsd(), "2026-06-01");
//! assert!(matches!(expr, TemporalExpr::Date(_)));
//! ```
//!
//! Compile-time arithmetic (§6 of the spec) lives in [`apply`] and
//! [`apply_chain`]; the Dolfin compiler orchestrates chains of smart literals
//! and bare duration terms ([`parse_duration`]).

mod arithmetic;
mod context;
mod error;
mod parser;
mod serial;
mod timezone;
mod types;

pub use arithmetic::{apply, apply_chain, Evaluated, Op};
pub use context::{parse_locale_order, DateField, DateLocale, TemporalContext, Timezone};
pub use error::TemporalError;
pub use parser::duration::parse_duration;
pub use parser::parse_temporal;
pub use timezone::{resolve_abbrev, resolve_named};
pub use types::{Date, DateTime, Duration, Time, TemporalExpr, UtcOffset};
