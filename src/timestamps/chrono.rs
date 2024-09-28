//! All implementations here are implemented as `TryFrom` and `From` traits
//! between the `TimeStamp` struct and various chrono types. This allows for
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
//! use hl7_parser::timestamps::{TimeStamp, TimeStampOffset};
//! use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, Utc, Datelike, Timelike};
//!
//! let ts = TimeStamp {
//!    year: 2023,
//!    month: Some(3),
//!    day: Some(12),
//!    hour: Some(19),
//!    minute: Some(59),
//!    second: Some(5),
//!    microsecond: Some(1234),
//!    offset: Some(TimeStampOffset {
//!        hours: -7,
//!        minutes: 0,
//!    })
//! };
//!
//! let datetime: DateTime<FixedOffset> = ts.try_into().unwrap();
//! assert_eq!(datetime.year(), 2023);
//! assert_eq!(datetime.month(), 3);
//! assert_eq!(datetime.day(), 12);
//! assert_eq!(datetime.hour(), 19);
//! assert_eq!(datetime.minute(), 59);
//! assert_eq!(datetime.second(), 5);
//! assert_eq!(datetime.nanosecond(), 1234 * 1000);
//! assert_eq!(datetime.offset().local_minus_utc() / 3600, -7);
//! assert_eq!(datetime.offset().local_minus_utc() % 3600, 0);
//! ```
//!
//! ```
//! use hl7_parser::timestamps::{TimeStamp, TimeStampOffset};
//! use chrono::{DateTime, Utc, NaiveDate, TimeZone};
//!
//! let datetime = Utc.from_utc_datetime(
//!     &NaiveDate::from_ymd_opt(2023, 3, 12).unwrap()
//!     .and_hms_opt(19, 59, 5).unwrap(),
//! );
//!
//! let ts: TimeStamp = datetime.into();
//! assert_eq!(ts.year, 2023);
//! assert_eq!(ts.month, Some(3));
//! assert_eq!(ts.day, Some(12));
//! assert_eq!(ts.hour, Some(19));
//! assert_eq!(ts.minute, Some(59));
//! assert_eq!(ts.second, Some(5));
//! assert_eq!(ts.microsecond, Some(0));
//! assert_eq!(ts.offset, Some(TimeStampOffset {
//!    hours: 0,
//!    minutes: 0,
//! }));
//! ```

use super::{TimeParseError, TimeStamp, TimeStampOffset};
use chrono::{
    offset::LocalResult, DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime,
    TimeZone, Timelike,
};

/// Attempt to convert a `TimeStamp` into a `NaiveDate`. If the `TimeStamp` is
/// missing date components, those components will be set to `1`.
impl TryFrom<TimeStamp> for NaiveDate {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let TimeStamp {
            year, month, day, ..
        } = value;

        let month = month.unwrap_or(1);
        let day = day.unwrap_or(1);

        let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
            .ok_or(TimeParseError::InvalidComponentRange("date does not exist"))?;
        Ok(date)
    }
}

/// Convert a `NaiveDate` into a `TimeStamp`. The `TimeStamp` will have the
/// date components set to the `NaiveDate`'s components and the time components
/// set to `None`.
impl From<NaiveDate> for TimeStamp {
    fn from(value: NaiveDate) -> Self {
        let year = value.year() as u16;
        let month = Some(value.month() as u8);
        let day = Some(value.day() as u8);
        let hour = None;
        let minute = None;
        let second = None;
        let microsecond = None;
        let offset = None;
        TimeStamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            offset,
        }
    }
}

/// Attempt to convert a `TimeStamp` into a `NaiveDateTime`. If the `TimeStamp`
/// is missing time components, those components will be set to zero.
impl TryFrom<TimeStamp> for NaiveDateTime {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let date = NaiveDate::try_from(value)?;
        let time = NaiveTime::from_hms_micro_opt(
            value.hour.unwrap_or(0) as u32,
            value.minute.unwrap_or(0) as u32,
            value.second.unwrap_or(0) as u32,
            value.microsecond.unwrap_or(0),
        )
        .ok_or(TimeParseError::InvalidComponentRange("time does not exist"))?;
        Ok(NaiveDateTime::new(date, time))
    }
}

