//! # Querying HL7 messages
//!
//! This module provides utilities for querying HL7 messages. This is useful for extracting
//! specific values from an HL7 message, such as the patient's name or the ordering provider's
//! name without having to manually traverse the message.
//!
//! ## Location queries
//!
//! A location query is a string that describes the location of a value within an HL7 message.
//! The query is made up of the segment name, field index, repeat index, component index, and
//! subcomponent index. Each part of the query is separated by a period (`.`), and each index is
//! enclosed in square brackets (`[]`). For example, the query `PID.5[1].1` would refer to the
//! first subcomponent of the first component of the first repeat of the fifth field of the PID
//! segment.
//!
//! ## Examples
//!
//! ```
//! use hl7_parser::query::LocationQuery;
//! let query = LocationQuery::parse("MSH[1].2[3].4.5").unwrap();
//! assert_eq!(query.segment, "MSH");
//! assert_eq!(query.segment_index, Some(1));
//! assert_eq!(query.field, Some(2));
//! assert_eq!(query.repeat, Some(3));
//! assert_eq!(query.component, Some(4));
//! assert_eq!(query.subcomponent, Some(5));
//! ```
//!
//! ```
//! use hl7_parser::query::LocationQuery;
//! let query = LocationQuery::parse("MSH.2.4").unwrap();
//! assert_eq!(query.segment, "MSH");
//! assert_eq!(query.segment_index, None);
//! assert_eq!(query.field, Some(2));
//! assert_eq!(query.repeat, None);
//! assert_eq!(query.component, Some(4));
//! assert_eq!(query.subcomponent, None);
//! ```
//!
//! ## Building location queries
//!
//! A location query can also be built using a builder pattern. This is useful when you want to
//! ensure that the query is valid at before using it.
//! ```
//! use hl7_parser::query::LocationQueryBuilder;
//! let query = LocationQueryBuilder::new()
//!    .segment("MSH")
//!    .segment_index(1)
//!    .field(2)
//!    .repeat(3)
//!    .component(4)
//!    .subcomponent(5)
//!    .build()
//!    .unwrap();
//! assert_eq!(query.to_string(), "MSH[1].2[3].4.5");
//! ```

mod parser;

use std::{fmt::Display, str::FromStr};

pub use parser::QueryParseError;
use thiserror::Error;

use crate::{
    message::{Component, Field, Repeat, Segment, Separators, Subcomponent},
    parser::Span,
};

/// A location query that describes the location of a value within an HL7 message.
/// The query is made up of the segment name, field index, repeat index, component index, and
/// subcomponent index. Each part of the query is separated by a period (`.`), and each index is
/// enclosed in square brackets (`[]`). For example, the query `PID.5[1].1` would refer to the
/// first subcomponent of the first component of the first repeat of the fifth field of the PID
/// segment.
///
/// All indexes are 1-based.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocationQuery {
    pub segment: String,
    pub segment_index: Option<usize>,
    pub field: Option<usize>,
    pub repeat: Option<usize>,
    pub component: Option<usize>,
    pub subcomponent: Option<usize>,
}

/// Parse a location query from a string
///
/// # Examples
/// ```
/// use hl7_parser::query::parse_location_query;
/// let query = parse_location_query("MSH[1].2[3].4.5").unwrap();
/// assert_eq!(query.segment, "MSH");
/// assert_eq!(query.segment_index, Some(1));
/// assert_eq!(query.field, Some(2));
/// assert_eq!(query.repeat, Some(3));
/// assert_eq!(query.component, Some(4));
/// assert_eq!(query.subcomponent, Some(5));
/// ```
///
/// ```
/// use hl7_parser::query::parse_location_query;
/// let query = parse_location_query("MSH.2.4").unwrap();
/// assert_eq!(query.segment, "MSH");
/// assert_eq!(query.segment_index, None);
/// assert_eq!(query.field, Some(2));
/// assert_eq!(query.repeat, None);
/// assert_eq!(query.component, Some(4));
/// assert_eq!(query.subcomponent, None);
/// ```
pub fn parse_location_query(query: &str) -> Result<LocationQuery, QueryParseError> {
    parser::parse_query(Span::new(query))
        .map(|(_, m)| m)
        .map_err(|e| e.into())
}

impl FromStr for LocationQuery {
    type Err = QueryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_location_query(s)
    }
}

