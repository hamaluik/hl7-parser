mod parser;

use std::{fmt::Display, str::FromStr};

pub use parser::QueryParseError;
use thiserror::Error;

use crate::{
    message::{Component, Field, Repeat, Segment, Separators, Subcomponent},
    parser::Span,
};

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
    segment: String,
    segment_index: Option<usize>,
    field: Option<usize>,
    repeat: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
}

/// Errors that can occur when building a location query
#[derive(Debug, Clone, Error)]
pub enum LocationQueryBuildError {
    #[error("Invalid segment length: segments must be 3 characters long")]
    InvalidSegmentLength,
    #[error("Invalid segment name: segments must be ASCII uppercase")]
    InvalidSegmentName,
    #[error("Invalid segment index: segment index must be greater than 0")]
    InvalidSegmentIndex,
    #[error("Invalid field index: field index must be greater than 0")]
    InvalidFieldIndex,
    #[error("Invalid repeat index: repeat index must be greater than 0")]
    InvalidRepeatIndex,
    #[error("Invalid component index: component index must be greater than 0")]
    InvalidComponentIndex,
    #[error("Invalid subcomponent index: subcomponent index must be greater than 0")]
    InvalidSubcomponentIndex,
}

impl LocationQueryBuilder {
    /// Create a new location query builder. The segment must be 3 characters long and ASCII
    /// uppercase.
    pub fn new(segment: &str) -> Result<Self, LocationQueryBuildError> {
        if segment.len() != 3 {
            return Err(LocationQueryBuildError::InvalidSegmentLength);
        }
        if !segment.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(LocationQueryBuildError::InvalidSegmentName);
        }
        Ok(Self {
            segment: segment.to_string(),
            segment_index: None,
            field: None,
            repeat: None,
            component: None,
            subcomponent: None,
        })
    }

    /// Set the segment index. This is optional. If not set, the segment index will not be included
    /// in the query. If set, the segment index must be greater than 0.
    pub fn segment_index(mut self, index: usize) -> Result<Self, LocationQueryBuildError> {
        if index == 0 {
            return Err(LocationQueryBuildError::InvalidSegmentIndex);
        }
        self.segment_index = Some(index);
        Ok(self)
    }

    /// Set the field index. This is optional. If not set, the field index will not be included in
    /// the query. If set, the field index must be greater than 0.
    pub fn field(mut self, index: usize) -> Result<Self, LocationQueryBuildError> {
        if index == 0 {
            return Err(LocationQueryBuildError::InvalidFieldIndex);
        }
        self.field = Some(index);
        Ok(self)
    }

    /// Set the repeat index. This is optional. If not set, the repeat index will not be included
    /// in the query. If set, the repeat index must be greater than 0.
    pub fn repeat(mut self, index: usize) -> Result<Self, LocationQueryBuildError> {
        if index == 0 {
            return Err(LocationQueryBuildError::InvalidRepeatIndex);
        }
        self.repeat = Some(index);
        Ok(self)
    }

    /// Set the component index. This is optional. If not set, the component index will not be
    /// included in the query. If set, the component index must be greater than 0.
    pub fn component(mut self, index: usize) -> Result<Self, LocationQueryBuildError> {
        if index == 0 {
            return Err(LocationQueryBuildError::InvalidComponentIndex);
        }
        self.component = Some(index);
        Ok(self)
    }

    /// Set the subcomponent index. This is optional. If not set, the subcomponent index will not
    /// be included in the query. If set, the subcomponent index must be greater than 0.
    pub fn subcomponent(mut self, index: usize) -> Result<Self, LocationQueryBuildError> {
        if index == 0 {
            return Err(LocationQueryBuildError::InvalidSubcomponentIndex);
        }
        self.subcomponent = Some(index);
        Ok(self)
    }

    /// Build the location query
    pub fn build(self) -> LocationQuery {
        LocationQuery {
            segment: self.segment,
            segment_index: self.segment_index,
            field: self.field,
            repeat: self.repeat,
            component: self.component,
            subcomponent: self.subcomponent,
        }
    }
}

impl From<LocationQueryBuilder> for LocationQuery {
    fn from(builder: LocationQueryBuilder) -> Self {
        builder.build()
    }
}

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

    #[test]
    fn can_build_query() {
        let query = LocationQueryBuilder::new("MSH")
            .unwrap()
            .segment_index(1)
            .unwrap()
            .field(2)
            .unwrap()
            .repeat(3)
            .unwrap()
            .component(4)
            .unwrap()
            .subcomponent(5)
            .unwrap()
            .build();
        assert_eq!(query.to_string(), "MSH[1].2[3].4.5");

        let query = LocationQueryBuilder::new("MSH")
            .unwrap()
            .field(2)
            .unwrap()
            .component(4)
            .unwrap()
            .build();
        assert_eq!(query.to_string(), "MSH.2.4");

        let query = LocationQueryBuilder::new("MSH").unwrap().build();
        assert_eq!(query.to_string(), "MSH");
    }
}