/// Convert a `NaiveDateTime` into a `TimeStamp`. The `TimeStamp` will have the
/// date and time components set to the `NaiveDateTime`'s components and the
/// offset components set to `None`.
impl From<NaiveDateTime> for TimeStamp {
    fn from(value: NaiveDateTime) -> Self {
        let year = value.year() as u16;
        let month = Some(value.month() as u8);
        let day = Some(value.day() as u8);
        let hour = Some(value.hour() as u8);
        let minute = Some(value.minute() as u8);
        let second = Some(value.second() as u8);
        let microsecond = Some(value.nanosecond() / 1000);
        let offset = None;
        TimeStamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            offset,
        }
    }
}

/// Attempt to convert a `TimeStamp` into a `DateTime<FixedOffset>`. If the
/// `TimeStamp` is missing date components, those components will be set to `1`.
/// If the `TimeStamp` is missing time components, those components will be set
/// to zero. If the `TimeStamp` is missing offset components, those components
/// will be set to zero.
impl TryFrom<TimeStamp> for LocalResult<DateTime<FixedOffset>> {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let TimeStamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            offset,
        } = value;

        let month = month.unwrap_or(1);
        let day = day.unwrap_or(1);
        let date = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
            .ok_or(TimeParseError::InvalidComponentRange("date does not exist"))?;

        let hour = hour.unwrap_or(0);
        let minute = minute.unwrap_or(0);
        let second = second.unwrap_or(0);
        let microsecond = microsecond.unwrap_or(0);

        let time =
            NaiveTime::from_hms_micro_opt(hour as u32, minute as u32, second as u32, microsecond)
                .ok_or(TimeParseError::InvalidComponentRange("time does not exist"))?;

        let offset = offset.unwrap_or_default();
        let offset_hours = offset.hours as i32;
        let offset_minutes = offset.minutes as i32;
        let offset = FixedOffset::east_opt(offset_hours * 3600 + offset_minutes * 60).ok_or(
            TimeParseError::InvalidComponentRange("offset does not exist"),
        )?;

        let datetime = NaiveDateTime::new(date, time);
        let datetime = datetime.and_local_timezone(offset);
        Ok(datetime)
    }
}

/// Attempt to convert a `TimeStamp` into a `DateTime<Tz>`. If the `TimeStamp` is
/// missing date components, those components will be set to `1`. If the
/// `TimeStamp` is missing time components, those components will be set to zero.
/// If the `TimeStamp` is missing offset components, those components will be set
/// to zero.
///
/// Note that this implementation will return an error if the `TimeStamp` is
/// ambiguous or does not exist.
impl<Tz> TryFrom<TimeStamp> for DateTime<Tz>
where
    Tz: TimeZone,
    DateTime<Tz>: From<DateTime<FixedOffset>>,
{
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let datetime: LocalResult<DateTime<FixedOffset>> = LocalResult::try_from(value)?;
        match datetime {
            LocalResult::Single(datetime) => Ok(datetime.into()),
            LocalResult::Ambiguous(earliest, latest) => Err(TimeParseError::AmbiguousTime(
                earliest.to_rfc3339(),
                latest.to_rfc3339(),
            )),
            LocalResult::None => Err(TimeParseError::InvalidComponentRange(
                "datetime does not exist",
            )),
        }
    }
}

