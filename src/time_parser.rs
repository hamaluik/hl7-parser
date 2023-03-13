use crate::Span;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::one_of,
    combinator::{map_res, opt},
    sequence::preceded,
    IResult,
};
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

pub fn parse_time<'s>(s: &'s str) -> Result<OffsetDateTime, nom::Err<nom::error::Error<Span<'s>>>> {
    fn is_decimal_digit(c: char) -> bool {
        c.is_digit(10)
    }

    fn from_digits(i: Span) -> Result<usize, std::num::ParseIntError> {
        usize::from_str_radix(i.fragment(), 10)
    }

    fn digit2(input: Span) -> IResult<Span, usize> {
        map_res(take_while_m_n(2, 2, is_decimal_digit), from_digits)(input)
    }

    fn digit4(input: Span) -> IResult<Span, usize> {
        map_res(take_while_m_n(4, 4, is_decimal_digit), from_digits)(input)
    }

    let s = Span::new(s);
    let (s, year) = digit4(s)?;
    let (s, month) = opt(digit2)(s)?;
    let (s, day) = opt(digit2)(s)?;
    let (s, hour) = opt(digit2)(s)?;
    let (s, minute) = opt(digit2)(s)?;
    let (s, second) = opt(digit2)(s)?;
    let (s, second_fracs) = opt(preceded(tag("."), take_while_m_n(1, 4, is_decimal_digit)))(s)?;
    let (s, offset_dir) = opt(one_of("+-"))(s)?;
    let (s, offset_hours) = opt(digit2)(s)?;
    let (_, offset_minutes) = opt(digit2)(s)?;

    let month = match month.unwrap_or(1) {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => todo!("bad month error"),
    };
    let day = day.unwrap_or(1);
    let date =
        Date::from_calendar_date(year as i32, month, day as u8).expect("TODO: date number error");

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
    let microseconds = usize::from_str_radix(second_fracs.fragment(), 10)
        .expect("can parse fractional seconds as number")
        * fracs_multiplier;
    let time = Time::from_hms_micro(hour as u8, minute as u8, second as u8, microseconds as u32)
        .expect("TODO: time number error");

    let offset_dir = match offset_dir.unwrap_or('+') {
        '-' => -1,
        _ => 1,
    };
    let offset_hours = offset_hours.unwrap_or(0);
    let offset_minutes = offset_minutes.unwrap_or(0);
    let offset_hours = (offset_hours as i8) * offset_dir;
    let offset_minutes = (offset_minutes as i8) * offset_dir;
    let offset =
        UtcOffset::from_hms(offset_hours, offset_minutes, 0).expect("TODO: offset number error");

    let datetime = PrimitiveDateTime::new(date, time);
    let datetime = datetime.assume_offset(offset);

    Ok(datetime)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_time_with_offsets() {
        let ts = "20230312195905.1234-0700";
        let ts = parse_time(ts).expect("can parse timestamp");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), Month::March);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 05);
        assert_eq!(ts.microsecond(), 123_400);
        assert_eq!(ts.offset().whole_hours(), -7);
        assert_eq!(ts.offset().minutes_past_hour(), 0);
    }

    #[test]
    fn can_parse_time_without_offsets() {
        let ts = "20230312195905.1234";
        let ts = parse_time(ts).expect("can parse timestamp");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), Month::March);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 05);
        assert_eq!(ts.microsecond(), 123_400);
        assert!(ts.offset().is_utc());
    }

    #[test]
    fn can_parse_time_without_offsets_or_fractional_seconds() {
        let ts = "20230312195905";
        let ts = parse_time(ts).expect("can parse timestamp");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), Month::March);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 05);
        assert_eq!(ts.microsecond(), 0);
        assert!(ts.offset().is_utc());
    }

    #[test]
    fn can_parse_time_with_offsets_without_fractional_seconds() {
        let ts = "20230312195905-0700";
        let ts = parse_time(ts).expect("can parse timestamp");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), Month::March);
        assert_eq!(ts.day(), 12);
        assert_eq!(ts.hour(), 19);
        assert_eq!(ts.minute(), 59);
        assert_eq!(ts.second(), 05);
        assert_eq!(ts.microsecond(), 0);
        assert_eq!(ts.offset().whole_hours(), -7);
        assert_eq!(ts.offset().minutes_past_hour(), 0);
    }

    #[test]
    fn can_parse_time_with_only_year() {
        let ts = "2023";
        let ts = parse_time(ts).expect("can parse timestamp");

        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), Month::January);
        assert_eq!(ts.day(), 1);
        assert_eq!(ts.hour(), 0);
        assert_eq!(ts.minute(), 0);
        assert_eq!(ts.second(), 00);
        assert_eq!(ts.microsecond(), 0);
        assert!(ts.offset().is_utc());
    }

    #[test]
    fn cant_parse_bad_timestamps() {
        assert!(parse_time("23").is_err());
        assert!(parse_time("abcd").is_err());
    }
}
