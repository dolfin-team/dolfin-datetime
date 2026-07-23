//! Natural-language and numeric date parsing (§4.2), plus the `as`-mask and
//! locale field mapping (§4.2, §7.1).
//!
//! Field mapping is done in plain Rust rather than `nom`: the interesting work
//! here is diagnostic (separator-mismatch, ambiguous-without-locale,
//! unknown-month), and hand-written control flow yields precise
//! `TemporalError`s that a combinator's opaque error would bury.

use crate::context::{DateField, TemporalContext};
use crate::error::TemporalError;
use crate::types::Date;

const DATE_SEPS: [char; 3] = ['/', '-', '.'];

/// Days in a given (proleptic Gregorian) year/month. `month` must be 1–12.
pub(crate) fn days_in_month(year: i32, month: u8) -> u8 {
    match time::Month::try_from(month) {
        Ok(m) => m.length(year),
        Err(_) => 0,
    }
}

/// Validate (year, month, day) as a real calendar date and build a `Date`.
pub(crate) fn build_date(year: i32, month: u8, day: u8) -> Result<Date, TemporalError> {
    let m = time::Month::try_from(month).map_err(|_| TemporalError::InvalidDate {
        reason: format!("month {month} out of range 1–12"),
    })?;
    time::Date::from_calendar_date(year, m, day).map_err(|_| TemporalError::InvalidDate {
        reason: format!("{year:04}-{month:02}-{day:02} is not a valid date"),
    })?;
    Ok(Date { year, month, day })
}

/// Try to parse the whole `input` as a date. Returns:
/// - `Some(Ok(date))`  — parsed successfully
/// - `Some(Err(e))`    — clearly a date attempt but malformed / ambiguous
/// - `None`            — does not look like a date at all (try other productions)
pub(crate) fn try_date(
    input: &str,
    mask: Option<&str>,
    ctx: &TemporalContext,
) -> Option<Result<Date, TemporalError>> {
    let input = input.trim();
    if let Some(res) = parse_natural(input) {
        return Some(res);
    }
    if has_date_sep(input) {
        return Some(parse_numeric(input, mask, ctx));
    }
    None
}

fn has_date_sep(s: &str) -> bool {
    DATE_SEPS.iter().any(|c| s.contains(*c))
}

fn is_alpha(tok: &str) -> bool {
    !tok.is_empty() && tok.chars().all(|c| c.is_ascii_alphabetic())
}