/// Convert a `DateTime` into a `TimeStamp`. The `TimeStamp` will have the date
/// and time components set to the `DateTime`'s components and the offset
/// components set to the `DateTime`'s offset components.
impl<Tz> From<DateTime<Tz>> for TimeStamp
where
    Tz: TimeZone,
    DateTime<Tz>: Into<DateTime<FixedOffset>>,
{
    fn from(value: DateTime<Tz>) -> Self {
        let datetime: DateTime<FixedOffset> = value.into();

        let year = datetime.year() as u16;
        let month = Some(datetime.month() as u8);
        let day = Some(datetime.day() as u8);
        let hour = Some(datetime.hour() as u8);
        let minute = Some(datetime.minute() as u8);
        let second = Some(datetime.second() as u8);
        let microsecond = Some(datetime.nanosecond() / 1000);
        let offset = Some(TimeStampOffset {
            hours: (datetime.offset().local_minus_utc() / 3600) as i8,
            minutes: (datetime.offset().local_minus_utc() % 3600) as u8,
        });

        TimeStamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
            microsecond,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::timestamps::TimeStampOffset;
    use chrono::{Timelike, Utc};

    use super::*;

    #[test]
    fn can_convert_timestamp_to_date() {
        let ts = TimeStamp {
            year: 2023,
            month: Some(3),
            day: Some(12),
            hour: Some(19),
            minute: Some(59),
            second: None,
            microsecond: None,
            offset: None,
        };
        let actual = NaiveDate::try_from(ts).unwrap();
        assert_eq!(actual.year(), 2023);
        assert_eq!(actual.month(), 3);
        assert_eq!(actual.day(), 12);
    }

    #[test]
    fn can_convert_timestamp_to_datetime_with_fixed_offset() {
        let ts = TimeStamp {
            year: 2023,
            month: Some(3),
            day: Some(12),
            hour: Some(19),
            minute: Some(59),
            second: Some(5),
            microsecond: Some(1234),
            offset: Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            }),
        };
        let actual = DateTime::<FixedOffset>::try_from(ts).unwrap();
        assert_eq!(actual.year(), 2023);
        assert_eq!(actual.month(), 3);
        assert_eq!(actual.day(), 12);
        assert_eq!(actual.hour(), 19);
        assert_eq!(actual.minute(), 59);
        assert_eq!(actual.second(), 5);
        assert_eq!(actual.nanosecond(), 1234 * 1000);
        assert_eq!(actual.offset().local_minus_utc() / 3600, -7);
        assert_eq!(actual.offset().local_minus_utc() % 3600, 0);
    }

    #[test]
    fn can_convert_timestamp_datetime_with_utc_offset() {
        let ts = TimeStamp {
            year: 2023,
            month: Some(3),
            day: Some(12),
            hour: Some(19),
            minute: Some(59),
            second: Some(5),
            microsecond: Some(1234),
            offset: Some(TimeStampOffset {
                hours: -7,
                minutes: 0,
            }),
        };
        let actual = DateTime::<Utc>::try_from(ts).unwrap();

        assert_eq!(actual.year(), 2023);
        assert_eq!(actual.month(), 3);
        assert_eq!(actual.day(), 13);
        assert_eq!(actual.hour(), 2);
        assert_eq!(actual.minute(), 59);
        assert_eq!(actual.second(), 5);
        assert_eq!(actual.nanosecond(), 1234 * 1000);
    }

    #[test]
    fn can_convert_datetime_to_timestamp() {
        let datetime = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(2023, 3, 12)
                .unwrap()
                .and_hms_opt(19, 59, 5)
                .unwrap(),
        );
        let actual = TimeStamp::from(datetime);
        assert_eq!(actual.year, 2023);
        assert_eq!(actual.month, Some(3));
        assert_eq!(actual.day, Some(12));
        assert_eq!(actual.hour, Some(19));
        assert_eq!(actual.minute, Some(59));
        assert_eq!(actual.second, Some(5));
        assert_eq!(actual.microsecond, Some(0));
        assert_eq!(
            actual.offset,
            Some(TimeStampOffset {
                hours: 0,
                minutes: 0
            })
        );
    }
}
