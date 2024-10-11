use crate::parser::Span;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::one_of,
    combinator::{map_res, opt},
    sequence::preceded,
    IResult,
};
use std::{fmt::Display, str::FromStr};

/// Utilities to convert back and forth between `TimeStamp`s and `chrono`'s `DateTime` and
/// `NaiveDateTime`
#[cfg(feature = "chrono")]
pub mod chrono;

/// Utilities to convert back and forth between `TimeStamp`s and `time`'s `OffsetDateTime`
/// and `PrimitiveDateTime`
#[cfg(feature = "time")]
pub mod time;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeComponent {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Microsecond,
    Offset,
    Date,
    Time,
    DateTime,
}

impl Display for TimeComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeComponent::Year => write!(f, "year"),
            TimeComponent::Month => write!(f, "month"),
            TimeComponent::Day => write!(f, "day"),
            TimeComponent::Hour => write!(f, "hour"),
            TimeComponent::Minute => write!(f, "minute"),
            TimeComponent::Second => write!(f, "second"),
            TimeComponent::Microsecond => write!(f, "microsecond"),
            TimeComponent::Offset => write!(f, "offset"),
            TimeComponent::Date => write!(f, "date"),
            TimeComponent::Time => write!(f, "time"),
            TimeComponent::DateTime => write!(f, "date and time"),
        }
    }
}

/// Errors that can result from parsing HL7 timestamps
#[derive(thiserror::Error, Debug)]
pub enum TimeParseError {
    #[error("Failed to parse '{0}' component of timestamp")]
    ParsingFailed(&'static str),
    #[error("Unexpected character '{1}' in timestamp at position {0}")]
    UnexpectedCharacter(usize, char),
    #[error("Invalid component range: {0:}")]
    InvalidComponentRange(TimeComponent),
    #[error("Ambiguous time, could be {0} or {1}")]
    AmbiguousTime(String, String),
    #[error("Missing component: {0:}")]
    MissingComponent(TimeComponent),
}

/// A parsed timezone offset in hours and minutes
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeStampOffset {
    /// The hours offset from UTC. Note: if this value is negative, the timezone
    /// is behind UTC, if positive, it is ahead of UTC.
    pub hours: i8,
    /// The minutes offset from UTC
    pub minutes: u8,
}

impl Display for TimeStampOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:+03}{:02}", self.hours, self.minutes)
    }
}

/// The results of parsing a timestamp. Note that the timestamp is not validated,
/// i.e. it may not be a valid date or time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeStamp {
    /// The year of the timestamp
    pub year: u16,
    /// The month of the timestamp (1-12)
    pub month: Option<u8>,
    /// The day of the timestamp (1-31)
    pub day: Option<u8>,
    /// The hour of the timestamp (0-23)
    pub hour: Option<u8>,
    /// The minute of the timestamp (0-59)
    pub minute: Option<u8>,
    /// The second of the timestamp (0-59)
    pub second: Option<u8>,
    /// The microsecond of the timestamp (0-999_900)
    pub microsecond: Option<u32>,
    /// The timezone offset of the timestamp
    pub offset: Option<TimeStampOffset>,
}

