use crate::{parser::Span, TimeParseError};
use chrono::{DateTime, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, NaiveTime};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::one_of,
    combinator::{map_res, opt},
    sequence::preceded,
    IResult,
};

/// Parse an HL7 timestamp
///
/// Any missing components from the timestamp will be substituted with the first of that time
/// period (for example, if the month is not provided, it will default to 1 (January),
/// hour will default to `0`, offset will default to _UTC_)
///
/// # Arguments
///
/// * `s` - A string slice representing the HL7 timestamp (format: `YYYY[MM[DD[HH[MM[SS[.S[S[S[S]]]]]]]]][+/-ZZZZ]`)
pub fn parse_timestamp_chrono<'s>(
    s: &'s str,
) -> Result<LocalResult<DateTime<FixedOffset>>, TimeParseError> {
    fn is_decimal_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn from_digits(i: Span) -> Result<usize, std::num::ParseIntError> {
        i.fragment().parse::<usize>()
    }

    fn digit2(input: Span) -> IResult<Span, usize> {
        map_res(take_while_m_n(2, 2, is_decimal_digit), from_digits)(input)
    }

    fn digit4(input: Span) -> IResult<Span, usize> {
        map_res(take_while_m_n(4, 4, is_decimal_digit), from_digits)(input)
    }

    let s = Span::new(s);
    let (s, year) = digit4(s).map_err(|_| TimeParseError::ParsingFailed("year"))?;
    let (s, month) = opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("month"))?;
    let (s, day) = opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("day"))?;
    let (s, hour) = opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("hour"))?;
    let (s, minute) = opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("minute"))?;
    let (s, second) = opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("second"))?;
    let (s, second_fracs) = opt(preceded(tag("."), take_while_m_n(1, 4, is_decimal_digit)))(s)
        .map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            TimeParseError::ParsingFailed("fractional seconds")
        })?;
    let (s, offset_dir) =
        opt(one_of("+-"))(s).map_err(|_: nom::Err<nom::error::Error<Span<'s>>>| {
            TimeParseError::ParsingFailed("offset direction")
        })?;
    let (s, offset_hours) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("offset hours"))?;
    let (_, offset_minutes) =
        opt(digit2)(s).map_err(|_| TimeParseError::ParsingFailed("offset minutes"))?;

    let month = month.unwrap_or(1);
    let day = day.unwrap_or(1);
    let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .ok_or(TimeParseError::InvalidComponentRange("date does not exist"))?;

    let hour = hour.unwrap_or(0);
    let minute = minute.unwrap_or(0);
    let second = second.unwrap_or(0);

    let second_fracs = second_fracs.unwrap_or(Span::new("0"));
    let fracs_multiplier = match second_fracs.len() {
        1 => 100_000,
        2 => 10_000,
        3 => 1_000,
        4 => 100,
        _ => panic!("second_fracs.len() not in 1..=4"),
    };
    let microseconds = second_fracs
        .fragment()
        .parse::<u32>()
        .expect("can parse fractional seconds as number")
        * fracs_multiplier;
    let time =
        NaiveTime::from_hms_micro_opt(hour as u32, minute as u32, second as u32, microseconds)
            .ok_or(TimeParseError::InvalidComponentRange("time does not exist"))?;

    let offset_dir = match offset_dir.unwrap_or('+') {
        '-' => -1,
        _ => 1,
    };
    let offset_hours = offset_hours.unwrap_or(0) as i32 * offset_dir;
    let offset_minutes = offset_minutes.unwrap_or(0) as i32;
    let offset = FixedOffset::east_opt(offset_hours * 3600 + offset_minutes * 60)
        .ok_or(TimeParseError::InvalidComponentRange("offset does not exist"))?;

    let datetime = NaiveDateTime::new(date, time);
    let datetime = datetime.and_local_timezone(offset);
    Ok(datetime)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn can_parse_time_with_offsets() {
        let ts = "20230312195905.1234-0700";
        let ts = parse_timestamp_chrono(ts)
            .expect("can parse timestamp")
            .earliest()
            .expect("can convert to datetime");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 3);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 5);
        assert_eq!(ts.nanosecond(), 123_400_000);
        assert_eq!(ts.offset().local_minus_utc() / 3600, -7);
        assert_eq!(ts.offset().local_minus_utc() % 3600, 0);
    }

    #[test]
    fn can_parse_time_without_offsets() {
        let ts = "20230312195905.1234";
        let ts = parse_timestamp_chrono(ts)
            .expect("can parse timestamp")
            .earliest()
            .expect("can convert to datetime");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 3);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 5);
        assert_eq!(ts.nanosecond(), 123_400_000);
        assert_eq!(ts.offset().local_minus_utc(), 0);
    }

    #[test]
    fn can_parse_time_without_offsets_or_fractional_seconds() {
        let ts = "20230312195905";
        let ts = parse_timestamp_chrono(ts)
            .expect("can parse timestamp")
            .earliest()
            .expect("can convert to datetime");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 3);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 5);
        assert_eq!(ts.nanosecond(), 0);
        assert_eq!(ts.offset().local_minus_utc(), 0);
    }

    #[test]
    fn can_parse_time_with_offsets_without_fractional_seconds() {
        let ts = "20230312195905-0700";
        let ts = parse_timestamp_chrono(ts)
            .expect("can parse timestamp")
            .earliest()
            .expect("can convert to datetime");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 3);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 5);
        assert_eq!(ts.nanosecond(), 0);
        assert_eq!(ts.offset().local_minus_utc() / 3600, -7);
        assert_eq!(ts.offset().local_minus_utc() % 3600, 0);
    }

    #[test]
    fn can_parse_time_with_only_year() {
        let ts = "2023";
        let ts = parse_timestamp_chrono(ts)
            .expect("can parse timestamp")
            .earliest()
            .expect("can convert to datetime");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 1);
        assert_eq!(ts.day(), 1);
        assert_eq!(ts.hour(), 0);
        assert_eq!(ts.minute(), 0);
        assert_eq!(ts.second(), 00);
        assert_eq!(ts.nanosecond(), 0);
        assert_eq!(ts.offset().local_minus_utc(), 0);
    }

    #[test]
    fn cant_parse_bad_timestamps() {
        assert!(parse_timestamp_chrono("23").is_err());
        assert!(parse_timestamp_chrono("abcd").is_err());
        assert!(parse_timestamp_chrono("20230230").is_err());
    }
}
