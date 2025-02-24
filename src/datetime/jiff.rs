//! All implementations here are implemented as `TryFrom` and `From` traits
//! between the `TimeStamp` struct and various `chrono` types. This allows for
//! easy conversion between the two types. The `TryFrom` implementations will
//! return an error if the conversion is not possible, such as if the date or
//! time components are invalid. The `From` implementations will always succeed
//! and will set missing components to zero or the epoch if necessary.
//!
//! View the `TimeStamp` struct's documentation for more information on exactly
//! which traits are implemented.
//!
//! # Examples
//!
//! ```
//! use hl7_parser::datetime::{TimeStamp, TimeStampOffset};
//! use jiff::civil::{date, time, datetime};
//!
//! let timestamp = TimeStamp {
//!    year: 2021,
//!    month: Some(1),
//!    day: Some(1),
//!    hour: Some(12),
//!    minute: Some(0),
//!    second: Some(0),
//!    microsecond: Some(0),
//!    offset: None,
//! };
//!
//! let datetime = datetime(2021, 1, 1, 12, 0, 0, 0);
//! assert_eq!(TimeStamp::from(datetime), timestamp);
//! ```
use jiff::{
    civil::{date, Date, DateTime, Time},
    Zoned,
};

use super::{DateTimeParseError, TimeStamp as HL7TimeStamp, Time as HL7Time, Date as HL7Date};

impl TryFrom<HL7TimeStamp> for Date {
    type Error = DateTimeParseError;

    fn try_from(value: HL7TimeStamp) -> Result<Self, Self::Error> {
        let HL7TimeStamp {
            year, month, day, ..
        } = value;

        let month = month.unwrap_or(1);
        let day = day.unwrap_or(1);

        Ok(date(year as i16, month as i8, day as i8))
    }
}

impl From<Date> for HL7TimeStamp {
    fn from(value: Date) -> Self {
        let year = value.year();
        let month = value.month();
        let day = value.day();

        HL7TimeStamp {
            year: year as u16,
            month: Some(month as u8),
            day: Some(day as u8),
            ..Default::default()
        }
    }
}

impl TryFrom<HL7Date> for Date {
    type Error = DateTimeParseError;

    fn try_from(value: super::Date) -> Result<Self, Self::Error> {
        let super::Date { year, month, day } = value;

        let month = month.unwrap_or(1);
        let day = day.unwrap_or(1);

        Ok(date(year as i16, month as i8, day as i8))
    }
}

impl From<Date> for HL7Date {
    fn from(value: Date) -> Self {
        let year = value.year();
        let month = value.month();
        let day = value.day();

        super::Date {
            year: year as u16,
            month: Some(month as u8),
            day: Some(day as u8),
        }
    }
}

impl TryFrom<HL7Time> for Time {
    type Error = DateTimeParseError;

    fn try_from(value: super::Time) -> Result<Self, Self::Error> {
        let super::Time {
            hour,
            minute,
            second,
            microsecond,
            ..
        } = value;

        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let microsecond = microsecond.unwrap_or(0);
        Ok(jiff::civil::time(
            hour as i8,
            minute as i8,
            second as i8,
            microsecond as i32,
        ))
    }
}

impl From<Time> for HL7Time {
    fn from(value: Time) -> Self {
        let hour = value.hour();
        let minute = value.minute();
        let second = value.second();
        let microsecond = value.microsecond();

        super::Time {
            hour: hour as u8,
            minute: Some(minute as u8),
            second: Some(second as u8),
            microsecond: Some(microsecond as u32),
            offset: None,
        }
    }
}

impl TryFrom<HL7TimeStamp> for Time {
    type Error = DateTimeParseError;

    fn try_from(value: HL7TimeStamp) -> Result<Self, Self::Error> {
        let HL7TimeStamp {
            hour,
            minute,
            second,
            microsecond,
            ..
        } = value;

        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let microsecond = microsecond.unwrap_or(0);
        Ok(jiff::civil::time(
            hour as i8,
            minute as i8,
            second as i8,
            microsecond as i32,
        ))
    }
}

impl TryFrom<HL7TimeStamp> for DateTime {
    type Error = DateTimeParseError;

    fn try_from(value: HL7TimeStamp) -> Result<Self, Self::Error> {
        let date = Date::try_from(value)?;
        let time = Time::try_from(value)?;

        Ok(jiff::civil::datetime(
            date.year(),
            date.month(),
            date.day(),
            time.hour(),
            time.minute(),
            time.second(),
            time.microsecond().into(),
        ))
    }
}

