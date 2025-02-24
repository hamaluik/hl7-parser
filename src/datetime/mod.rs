use std::fmt::Display;

mod timestamp;
pub use timestamp::*;
mod time;
pub use time::*;
mod date;
pub use date::*;

/// Utilies to convert back and forth between chrono's data structures and the hl7-parser ones
#[cfg(feature = "chrono")]
pub mod chrono;

/// Utilies to convert back and forth between time's data structures and the hl7-parser ones
#[cfg(feature = "time")]
pub mod time_crate;

/// Utilies to convert back and forth between jiff's data structures and the hl7-parser ones
#[cfg(feature = "jiff")]
pub mod jiff;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErroredDateTimeComponent {
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

impl Display for ErroredDateTimeComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroredDateTimeComponent::Year => write!(f, "year"),
            ErroredDateTimeComponent::Month => write!(f, "month"),
            ErroredDateTimeComponent::Day => write!(f, "day"),
            ErroredDateTimeComponent::Hour => write!(f, "hour"),
            ErroredDateTimeComponent::Minute => write!(f, "minute"),
            ErroredDateTimeComponent::Second => write!(f, "second"),
            ErroredDateTimeComponent::Microsecond => write!(f, "microsecond"),
            ErroredDateTimeComponent::Offset => write!(f, "offset"),
            ErroredDateTimeComponent::Date => write!(f, "date"),
            ErroredDateTimeComponent::Time => write!(f, "time"),
            ErroredDateTimeComponent::DateTime => write!(f, "date and time"),
        }
    }
}

/// Errors that can result from parsing HL7 timestamps
#[derive(thiserror::Error, Debug)]
pub enum DateTimeParseError {
    #[error("Failed to parse '{0}' component of timestamp")]
    ParsingFailed(&'static str),
    #[error("Unexpected character '{1}' in timestamp at position {0}")]
    UnexpectedCharacter(usize, char),
    #[error("Invalid component range: {0:}")]
    InvalidComponentRange(ErroredDateTimeComponent),
    #[error("Ambiguous time, could be {0} or {1}")]
    AmbiguousTime(String, String),
    #[error("Missing component: {0:}")]
    MissingComponent(ErroredDateTimeComponent),
}

/// Trait for parsing HL7 date and time strings into `Date`, `Time`, and `TimeStamp` structs
pub trait DateTime {
    /// Parse an HL7 date and/or time string into a `Date`, `Time`, or `TimeStamp` struct
    fn parse(s: &str, lenient_trailing_chars: bool) -> Result<Self, DateTimeParseError>
    where
        Self: Sized;

    /// Parse an HL7 date and/or time string into a `Date`, `Time`, or `TimeStamp` struct, with
    /// strict parsing dissallowing trailing characters
    fn parse_strict(s: &str) -> Result<Self, DateTimeParseError>
    where
        Self: Sized,
    {
        Self::parse(s, false)
    }
}

impl DateTime for Time {
    fn parse(s: &str, lenient_trailing_chars: bool) -> Result<Self, DateTimeParseError> {
        parse_time(s, lenient_trailing_chars)
    }
}

impl DateTime for Date {
    fn parse(s: &str, lenient_trailing_chars: bool) -> Result<Self, DateTimeParseError> {
        parse_date(s, lenient_trailing_chars)
    }
}

impl DateTime for TimeStamp {
    fn parse(s: &str, lenient_trailing_chars: bool) -> Result<Self, DateTimeParseError> {
        parse_timestamp(s, lenient_trailing_chars)
    }
}
