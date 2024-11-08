use super::{DateTimeParseError, TimeStampOffset};
use crate::parser::Span;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::one_of,
    combinator::{map_res, opt},
    sequence::preceded,
    IResult,
};
use std::{fmt::Display, str::FromStr};

/// The results of parsing a timestamp. Note that the timestamp is not validated,
/// i.e. it may not be a valid date or time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Time {
    /// The hour of the time (0-23)
    pub hour: u8,
    /// The minute of the time (0-59)
    pub minute: Option<u8>,
    /// The second of the time (0-59)
    pub second: Option<u8>,
    /// The microsecond of the time (0-999_900)
    pub microsecond: Option<u32>,
    /// The timezone offset of the time
    pub offset: Option<TimeStampOffset>,
}

/// Parse an HL7 time in the format: `HH[MM[SS[.S[S[S[S]]]]]][+/-ZZZZ]`
///
/// # Arguments
/// * `s` - The string to parse
/// * `lenient_trailing_chars` - If true, allow trailing characters after the timestamp, otherwise
///   throw an error
///
/// # Example
///
/// ```
/// use hl7_parser::datetime::{parse_time, Time, TimeStampOffset};
///
/// let time: Time = parse_time("195905.1234-0700", false).expect("can parse time");
///
/// assert_eq!(time.hour, 19);
/// assert_eq!(time.minute, Some(59));
/// assert_eq!(time.second, Some(5));
/// assert_eq!(time.microsecond, Some(123_400));
/// assert_eq!(time.offset, Some(TimeStampOffset {
///     hours: -7,
///     minutes: 0,
/// }));
/// ```
pub fn parse_time<'s>(
    s: &'s str,
    lenient_trailing_chars: bool,
) -> Result<Time, DateTimeParseError> {
    fn is_decimal_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn from_digits<F: FromStr>(i: Span) -> Result<F, F::Err> {
        i.input.parse::<F>()
    }

    fn digit2<F: FromStr>(input: Span) -> IResult<Span, F> {
        map_res(take_while_m_n(2, 2, is_decimal_digit), from_digits::<F>)(input)
    }

    let s = Span::new(s);
    let (s, hour): (Span, u8) = digit2(s).map_err(|_| DateTimeParseError::ParsingFailed("hour"))?;
    let (s, minute): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("minute"))?;
    let (s, second): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("second"))?;
    let (s, second_fracs) = opt(preceded(tag("."), take_while_m_n(1, 4, is_decimal_digit)))(s)
        .map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            DateTimeParseError::ParsingFailed("fractional seconds")
        })?;
    let (s, offset_dir) =
        opt(one_of("+-"))(s).map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            DateTimeParseError::ParsingFailed("offset direction")
        })?;

    let offset_dir = match offset_dir.unwrap_or('+') {
        '-' => -1i8,
        _ => 1i8,
    };
    let (s, offset_hours): (Span, Option<i8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("offset hours"))?;
    let offset_hours = offset_hours.map(|h| h * offset_dir);
    let (s, offset_minutes): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("offset minutes"))?;

    if !lenient_trailing_chars && !s.is_empty() {
        return Err(DateTimeParseError::UnexpectedCharacter(
            s.offset,
            s.input.chars().next().unwrap_or_default(),
        ));
    }

    let microsecond = match second_fracs {
        Some(fracs) => {
            let fracs_multiplier = match fracs.len() {
                1 => 100_000,
                2 => 10_000,
                3 => 1_000,
                4 => 100,
                _ => panic!("second_fracs.len() not in 1..=4"),
            };
            Some(
                fracs
                    .input
                    .parse::<u32>()
                    .expect("can parse fractional seconds as number")
                    * fracs_multiplier,
            )
        }
        None => None,
    };

    let offset = match (offset_hours, offset_minutes) {
        (Some(hours), Some(minutes)) => Some(TimeStampOffset { hours, minutes }),
        _ => None,
    };

    Ok(Time {
        hour,
        minute,
        second,
        microsecond,
        offset,
    })
}

/// Implement `FromStr` for `Time` to allow parsing timestamps from strings
impl FromStr for Time {
    type Err = DateTimeParseError;

    /// Synonymous with `parse_time`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_time(s, false)
    }
}

/// Implement `Display` for `Time` to allow formatting timestamps as HL7 strings
impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", self.hour)?;
        if let Some(minute) = self.minute {
            write!(f, "{:02}", minute)?;
            if let Some(second) = self.second {
                write!(f, "{:02}", second)?;
                if let Some(microsecond) = self.microsecond {
                    let microsecond = format!("{:06}", microsecond);
                    write!(f, ".{}", &microsecond[..4])?;
                }
            }
        }
        if let Some(offset) = &self.offset {
            write!(f, "{}", offset)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_time_with_offsets() {
        let ts = "195905.1234-0700";
        let ts = parse_time(ts, false).expect("can parse time");

        assert_eq!(ts.hour, 19);
        assert_eq!(ts.minute, Some(59));
        assert_eq!(ts.second, Some(5));
        assert_eq!(ts.microsecond, Some(123_400));
        assert_eq!(
            ts.offset,
            Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            })
        );
    }
}