impl TryFrom<&str> for LocationQuery {
    type Error = QueryParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_location_query(value)
    }
}

impl TryFrom<String> for LocationQuery {
    type Error = QueryParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_location_query(&value)
    }
}

impl TryFrom<&String> for LocationQuery {
    type Error = QueryParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        parse_location_query(value)
    }
}

impl Display for LocationQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segment)?;
        if let Some(i) = self.segment_index {
            write!(f, "[{}]", i)?;
        }
        if let Some(i) = self.field {
            write!(f, ".{}", i)?;
        } else {
            return Ok(());
        }
        if let Some(i) = self.repeat {
            write!(f, "[{}]", i)?;
        }
        if let Some(i) = self.component {
            write!(f, ".{}", i)?;
        } else {
            return Ok(());
        }
        if let Some(i) = self.subcomponent {
            write!(f, ".{}", i)?;
        }
        Ok(())
    }
}

impl LocationQuery {
    /// Parse a location query from a string. Equivalent to `parse_location_query`.
    pub fn parse(query: &str) -> Result<Self, QueryParseError> {
        parse_location_query(query)
    }
}

/// A builder for creating a location query with error checking.
#[derive(Debug, Clone)]
pub struct LocationQueryBuilder {
    segment: Option<String>,
    segment_index: Option<usize>,
    field: Option<usize>,
    repeat: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
}

/// Errors that can occur when building a location query
#[derive(Debug, Clone, Error)]
pub enum LocationQueryBuildError {
    /// The segment is missing
    #[error("Missing segment: segment is required")]
    MissingSegment,
    /// The segment is not 3 characters long
    #[error("Invalid segment length: segments must be 3 characters long")]
    InvalidSegmentLength,
    /// The segment contains non-ASCII uppercase characters
    #[error("Invalid segment name: segments must be ASCII uppercase")]
    InvalidSegmentName,
    /// The segment index is 0
    #[error("Invalid segment index: segment index must be greater than 0")]
    InvalidSegmentIndex,
    /// The field index is 0
    #[error("Invalid field index: field index must be greater than 0")]
    InvalidFieldIndex,
    /// The repeat index is 0
    #[error("Invalid repeat index: repeat index must be greater than 0")]
    InvalidRepeatIndex,
    /// The component index is 0
    #[error("Invalid component index: component index must be greater than 0")]
    InvalidComponentIndex,
    /// The subcomponent index is 0
    #[error("Invalid subcomponent index: subcomponent index must be greater than 0")]
    InvalidSubcomponentIndex,
}

impl Default for LocationQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LocationQueryBuilder {
    pub fn new() -> Self {
        Self {
            segment: None,
            segment_index: None,
            field: None,
            repeat: None,
            component: None,
            subcomponent: None,
        }
    }

    /// Create a new location query builder. The segment must be 3 characters long and ASCII
    /// uppercase.
    pub fn segment<S: ToString>(mut self, segment: S) -> Self {
        self.segment = Some(segment.to_string());
        self
    }

    /// Set the segment index. This is optional. If not set, the segment index will not be included
    /// in the query. If set, the segment index must be greater than 0.
    pub fn segment_index(mut self, index: usize) -> Self {
        self.segment_index = Some(index);
        self
    }

    /// Set the field index. This is optional. If not set, the field index will not be included in
    /// the query. If set, the field index must be greater than 0.
    pub fn field(mut self, index: usize) -> Self {
        self.field = Some(index);
        self
    }

    /// Set the repeat index. This is optional. If not set, the repeat index will not be included
    /// in the query. If set, the repeat index must be greater than 0.
    pub fn repeat(mut self, index: usize) -> Self {
        self.repeat = Some(index);
        self
    }

    /// Set the component index. This is optional. If not set, the component index will not be
    /// included in the query. If set, the component index must be greater than 0.
    pub fn component(mut self, index: usize) -> Self {
        self.component = Some(index);
        self
    }

    /// Set the subcomponent index. This is optional. If not set, the subcomponent index will not
    /// be included in the query. If set, the subcomponent index must be greater than 0.
    pub fn subcomponent(mut self, index: usize) -> Self {
        self.subcomponent = Some(index);
        self
    }

