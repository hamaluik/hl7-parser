use crate::{LocatedData, VResult};
use nom::{
    bytes::complete::{tag, take_while1, take_while_m_n},
    character::complete::one_of,
    combinator::opt,
    error::{VerboseError, VerboseErrorKind},
    multi::many_m_n,
    sequence::{delimited, preceded},
    Finish,
};
use std::num::NonZeroUsize;
use std::str::FromStr;

/// A query for a particular piece of a message, to be used in
/// [ParsedMessage::query](crate::ParsedMessage::query) (or [ParsedMessageOwned::query](crate::ParsedMessageOwned::query))
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationQuery {
    /// The segment identifier to query
    pub segment: String,
    /// The 1-based field ID to optionally query
    pub field: Option<NonZeroUsize>,
    /// The 1-based repeat ID to optionally query
    pub repeat: Option<NonZeroUsize>,
    /// The 1-based component ID to optionally query
    pub component: Option<NonZeroUsize>,
    /// The 1-based sub-component ID to optionally query
    pub sub_component: Option<NonZeroUsize>,
}

fn is_valid_seg_id(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn parse_segment_id(s: &str) -> VResult<&str, &str> {
    take_while_m_n(3, 3, is_valid_seg_id)(s)
}

fn is_digit_base_10(c: char) -> bool {
    c.is_ascii_digit()
}

fn preceeded_nonzero_integer(s: &str) -> VResult<&str, NonZeroUsize> {
    let (_s, val) = preceded(one_of(".- "), take_while1(is_digit_base_10))(s)?;
    let val = val.parse::<usize>().map_err(|_| {
        nom::Err::Failure(VerboseError {
            errors: vec![(
                s,
                VerboseErrorKind::Context("Failed to parse value as an integer"),
            )],
        })
    })?;
    let val = NonZeroUsize::new(val).ok_or_else(|| {
        nom::Err::Failure(VerboseError {
            errors: vec![(s, VerboseErrorKind::Context("Integer was 0"))],
        })
    })?;

    Ok((_s, val))
}

fn nonzero_integer(s: &str) -> VResult<&str, NonZeroUsize> {
    let (_s, val) = take_while1(is_digit_base_10)(s)?;
    let val = val.parse::<usize>().map_err(|_| {
        nom::Err::Failure(VerboseError {
            errors: vec![(
                s,
                VerboseErrorKind::Context("Failed to parse value as an integer"),
            )],
        })
    })?;
    let val = NonZeroUsize::new(val).ok_or_else(|| {
        nom::Err::Failure(VerboseError {
            errors: vec![(s, VerboseErrorKind::Context("Integer was 0"))],
        })
    })?;

    Ok((_s, val))
}

fn nonzero_array_access(s: &str) -> VResult<&str, NonZeroUsize> {
    delimited(tag("["), nonzero_integer, tag("]"))(s)
}

fn parse_query(s: &str) -> VResult<&str, LocationQuery> {
    let (s, segment) = parse_segment_id(s)?;
    let (s, field) = opt(preceeded_nonzero_integer)(s)?;
    let (s, repeat) = many_m_n(0, 1, nonzero_array_access)(s)?;
    let repeat = Some(
        repeat
            .into_iter()
            .next()
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap()),
    );
    let (s, component) = opt(preceeded_nonzero_integer)(s)?;
    let (s, sub_component) = opt(preceeded_nonzero_integer)(s)?;

    let segment = segment.to_uppercase();

    Ok((
        s,
        LocationQuery {
            segment,
            field,
            repeat,
            component,
            sub_component,
        },
    ))
}

impl FromStr for LocationQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_query(s).finish() {
            Ok((_leftovers, query)) => Ok(query),
            Err(err) => Err(nom::error::convert_error(s, err)),
        }
    }
}

