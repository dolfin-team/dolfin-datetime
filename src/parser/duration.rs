//! Duration-expression parsing with `nom` (§4.5). Components may appear in any
//! order; values are preserved as written (no normalisation, §4.5).

use nom::character::complete::{alpha1, digit1, space0};
use nom::combinator::{all_consuming, map_res};
use nom::multi::many1;
use nom::sequence::tuple;
use nom::IResult;

use crate::error::TemporalError;
use crate::types::Duration;

/// One `integer duration-unit` pair, e.g. `40min`.
fn component(input: &str) -> IResult<&str, (u32, String)> {
    let (input, _) = space0(input)?;
    let (input, (n, _sp, unit)) = tuple((
        map_res(digit1, |s: &str| s.parse::<u32>()),
        space0,
        alpha1,
    ))(input)?;
    Ok((input, (n, unit.to_string())))
}

/// Parse a bare duration expression (§4.5, §6.1). Public so the Dolfin compiler
/// can parse bare duration terms that appear in arithmetic position.
pub fn parse_duration(input: &str) -> Result<Duration, TemporalError> {
    match try_duration(input.trim()) {
        Some(res) => res,
        None => Err(TemporalError::ParseError {
            token: input.to_string(),
            reason: "not a duration".into(),
        }),
    }
}

/// `None` if the input does not consist of duration components at all.
pub(crate) fn try_duration(input: &str) -> Option<Result<Duration, TemporalError>> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let comps = match all_consuming(many1(component))(input) {
        Ok((_, c)) => c,
        Err(_) => return None,
    };

    let mut d = Duration::default();
    for (n, unit) in comps {
        let slot = match unit.to_ascii_lowercase().as_str() {
            "y" => &mut d.years,
            "mo" => &mut d.months,
            "w" => &mut d.weeks,
            "d" => &mut d.days,
            "h" => &mut d.hours,
            "min" => &mut d.minutes,
            "s" => &mut d.seconds,
            other => {
                return Some(Err(TemporalError::ParseError {
                    token: other.to_string(),
                    reason: "unknown duration unit (use y, mo, w, d, h, min, s)".into(),
                }));
            }
        };
        *slot += n;
    }
    Some(Ok(d))
}
