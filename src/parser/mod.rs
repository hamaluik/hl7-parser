use crate::message::{Component, Field, Repeat, Segment, Separators, Subcomponent};

pub(crate) type Span<'m> = nom_locate::LocatedSpan<&'m str>;

mod component;
mod field;
pub(crate) mod message;
mod msh;
mod repeat;
mod segment;
mod subcomponent;

/// Errors that can occur during parsing
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseError {
    /// The parsing failed for some reason
    #[error(
        "Message parsing failed at position {position} (line {line} column {column}): `{fragment}`"
    )]
    FailedToParse {
        position: usize,
        line: usize,
        column: usize,
        fragment: String,
    },

    /// The input was incomplete
    #[error("Message parsing failed because of incomplete input.{}", .0.map(|s| format!(" Need at least {s} more characters to continue.")).unwrap_or_default())]
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
                    fragment: e.input.fragment().chars().take(7).collect(),
                }
            }
        }
    }
}

/// Parse a subcomponent using the default separators.
pub fn parse_subcomponent(input: &str) -> Result<Subcomponent<'_>, ParseError> {
    let separators = Separators::default();
    subcomponent::subcomponent(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a subcomponent using the given separators.
pub fn parse_subcomponent_with_separators(
    input: &str,
    separators: Separators,
) -> Result<Subcomponent<'_>, ParseError> {
    subcomponent::subcomponent(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a component using the default separators.
pub fn parse_component(input: &str) -> Result<Component<'_>, ParseError> {
    let separators = Separators::default();
    component::component(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a component using the given separators.
pub fn parse_component_with_separators(
    input: &str,
    separators: Separators,
) -> Result<Component<'_>, ParseError> {
    component::component(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a repeat using the default separators.
pub fn parse_repeat(input: &str) -> Result<Repeat<'_>, ParseError> {
    let separators = Separators::default();
    repeat::repeat(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a repeat using the given separators.
pub fn parse_repeat_with_separators(
    input: &str,
    separators: Separators,
) -> Result<Repeat<'_>, ParseError> {
    repeat::repeat(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a field using the default separators.
pub fn parse_field(input: &str) -> Result<Field<'_>, ParseError> {
    let separators = Separators::default();
    field::field(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a field using the given separators.
pub fn parse_field_with_separators(
    input: &str,
    separators: Separators,
) -> Result<Field<'_>, ParseError> {
    field::field(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a segment using the default separators.
pub fn parse_segment(input: &str) -> Result<Segment<'_>, ParseError> {
    let separators = Separators::default();
    segment::segment(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a segment using the given separators.
pub fn parse_segment_with_separators(
    input: &str,
    separators: Separators,
) -> Result<Segment<'_>, ParseError> {
    segment::segment(separators)(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

/// Parse a MSH segment and return the separators and the segment.
pub fn parse_msh(input: &str) -> Result<(Separators, Segment<'_>), ParseError> {
    msh::msh()(Span::new(input))
        .map(|(_, m)| (m.separators, m.into()))
        .map_err(|e| e.into())
}

/// Parse a complete HL7 message.
pub fn parse_message(input: &str) -> Result<crate::Message<'_>, ParseError> {
    crate::parser::message::message()(Span::new(input))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}