impl LocationQuery {
    /// Create a new location query by attempting to parse a string query
    ///
    /// Query is expected to be in the form of: `<SEGMENT ID>[<SEP><FIELD>][\[<REPEAT>\]][<SEP><COMPONENT>][<SEP><SUB-COMPONENT>]`
    /// where:
    /// * `<SEGMENT ID>` is 3 alpha-numeric characters which will be normalized to uppercase
    /// * `<SEP>` is one of `.`, `-`, or ` ` (space)
    /// * `<FIELD>` is a non-zero integer specifying the field number
    /// * `<REPEAT>` is a non-zero integer specifying the repeat number
    /// * `<COMPONENT>` is a non-zero integer specifying the component number
    /// * `<SUB-COMPONENT>` is a non-zero integer specifying the sub-component number
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::LocationQuery;
    /// let query = LocationQuery::new("MSH.9").expect("can parse query");
    /// assert_eq!(query.segment.as_str(), "MSH");
    /// assert_eq!(query.field.unwrap().get(), 9);
    /// assert!(query.component.is_none());
    /// ```
    ///
    /// ```
    /// # use hl7_parser::LocationQuery;
    /// let query = LocationQuery::new("AL1.5[3]").expect("can parse query");
    /// assert_eq!(query.segment.as_str(), "AL1");
    /// assert_eq!(query.field.unwrap().get(), 5);
    /// assert_eq!(query.repeat.unwrap().get(), 3);
    /// assert!(query.component.is_none());
    /// ```
    ///
    /// ```
    /// # use hl7_parser::LocationQuery;
    /// let query = LocationQuery::new("PID-3-4-2").expect("can parse query");
    /// assert_eq!(query.segment.as_str(), "PID");
    /// assert_eq!(query.field.unwrap().get(), 3);
    /// assert_eq!(query.component.unwrap().get(), 4);
    /// assert_eq!(query.sub_component.unwrap().get(), 2);
    /// ```
    pub fn new<S: AsRef<str>>(source: S) -> Result<LocationQuery, String> {
        FromStr::from_str(source.as_ref())
    }

    /// Create a location query solely for a segment
    pub fn new_segment<S: ToString>(segment: S) -> LocationQuery {
        let segment = segment.to_string();
        LocationQuery {
            segment,
            field: None,
            repeat: None,
            component: None,
            sub_component: None,
        }
    }

    /// Create a location query for a segment and a field
    pub fn new_field<S, U, UErr>(segment: S, field: U) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: None,
            component: None,
            sub_component: None,
        })
    }

    /// Create a location query for a segment, a field, and a repeat
    pub fn new_field_repeat<S, U, UErr>(
        segment: S,
        field: U,
        repeat: U,
    ) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        let repeat: NonZeroUsize = repeat.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: Some(repeat),
            component: None,
            sub_component: None,
        })
    }

    /// Create a location query for a segment, a field, and a component
    pub fn new_component<S, U, UErr>(
        segment: S,
        field: U,
        component: U,
    ) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        let component: NonZeroUsize = component.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: None,
            component: Some(component),
            sub_component: None,
        })
    }

    /// Create a location query for a segment, a field, a repeat, and a component
    pub fn new_repeat_component<S, U, UErr>(
        segment: S,
        field: U,
        repeat: U,
        component: U,
    ) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        let repeat: NonZeroUsize = repeat.try_into()?;
        let component: NonZeroUsize = component.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: Some(repeat),
            component: Some(component),
            sub_component: None,
        })
    }

    /// Create a location query for a segment, a field, a component, and a sub-component
    pub fn new_sub_component<S, U, UErr>(
        segment: S,
        field: U,
        component: U,
        sub_component: U,
    ) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        let component: NonZeroUsize = component.try_into()?;
        let sub_component: NonZeroUsize = sub_component.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: None,
            component: Some(component),
            sub_component: Some(sub_component),
        })
    }

    /// Create a location query for a segment, a field, a component, and a sub-component
    pub fn new_repeat_sub_component<S, U, UErr>(
        segment: S,
        field: U,
        repeat: U,
        component: U,
        sub_component: U,
    ) -> Result<LocationQuery, UErr>
    where
        S: ToString,
        U: TryInto<NonZeroUsize, Error = UErr>,
    {
        let segment = segment.to_string();
        let field: NonZeroUsize = field.try_into()?;
        let repeat: NonZeroUsize = repeat.try_into()?;
        let component: NonZeroUsize = component.try_into()?;
        let sub_component: NonZeroUsize = sub_component.try_into()?;
        Ok(LocationQuery {
            segment,
            field: Some(field),
            repeat: Some(repeat),
            component: Some(component),
            sub_component: Some(sub_component),
        })
    }
}

impl<'s> TryFrom<&'s str> for LocationQuery {
    type Error = String;

    fn try_from(value: &'s str) -> Result<Self, Self::Error> {
        FromStr::from_str(value)
    }
}

impl TryFrom<String> for LocationQuery {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        FromStr::from_str(value.as_str())
    }
}

impl<'l> From<LocatedData<'l>> for LocationQuery {
    fn from(value: LocatedData) -> Self {
        let LocatedData {
            segment,
            field,
            repeat,
            component,
            sub_component,
        } = value;
        LocationQuery {
            segment: segment.map(|(seg, _, _)| seg).unwrap_or("MSH").to_string(),
            field: field.map(|(f, _)| f),
            repeat: repeat.map(|(r, _)| r),
            component: component.map(|(c, _)| c),
            sub_component: sub_component.map(|(s, _)| s),
        }
    }
}