/// Parse an HL7 timestamp in the format: `YYYY[MM[DD[HH[MM[SS[.S[S[S[S]]]]]]]]][+/-ZZZZ]`
///
/// # Example
///
/// ```
/// use hl7_parser::timestamps::{parse_timestamp, TimeStamp, TimeStampOffset};
///
/// let ts: TimeStamp = parse_timestamp("20230312195905.1234-0700").expect("can parse timestamp");
///
/// assert_eq!(ts.year, 2023);
/// assert_eq!(ts.month, Some(3));
/// assert_eq!(ts.day, Some(12));
/// assert_eq!(ts.hour, Some(19));
/// assert_eq!(ts.minute, Some(59));
/// assert_eq!(ts.second, Some(5));
/// assert_eq!(ts.microsecond, Some(123_400));
/// assert_eq!(ts.offset, Some(TimeStampOffset {
///     hours: -7,
///     minutes: 0,
/// }));
/// ```
pub fn parse_timestamp<'s>(s: &'s str) -> Result<TimeStamp, TimeParseError> {
    fn is_decimal_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn from_digits<F: FromStr>(i: Span) -> Result<F, F::Err> {
        i.input.parse::<F>()
    }

    fn digit2<F: FromStr>(input: Span) -> IResult<Span, F> {
        map_res(take_while_m_n(2, 2, is_decimal_digit), from_digits::<F>)(input)
    }

    fn digit4<F: FromStr>(input: Span) -> IResult<Span, F> {
        map_res(take_while_m_n(4, 4, is_decimal_digit), from_digits::<F>)(input)
    }

    let s = Span::new(s);
    let (s, year): (Span, u16) = digit4(s).map_err(|_| TimeParseError::ParsingFailed("year"))?;
    let (s, month): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("month"))?;
    let (s, day): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("day"))?;
    let (s, hour): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("hour"))?;
    let (s, minute): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("minute"))?;
    let (s, second): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("second"))?;
    let (s, second_fracs) = opt(preceded(tag("."), take_while_m_n(1, 4, is_decimal_digit)))(s)
        .map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            TimeParseError::ParsingFailed("fractional seconds")
        })?;
    let (s, offset_dir) =
        opt(one_of("+-"))(s).map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            TimeParseError::ParsingFailed("offset direction")
        })?;

    let offset_dir = match offset_dir.unwrap_or('+') {
        '-' => -1i8,
        _ => 1i8,
    };
    let (s, offset_hours): (Span, Option<i8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("offset hours"))?;
    let offset_hours = offset_hours.map(|h| h * offset_dir);
    let (s, offset_minutes): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("offset minutes"))?;

    if !s.is_empty() {
        return Err(TimeParseError::UnexpectedCharacter(
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

    Ok(TimeStamp {
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        offset,
    })
}

/// Implement `FromStr` for `TimeStamp` to allow parsing timestamps from strings
impl FromStr for TimeStamp {
    type Err = TimeParseError;

    /// Synonymous with `parse_timestamp`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_timestamp(s)
    }
}

/// Implement `Display` for `TimeStamp` to allow formatting timestamps as HL7 strings
impl Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.year)?;
        if let Some(month) = self.month {
            write!(f, "{:02}", month)?;
            if let Some(day) = self.day {
                write!(f, "{:02}", day)?;
                if let Some(hour) = self.hour {
                    write!(f, "{:02}", hour)?;
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
        let ts = "20230312195905.1234-0700";
        let ts = parse_timestamp(ts).expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
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

    #[test]
    fn can_parse_time_without_offsets() {
        let ts = "20230312195905.1234";
        let ts = parse_timestamp(ts).expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
        assert_eq!(ts.minute, Some(59));
        assert_eq!(ts.second, Some(5));
        assert_eq!(ts.microsecond, Some(123_400));
        assert_eq!(ts.offset, None);
    }

    #[test]
    fn can_parse_time_without_offsets_or_fractional_seconds() {
        let ts = "20230312195905";
        let ts = parse_timestamp(ts).expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
        assert_eq!(ts.minute, Some(59));
        assert_eq!(ts.second, Some(5));
        assert_eq!(ts.microsecond, None);
        assert_eq!(ts.offset, None);
    }

    #[test]
    fn can_parse_time_with_offsets_without_fractional_seconds() {
        let ts = "20230312195905-0700";
        let ts = parse_timestamp(ts).expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
        assert_eq!(ts.minute, Some(59));
        assert_eq!(ts.second, Some(5));
        assert_eq!(ts.microsecond, None);
        assert_eq!(
            ts.offset,
            Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            })
        );
    }

    #[test]
    fn can_parse_time_with_only_year() {
        let ts = "2023";
        let ts = parse_timestamp(ts).expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, None);
        assert_eq!(ts.day, None);
        assert_eq!(ts.hour, None);
        assert_eq!(ts.minute, None);
        assert_eq!(ts.second, None);
        assert_eq!(ts.microsecond, None);
        assert_eq!(ts.offset, None);
    }

    #[test]
    fn cant_parse_bad_timestamps() {
        assert!(parse_timestamp("23").is_err());
        assert!(parse_timestamp("abcd").is_err());
        assert!(parse_timestamp("202303121959051").is_err());
    }

    #[test]
    fn can_parse_timestamp_fromstr() {
        let ts: TimeStamp = "20230312195905.1234-0700"
            .parse()
            .expect("can parse timestamp");

        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
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

    #[test]
    fn can_format_timestamp() {
        let ts = TimeStamp {
            year: 2023,
            month: Some(3),
            day: Some(12),
            hour: Some(19),
            minute: Some(59),
            second: Some(5),
            microsecond: Some(123_400),
            offset: Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            }),
        };
        assert_eq!(ts.to_string(), "20230312195905.1234-0700");

        let ts = TimeStamp {
            year: 2023,
            month: Some(3),
            day: Some(12),
            hour: Some(19),
            minute: None,
            second: None,
            microsecond: None,
            offset: Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            }),
        };
        assert_eq!(ts.to_string(), "2023031219-0700");
    }
}
