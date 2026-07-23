//! Time-expression parsing with `nom` (§4.3): 24h/12h clock, AM/PM, and inline
//! or file-level timezone resolution.

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, digit1, space0};
use nom::combinator::{all_consuming, map_res, opt, recognize};
use nom::sequence::preceded;
use nom::IResult;

use crate::context::TemporalContext;
use crate::error::TemporalError;
use crate::timezone::resolve_abbrev;
use crate::types::{Time, UtcOffset};

/// A timezone token captured by the parser, before context fallback.
enum TzTok {
    Offset(UtcOffset),
    Name(String),
}

struct ParsedTime {
    hour: i64,
    minute: i64,
    second: i64,
    is_pm: Option<bool>,
    tz: Option<TzTok>,
}

fn integer(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

/// `±HH:MM` inline offset.
fn tz_offset(input: &str) -> IResult<&str, TzTok> {
    let (input, sign) = alt((char('+'), char('-')))(input)?;
    let (input, hh) = map_res(recognize(digit1), |s: &str| s.parse::<i64>())(input)?;
    let (input, _) = char(':')(input)?;
    let (input, mm) = map_res(recognize(digit1), |s: &str| s.parse::<i64>())(input)?;
    let mins = hh * 60 + mm;
    let total = if sign == '-' { -mins } else { mins };
    Ok((
        input,
        TzTok::Offset(UtcOffset {
            total_minutes: total as i32,
        }),
    ))
}

/// A bare timezone abbreviation such as `UTC`, `Z`, `CET`.
fn tz_name(input: &str) -> IResult<&str, TzTok> {
    let (input, name) =
        nom::bytes::complete::take_while1(|c: char| c.is_ascii_alphabetic())(input)?;
    Ok((input, TzTok::Name(name.to_string())))
}

fn ampm(input: &str) -> IResult<&str, bool> {
    alt((
        nom::combinator::value(false, tag_no_case("am")),
        nom::combinator::value(true, tag_no_case("pm")),
    ))(input)
}

fn parse_time_expr(input: &str) -> IResult<&str, ParsedTime> {
    let (input, hour) = integer(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minute) = integer(input)?;
    let (input, second) = opt(preceded(tag(":"), integer))(input)?;
    let (input, is_pm) = opt(preceded(space0, ampm))(input)?;
    let (input, tz) = opt(preceded(space0, alt((tz_offset, tz_name))))(input)?;
    Ok((
        input,
        ParsedTime {
            hour,
            minute,
            second: second.unwrap_or(0),
            is_pm,
            tz,
        },
    ))
}

/// Try to parse the whole `input` as a time. `None` if it does not look like a
/// time at all; `Some(Err)` if it looks like a time but is malformed.
pub(crate) fn try_time(
    input: &str,
    ctx: &TemporalContext,
) -> Option<Result<Time, TemporalError>> {
    let input = input.trim();
    match all_consuming(parse_time_expr)(input) {
        Ok((_, p)) => Some(resolve(p, ctx)),
        // Only claim this as a (malformed) time if it started time-shaped:
        // a leading `HH:`. Otherwise let other productions try.
        Err(_) if looks_time_shaped(input) => Some(Err(TemporalError::InvalidTime {
            reason: format!("could not parse time '{input}'"),
        })),
        Err(_) => None,
    }
}

fn looks_time_shaped(input: &str) -> bool {
    // digit(s) then ':' near the start.
    let mut it = input.chars();
    let mut seen_digit = false;
    for c in it.by_ref() {
        if c.is_ascii_digit() {
            seen_digit = true;
        } else if c == ':' {
            return seen_digit;
        } else {
            return false;
        }
    }
    false
}

fn resolve(p: ParsedTime, ctx: &TemporalContext) -> Result<Time, TemporalError> {
    let hour = match p.is_pm {
        Some(pm) => {
            // 12-hour clock. Valid hours are 1–12.
            if !(1..=12).contains(&p.hour) {
                return Err(TemporalError::InvalidTime {
                    reason: format!("{} is not a valid 12-hour hour", p.hour),
                });
            }
            match (p.hour, pm) {
                (12, false) => 0,  // 12 AM = midnight
                (12, true) => 12,  // 12 PM = noon
                (h, false) => h,
                (h, true) => h + 12,
            }
        }
        None => {
            if !(0..=23).contains(&p.hour) {
                return Err(TemporalError::InvalidTime {
                    reason: format!("{} is not a valid 24-hour hour", p.hour),
                });
            }
            p.hour
        }
    };

    if !(0..=59).contains(&p.minute) {
        return Err(TemporalError::InvalidTime {
            reason: format!("minute {} out of range", p.minute),
        });
    }
    if !(0..=59).contains(&p.second) {
        return Err(TemporalError::InvalidTime {
            reason: format!("second {} out of range", p.second),
        });
    }

    // Timezone precedence: inline > file-level default > naive (§4.3).
    let offset = match p.tz {
        Some(TzTok::Offset(o)) => Some(o),
        Some(TzTok::Name(name)) => {
            Some(resolve_abbrev(&name).ok_or(TemporalError::UnknownTimezone(name))?)
        }
        None => match &ctx.timezone {
            Some(tz) => Some(tz.resolve()?),
            None => None,
        },
    };

    Ok(Time {
        hour: hour as u8,
        minute: p.minute as u8,
        second: p.second as u8,
        offset,
    })
}
