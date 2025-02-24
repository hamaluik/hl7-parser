use crate::parser::Span;
use nom::{
    bytes::complete::take_while_m_n,
    combinator::{map_res, opt},
    IResult,
};
use std::{fmt::Display, str::FromStr};

use super::DateTimeParseError;

/// A parsed date without a time component
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
    /// The year of the date
    pub year: u16,
    /// The month of the date (1-12)
    pub month: Option<u8>,
    /// The day of the date (1-31)
    pub day: Option<u8>,
}

/// Parse an HL7 date in the format: `YYYY[MM[DD]`
///
/// # Arguments
/// * `s` - The string to parse
/// * `lenient_trailing_chars` - If true, allow trailing characters after the date, otherwise throw
///   an error
///
/// # Example
///
/// ```
/// use hl7_parser::datetime::{parse_date, Date};
///
/// let date: Date = parse_date("20230312", false).expect("can parse date");
///
/// assert_eq!(date.year, 2023);
/// assert_eq!(date.month, Some(3));
/// assert_eq!(date.day, Some(12));
/// ```
pub fn parse_date(s: &str, lenient_trailing_chars: bool) -> Result<Date, DateTimeParseError> {
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
    let (s, year): (Span, u16) =
        digit4(s).map_err(|_| DateTimeParseError::ParsingFailed("year"))?;
    let (s, month): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("month"))?;
    let (s, day): (Span, Option<u8>) =
        opt(digit2)(s).map_err(|_| DateTimeParseError::ParsingFailed("day"))?;

    if !lenient_trailing_chars && !s.is_empty() {
        return Err(DateTimeParseError::UnexpectedCharacter(
            s.offset,
            s.input.chars().next().unwrap_or_default(),
        ));
    }

    Ok(Date { year, month, day })
}

/// Implement `FromStr` for `TimeStamp` to allow parsing timestamps from strings
impl FromStr for Date {
    type Err = DateTimeParseError;

    /// Synonymous with `parse_timestamp`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_date(s, false)
    }
}

/// Implement `Display` for `TimeStamp` to allow formatting timestamps as HL7 strings
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.year)?;
        if let Some(month) = self.month {
            write!(f, "{:02}", month)?;
            if let Some(day) = self.day {
                write!(f, "{:02}", day)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_date() {
        let date = "20230312";
        let date = parse_date(date, false).expect("can parse date");

        assert_eq!(date.year, 2023);
        assert_eq!(date.month, Some(3));
        assert_eq!(date.day, Some(12));
    }
}
