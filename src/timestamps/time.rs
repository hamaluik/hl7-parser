//! All implementations here are implemented as `TryFrom` and `From` traits
//! between the `TimeStamp` struct and various `time` types. This allows for
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
//! use time::{PrimitiveDateTime, OffsetDateTime};
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
//!     })
//! };
//!
//! let datetime: OffsetDateTime = ts.try_into().unwrap();
//! assert_eq!(datetime.year(), 2023);
//! assert_eq!(datetime.month(), time::Month::March);
//! assert_eq!(datetime.day(), 12);
//! assert_eq!(datetime.hour(), 19);
//! assert_eq!(datetime.minute(), 59);
//! assert_eq!(datetime.second(), 5);
//! assert_eq!(datetime.microsecond(), 1234);
//! assert_eq!(datetime.offset().whole_hours(), -7);
//! ```

use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

use super::{TimeComponent, TimeParseError, TimeStamp, TimeStampOffset};

impl TryFrom<TimeStamp> for Date {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let TimeStamp {
            year, month, day, ..
        } = value;

        match (year, month, day) {
            (year, Some(month), Some(day)) => {
                let month = Month::try_from(month as u8).map_err(|_| {
                    TimeParseError::InvalidComponentRange(TimeComponent::Month)
                })?;

                Ok(Date::from_calendar_date(year.into(), month, day).map_err(|_| {
                    TimeParseError::InvalidComponentRange(TimeComponent::Date)
                })?)
            },
            (_year, Some(_), None) => Err(TimeParseError::MissingComponent(TimeComponent::Day)),
            (_year, None, _) => Err(TimeParseError::MissingComponent(TimeComponent::Month)),
        }
    }
}

impl From<Date> for TimeStamp {
    fn from(value: Date) -> Self {
        let (year, month, day) = value.to_calendar_date();

        TimeStamp {
            year: year as u16,
            month: Some(month as u8),
            day: Some(day),
            hour: None,
            minute: None,
            second: None,
            microsecond: None,
            offset: None,
        }
    }
}

impl TryFrom<TimeStamp> for PrimitiveDateTime {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        let date = Date::try_from(value)?;

        if value.hour.is_none() {
            return Err(TimeParseError::MissingComponent(TimeComponent::Hour));
        }
        if value.minute.is_none() {
            return Err(TimeParseError::MissingComponent(TimeComponent::Minute));
        }
        if value.second.is_none() {
            return Err(TimeParseError::MissingComponent(TimeComponent::Second));
        }

        let TimeStamp {
            hour, minute, second, microsecond, ..
        } = value;

        let time = Time::from_hms_micro(hour.unwrap(), minute.unwrap(), second.unwrap(), microsecond.unwrap_or(0)).map_err(|_| TimeParseError::InvalidComponentRange(TimeComponent::Time))?;

        Ok(PrimitiveDateTime::new(date, time))
    }
}

impl From<PrimitiveDateTime> for TimeStamp {
    fn from(value: PrimitiveDateTime) -> Self {
        let date = value.date();
        let time = value.time();

        TimeStamp {
            year: date.year() as u16,
            month: Some(date.month().into()),
            day: Some(date.day()),
            hour: Some(time.hour()),
            minute: Some(time.minute()),
            second: Some(time.second()),
            microsecond: Some(time.microsecond()),
            offset: None,
        }
    }
}

impl TryFrom<TimeStamp> for OffsetDateTime {
    type Error = TimeParseError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        if value.offset.is_none() {
            return Err(TimeParseError::MissingComponent(TimeComponent::Offset));
        }

        let datetime = PrimitiveDateTime::try_from(value)?;
        let offset = value.offset.unwrap();
        let offset = UtcOffset::from_hms(offset.hours, offset.minutes as i8, 0).map_err(|_| TimeParseError::InvalidComponentRange(TimeComponent::Offset))?;

        let date = datetime.date();
        let time = datetime.time();

        Ok(OffsetDateTime::new_in_offset(date, time, offset))
    }
}

impl From<OffsetDateTime> for TimeStamp {
    fn from(value: OffsetDateTime) -> Self {
        let date = value.date();
        let time = value.time();
        let offset = value.offset();

        TimeStamp {
            year: date.year() as u16,
            month: Some(date.month().into()),
            day: Some(date.day()),
            hour: Some(time.hour()),
            minute: Some(time.minute()),
            second: Some(time.second()),
            microsecond: Some(time.microsecond()),
            offset: Some(TimeStampOffset {
                hours: offset.whole_hours(),
                minutes: (offset.whole_minutes() % 60).abs() as u8,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
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
        let actual = Date::try_from(ts).unwrap();
        assert_eq!(actual, Date::from_calendar_date(2023, Month::March, 12).unwrap());
    }

    #[test]
    fn can_convert_timestamp_to_offsetdateime() {
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
        let actual = OffsetDateTime::try_from(ts).unwrap();
        assert_eq!(actual.year(), 2023);
        assert_eq!(actual.month(), Month::March);
        assert_eq!(actual.day(), 12);
        assert_eq!(actual.hour(), 19);
        assert_eq!(actual.minute(), 59);
        assert_eq!(actual.second(), 5);
        assert_eq!(actual.microsecond(), 1234);
        assert_eq!(actual.offset(), UtcOffset::from_hms(-7, 0, 0).unwrap());
    }
}

