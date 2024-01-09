use thiserror::Error;

use crate::parser::Span;

pub(crate) type VResult<I, O, E = nom::error::VerboseError<I>> = Result<(I, O), nom::Err<E>>;

/// Errors that can occur during parsing
#[derive(Debug, Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseError {
    /// The parsing failed for some reason
    #[error(
        "ParsedMessage parsing failed at position {position} (line {line} column {column}): `{fragment}`"
    )]
    FailedToParse {
        position: usize,
        line: usize,
        column: usize,
        fragment: String,
    },

    #[error("ParsedMessage parsing failed because of incomplete input.{}", .0.map(|s| format!(" Need at least {s} more characters to continue.")).unwrap_or_default())]
    IncompleteInput(Option<usize>),
}

impl<'s> From<nom::Err<nom::error::Error<Span<'s>>>> for ParseError {
    fn from(e: nom::Err<nom::error::Error<Span<'s>>>) -> Self {
        match e {
            nom::Err::Incomplete(nom::Needed::Unknown) => ParseError::IncompleteInput(None),
            nom::Err::Incomplete(nom::Needed::Size(size)) => {
                ParseError::IncompleteInput(Some(size.get()))
            }
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let position = e.input.location_offset();
                let line = e.input.location_line() as usize;
                let column = e.input.naive_get_utf8_column();
                ParseError::FailedToParse {
                    position,
                    line,
                    column,
                    fragment: e.input.fragment().chars().take(3).collect(),
                }
            }
        }
    }
}

/// Errors that can occur when parsing timestamps
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimeParseError {
    /// The parsing failed for some reason parsing field `this.0`
    #[error("Failed to parse timestamp: parsing {0} failed.")]
    ParsingFailed(&'static str),

    /// The timestamp parsing failed because an invalid value (`this.0`) was supplied
    /// (that would make an invalid date/time)
    #[error("Invalid value for {0} (would make an invalid date/time)")]
    InvalidComponentRange(&'static str),
}
