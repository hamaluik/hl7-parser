use thiserror::Error;

pub(crate) type VResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

/// Errors that can occur during parsing
#[derive(Debug, Error)]
pub enum ParseError {
    /// The parsing failed for some reason, the reason isn't available.
    ///
    /// **TODO**: better errors!
    #[error("Failed to parse. TODO: better error messages")]
    Failed,
}

/// Errors that can occur during parsing timestamps
#[derive(Debug, Error)]
pub enum TimeParseError {
    /// The parsing failed for some reason parsing field `this.0`
    #[error("Failed to parse timestamp: parsing {0} failed.")]
    ParsingFailed(&'static str),

    /// The timestamp parsing failed because an invalid value (`this.0`) was supplied
    /// (that would make an invalid date/time)
    #[error("Invalid value for {0} (would make an invalid date/time)")]
    InvalidComponentRange(&'static str),
}