/// Strip an ordinal suffix (`st`/`nd`/`rd`/`th`) and parse the leading integer.
fn parse_day_ordinal(tok: &str) -> Option<u8> {
    let lower = tok.to_ascii_lowercase();
    let digits: String = lower.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &lower[digits.len()..];
    if !matches!(rest, "" | "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse::<u8>().ok()
}

fn parse_year(tok: &str) -> Option<i32> {
    if !tok.is_empty() && tok.chars().all(|c| c.is_ascii_digit()) {
        tok.parse::<i32>().ok()
    } else {
        None
    }
}

/// `January` … `Dec`, case-insensitive (§4.2). Returns the month number 1–12.
fn lookup_month(word: &str) -> Option<u8> {
    let w = word.to_ascii_lowercase();
    let n = match w.as_str() {
        "january" | "jan" => 1,
        "february" | "feb" => 2,
        "march" | "mar" => 3,
        "april" | "apr" => 4,
        "may" => 5,
        "june" | "jun" => 6,
        "july" | "jul" => 7,
        "august" | "aug" => 8,
        "september" | "sep" | "sept" => 9,
        "october" | "oct" => 10,
        "november" | "nov" => 11,
        "december" | "dec" => 12,
        _ => return None,
    };
    Some(n)
}

/// `month day-ordinal year` or `day-ordinal month year` (§4.2).
fn parse_natural(input: &str) -> Option<Result<Date, TemporalError>> {
    let toks: Vec<&str> = input.split_whitespace().collect();
    if toks.len() != 3 {
        return None;
    }
    // The month is the alphabetic token; its position selects the form.
    let (mon_tok, day_tok, year_tok) = if is_alpha(toks[0]) {
        (toks[0], toks[1], toks[2])
    } else if is_alpha(toks[1]) {
        (toks[1], toks[0], toks[2])
    } else {
        return None;
    };

    // Only commit to a natural-date reading if day and year are well-formed;
    // otherwise this isn't a date and other productions should get a chance.
    let day = parse_day_ordinal(day_tok)?;
    let year = parse_year(year_tok)?;

    match lookup_month(mon_tok) {
        Some(month) => Some(build_date(year, month, day)),
        None => Some(Err(TemporalError::UnknownMonth(mon_tok.to_string()))),
    }
}

/// `integer sep integer sep integer` with mask or locale field order (§4.2).
fn parse_numeric(
    input: &str,
    mask: Option<&str>,
    ctx: &TemporalContext,
) -> Result<Date, TemporalError> {
    let value_sep = DATE_SEPS
        .iter()
        .copied()
        .find(|c| input.contains(*c))
        .ok_or_else(|| TemporalError::ParseError {
            token: input.to_string(),
            reason: "numeric date needs a / - or . separator".into(),
        })?;

    let parts: Vec<&str> = input.split(value_sep).map(str::trim).collect();
    if parts.len() != 3 || parts.iter().any(|p| !p.chars().all(|c| c.is_ascii_digit()) || p.is_empty())
    {
        return Err(TemporalError::ParseError {
            token: input.to_string(),
            reason: "numeric date must be three integers".into(),
        });
    }

    let order = resolve_order(input, value_sep, mask, ctx)?;
    map_fields(&parts, &order, input)
}

/// Determine the [DateField; 3] order for a numeric value, honouring the
/// mask-isomorphism rule (§4.2) and the locale fallback (§4.2, §7.1).
fn resolve_order(
    input: &str,
    value_sep: char,
    mask: Option<&str>,
    ctx: &TemporalContext,
) -> Result<[DateField; 3], TemporalError> {
    if let Some(mask) = mask {
        let (mask_order, mask_sep) = parse_mask(mask)?;
        if mask_sep != value_sep {
            return Err(TemporalError::SeparatorMismatch {
                value_sep,
                mask_sep,
            });
        }
        return Ok(mask_order);
    }

    if let Some(locale) = ctx.locale {
        // Q1: enforce the locale separator — consistency over leniency.
        if locale.separator != value_sep {
            return Err(TemporalError::SeparatorMismatch {
                value_sep,
                mask_sep: locale.separator,
            });
        }
        return Ok(locale.order);
    }

    Err(TemporalError::AmbiguousDate(input.to_string()))
}

/// Parse an `as` mask like `d/m/y` into (field order, separator).
fn parse_mask(mask: &str) -> Result<([DateField; 3], char), TemporalError> {
    let mask = mask.trim();
    let sep = DATE_SEPS
        .iter()
        .copied()
        .find(|c| mask.contains(*c))
        .ok_or_else(|| TemporalError::ParseError {
            token: mask.to_string(),
            reason: "mask needs a / - or . separator".into(),
        })?;
    let fields: Vec<DateField> = mask
        .split(sep)
        .map(str::trim)
        .map(|f| match f.to_ascii_lowercase().as_str() {
            "y" => Ok(DateField::Year),
            "m" => Ok(DateField::Month),
            "d" => Ok(DateField::Day),
            other => Err(TemporalError::ParseError {
                token: other.to_string(),
                reason: "mask field must be y, m or d".into(),
            }),
        })
        .collect::<Result<_, _>>()?;
    if fields.len() != 3 {
        return Err(TemporalError::ParseError {
            token: mask.to_string(),
            reason: "mask must have three fields".into(),
        });
    }
    Ok(([fields[0], fields[1], fields[2]], sep))
}

fn map_fields(parts: &[&str], order: &[DateField; 3], input: &str) -> Result<Date, TemporalError> {
    let mut year: Option<i32> = None;
    let mut month: Option<u8> = None;
    let mut day: Option<u8> = None;
    for (field, raw) in order.iter().zip(parts.iter()) {
        match field {
            DateField::Year => year = raw.parse().ok(),
            DateField::Month => month = raw.parse().ok(),
            DateField::Day => day = raw.parse().ok(),
        }
    }
    match (year, month, day) {
        (Some(y), Some(m), Some(d)) => build_date(y, m, d),
        _ => Err(TemporalError::ParseError {
            token: input.to_string(),
            reason: "numeric date fields out of range".into(),
        }),
    }
}