impl<'l> From<&LocatedData<'l>> for LocationQuery {
    fn from(value: &LocatedData) -> Self {
        let LocatedData {
            segment,
            field,
            repeat,
            component,
            sub_component,
        } = value;
        LocationQuery {
            segment: segment.map(|(seg, _, _)| seg).unwrap_or("MSH").to_string(),
            field: field.map(|(f, _)| f),
            repeat: repeat.map(|(r, _)| r),
            component: component.map(|(c, _)| c),
            sub_component: sub_component.map(|(s, _)| s),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ParsedMessage;

    use super::*;

    #[test]
    fn can_parse_valid_queries() {
        let query = LocationQuery::from_str("MSH.1.2.3").expect("can parse query");
        assert_eq!(query.segment.as_str(), "MSH");
        assert_eq!(query.field.unwrap().get(), 1);
        assert_eq!(query.component.unwrap().get(), 2);
        assert_eq!(query.sub_component.unwrap().get(), 3);

        let query = LocationQuery::from_str("MSH-1-2").expect("can parse query");
        assert_eq!(query.segment.as_str(), "MSH");
        assert_eq!(query.field.unwrap().get(), 1);
        assert_eq!(query.component.unwrap().get(), 2);
        assert!(query.sub_component.is_none());

        let query = LocationQuery::from_str("MSH 1-2.3").expect("can parse query");
        assert_eq!(query.segment.as_str(), "MSH");
        assert_eq!(query.field.unwrap().get(), 1);
        assert_eq!(query.component.unwrap().get(), 2);
        assert_eq!(query.sub_component.unwrap().get(), 3);

        let query = LocationQuery::from_str("PV1").expect("can parse query");
        assert_eq!(query.segment.as_str(), "PV1");
        assert!(query.field.is_none());
        assert!(query.component.is_none());
        assert!(query.sub_component.is_none());

        let query = LocationQuery::from_str("pid").expect("can parse query");
        assert_eq!(query.segment.as_str(), "PID");
        assert!(query.field.is_none());
        assert!(query.component.is_none());
        assert!(query.sub_component.is_none());

        let query = LocationQuery::from_str("MSH.1[2]").expect("can parse query");
        assert_eq!(query.segment.as_str(), "MSH");
        assert_eq!(query.field.unwrap().get(), 1);
        assert_eq!(query.repeat.unwrap().get(), 2);
        assert!(query.component.is_none());
        assert!(query.sub_component.is_none());
    }

    #[test]
    fn is_forgiving_of_extra_query_items() {
        let query = LocationQuery::from_str("PV1.").expect("can parse query");
        assert_eq!(query.segment.as_str(), "PV1");
        assert!(query.field.is_none());
        assert!(query.component.is_none());
        assert!(query.sub_component.is_none());

        let query = LocationQuery::from_str("MSH.1.2.3.4.5").expect("can parse query");
        assert_eq!(query.segment.as_str(), "MSH");
        assert_eq!(query.field.unwrap().get(), 1);
        assert_eq!(query.component.unwrap().get(), 2);
        assert_eq!(query.sub_component.unwrap().get(), 3);

        assert!(LocationQuery::from_str("MSH123")
            .expect("can parse")
            .field
            .is_none());
        assert!(
            LocationQuery::from_str("Peter piper picked some pickled peppers")
                .expect("can parse")
                .field
                .is_none()
        );
        assert!(LocationQuery::from_str("MSH-1-a")
            .expect("can parse")
            .component
            .is_none());
    }

    #[test]
    fn cant_parse_invalid_segment_ids() {
        assert!(LocationQuery::from_str("4@4").is_err());
    }

    #[test]
    fn cant_parse_malformed_queries() {
        assert!(LocationQuery::from_str("😁SH123").is_err());
        assert!(LocationQuery::from_str("píd").is_err());
    }

    #[test]
    fn can_parse_located_data_display() {
        let message = include_str!("../test_assets/sample_adt_a04.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = ParsedMessage::parse(message.as_str()).expect("can parse message");
        let location = message.locate_cursor(0x1cc);
        let query_direct = LocationQuery::from(&location);
        let location = location.to_string();
        let query_result = LocationQuery::new(location).expect("can parse location");
        assert_eq!(query_direct, query_result);
    }
}