    /// Build the location query
    pub fn build(self) -> Result<LocationQuery, LocationQueryBuildError> {
        let segment = if let Some(segment) = self.segment {
            if segment.len() != 3 {
                return Err(LocationQueryBuildError::InvalidSegmentLength);
            }
            if !segment.chars().all(|c| c.is_ascii_uppercase()) {
                return Err(LocationQueryBuildError::InvalidSegmentName);
            }
            segment
        }
        else {
            return Err(LocationQueryBuildError::MissingSegment);
        };

        let segment_index = if let Some(segment_index) = self.segment_index {
            if segment_index == 0 {
                return Err(LocationQueryBuildError::InvalidSegmentIndex);
            }
            Some(segment_index)
        } else {
            None
        };

        let field = if let Some(field) = self.field {
            if field == 0 {
                return Err(LocationQueryBuildError::InvalidFieldIndex);
            }
            Some(field)
        } else {
            None
        };

        let repeat = if let Some(repeat) = self.repeat {
            if repeat == 0 {
                return Err(LocationQueryBuildError::InvalidRepeatIndex);
            }
            Some(repeat)
        } else {
            None
        };

        let component = if let Some(component) = self.component {
            if component == 0 {
                return Err(LocationQueryBuildError::InvalidComponentIndex);
            }
            Some(component)
        } else {
            None
        };

        let subcomponent = if let Some(subcomponent) = self.subcomponent {
            if subcomponent == 0 {
                return Err(LocationQueryBuildError::InvalidSubcomponentIndex);
            }
            Some(subcomponent)
        } else {
            None
        };

        Ok(LocationQuery {
            segment,
            segment_index,
            field,
            repeat,
            component,
            subcomponent,
        })
    }
}

/// A result of a location query. This can be a segment, field, repeat, component, or subcomponent.
/// The result contains a reference to the corresponding part of the message.
/// The result can be used to get the raw value of the part of the message, or to display the value
/// using the separators to decode escape sequences.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocationQueryResult<'m> {
    Segment(&'m Segment<'m>),
    Field(&'m Field<'m>),
    Repeat(&'m Repeat<'m>),
    Component(&'m Component<'m>),
    Subcomponent(&'m Subcomponent<'m>),
}

impl<'m> LocationQueryResult<'m> {
    /// Get the raw value of the result. This is the value as it appears in the message,
    /// without any decoding of escape sequences.
    /// Segments will be separated by the segment separator character.
    /// Fields will be separated by the field separator character.
    /// Repeats will be separated by the repeat separator character.
    /// Components will be separated by the component separator character.
    /// Subcomponents will be separated by the subcomponent separator character.
    pub fn raw_value(&self) -> &'m str {
        match self {
            LocationQueryResult::Segment(seg) => seg.raw_value(),
            LocationQueryResult::Field(field) => field.raw_value(),
            LocationQueryResult::Repeat(repeat) => repeat.raw_value(),
            LocationQueryResult::Component(component) => component.raw_value(),
            LocationQueryResult::Subcomponent(subcomponent) => subcomponent.raw_value(),
        }
    }

    /// Display the result, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", result.display(separators))`.
    pub fn display(&self, separators: &'m Separators) -> LocationQueryResultDisplay<'m> {
        LocationQueryResultDisplay {
            value: self.raw_value(),
            separators,
        }
    }
}

/// Display the result of a location query, using the separators to decode escape sequences
pub struct LocationQueryResultDisplay<'m> {
    value: &'m str,
    separators: &'m Separators,
}

impl<'m> Display for LocationQueryResultDisplay<'m> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "{}", self.separators.decode(self.value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_display_location_query() {
        let query = LocationQuery {
            segment: "MSH".to_string(),
            segment_index: Some(1),
            field: Some(2),
            repeat: Some(3),
            component: Some(4),
            subcomponent: Some(5),
        };
        assert_eq!(query.to_string(), "MSH[1].2[3].4.5");

        let query = LocationQuery {
            segment: "MSH".to_string(),
            segment_index: None,
            field: Some(2),
            repeat: None,
            component: Some(4),
            subcomponent: None,
        };
        assert_eq!(query.to_string(), "MSH.2.4");

        let query = LocationQuery {
            segment: "MSH".to_string(),
            segment_index: None,
            field: None,
            repeat: None,
            component: Some(4),
            subcomponent: Some(5),
        };
        assert_eq!(query.to_string(), "MSH");
    }
}