impl From<DateTime> for HL7TimeStamp {
    fn from(value: DateTime) -> Self {
        let date = value.date();
        let time = value.time();

        let year = date.year();
        let month = date.month();
        let day = date.day();

        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        HL7TimeStamp {
            year: year as u16,
            month: Some(month as u8),
            day: Some(day as u8),
            hour: Some(hour as u8),
            minute: Some(minute as u8),
            second: Some(second as u8),
            microsecond: Some(microsecond as u32),
            offset: None,
        }
    }
}

impl TryFrom<Zoned> for HL7TimeStamp {
    type Error = DateTimeParseError;

    fn try_from(value: Zoned) -> Result<Self, Self::Error> {
        let date = value.date();
        let time = value.time();
        let offset = value.offset();

        let year = date.year();
        let month = date.month();
        let day = date.day();

        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        let offset_seconds = offset.seconds();
        let offset_hours: i8 = (offset_seconds / 3600) as i8;
        let offset_minutes: u8 = ((offset_seconds.abs() % 3600) / 60) as u8;

        Ok(HL7TimeStamp {
            year: year as u16,
            month: Some(month as u8),
            day: Some(day as u8),
            hour: Some(hour as u8),
            minute: Some(minute as u8),
            second: Some(second as u8),
            microsecond: Some(microsecond as u32),
            offset: Some(super::TimeStampOffset {
                hours: offset_hours,
                minutes: offset_minutes,
            }),
        })
    }
}

impl TryFrom<HL7TimeStamp> for Zoned {
    type Error = jiff::Error;

    fn try_from(value: HL7TimeStamp) -> Result<Self, Self::Error> {
        let date = Date::try_from(value).unwrap();
        let time = Time::try_from(value).unwrap();
        let offset = value.offset.unwrap_or_default();

        let year = date.year();
        let month = date.month();
        let day = date.day();

        let hour = time.hour();
        let minute = time.minute();
        let second = time.second();
        let microsecond = time.microsecond();

        let offset_seconds = (offset.hours as i32 * 3600) + (offset.minutes as i32 * 60);
        let offset = jiff::tz::Offset::from_seconds(offset_seconds)?;
        let timezone = jiff::tz::TimeZone::fixed(offset);

        let datetime =
            jiff::civil::datetime(year, month, day, hour, minute, second, microsecond.into());

        datetime.to_zoned(timezone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_roundtrip_date() {
        let date = date(2021, 1, 1);
        let hl7_date = HL7Date::from(date);
        let date2 = Date::try_from(hl7_date).unwrap();
        assert_eq!(date, date2);
    }

    #[test]
    fn can_roundtrip_time() {
        let time = jiff::civil::time(12, 0, 0, 0);
        let hl7_time = HL7Time::from(time);
        let time2 = Time::try_from(hl7_time).unwrap();
        assert_eq!(time, time2);
    }

    #[test]
    fn can_roundtrip_timestamp() {
        let timestamp = jiff::civil::datetime(2021, 1, 1, 12, 0, 0, 0);
        let hl7_timestamp = HL7TimeStamp::from(timestamp);
        let timestamp2 = DateTime::try_from(hl7_timestamp).unwrap();
        assert_eq!(timestamp, timestamp2);
    }

    #[test]
    fn can_convert_timestamp_to_zoned() {
        let timestamp = jiff::civil::datetime(2021, 1, 1, 12, 0, 0, 0);
        let hl7_timestamp = HL7TimeStamp::from(timestamp);
        let zoned = Zoned::try_from(hl7_timestamp).unwrap();
        assert_eq!(zoned.date(), timestamp.date());
        assert_eq!(zoned.time(), timestamp.time());
    }

    #[test]
    fn can_convert_zoned_to_timestamp() {
        let timestamp = jiff::civil::datetime(2021, 1, 1, 12, 0, 0, 0);
        let zoned = timestamp.to_zoned(jiff::tz::TimeZone::UTC).unwrap();
        let hl7_timestamp = HL7TimeStamp::try_from(zoned).unwrap();

        assert_eq!(hl7_timestamp.year, 2021);
        assert_eq!(hl7_timestamp.month, Some(1));
        assert_eq!(hl7_timestamp.day, Some(1));
        assert_eq!(hl7_timestamp.hour, Some(12));
        assert_eq!(hl7_timestamp.minute, Some(0));
        assert_eq!(hl7_timestamp.second, Some(0));
        assert_eq!(hl7_timestamp.microsecond, Some(0));
        assert_eq!(hl7_timestamp.offset, Some(crate::datetime::TimeStampOffset { hours: 0, minutes: 0 }));
    }
}
