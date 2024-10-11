use super::LocationQuery;
use crate::parser::Span;
use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::one_of,
    combinator::opt,
    sequence::{delimited, preceded},
    IResult,
};
use thiserror::Error;

/// An error that can occur when parsing a query
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QueryParseError {
    /// The parsing failed for some reason
    #[error("Query parsing failed at position {position}: `{fragment}`")]
    FailedToParse {
        position: usize,
        fragment: String,
    },

    /// The input was incomplete
    #[error("Query parsing failed because of incomplete input.{}", .0.map(|s| format!(" Need at least {s} more characters to continue.")).unwrap_or_default())]
    IncompleteInput(Option<usize>),
}

impl<'s> From<nom::Err<nom::error::Error<Span<'s>>>> for QueryParseError {
    fn from(e: nom::Err<nom::error::Error<Span<'s>>>) -> Self {
        match e {
            nom::Err::Incomplete(nom::Needed::Unknown) => QueryParseError::IncompleteInput(None),
            nom::Err::Incomplete(nom::Needed::Size(size)) => {
                QueryParseError::IncompleteInput(Some(size.get()))
            }
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let position = e.input.offset;
                QueryParseError::FailedToParse {
                    position,
                    fragment: e.input.input.chars().take(7).collect(),
                }
            }
        }
    }
}

fn nonzero_integer(s: Span) -> IResult<Span, usize> {
    let (_s, val) = take_while1(|c: char| c.is_ascii_digit())(s)?;
    let val = val.input.parse::<usize>().map_err(|_| todo!())?;
    if val == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Digit,
        )));
    }
    Ok((_s, val))
}

fn nonzero_array_access(s: Span) -> IResult<Span, usize> {
    delimited(tag("["), nonzero_integer, tag("]"))(s)
}

fn preceeded_nonzero_integer(s: Span) -> IResult<Span, usize> {
    preceded(one_of(".- "), nonzero_integer)(s)
}

pub fn parse_query(i: Span) -> IResult<Span, LocationQuery> {
    let (i, segment) = crate::parser::segment::parse_segment_name(i)?;
    let (i, segment_index) = opt(nonzero_array_access)(i)?;
    let (i, field) = opt(preceeded_nonzero_integer)(i)?;
    let (i, repeat) = if field.is_some() {
        opt(nonzero_array_access)(i)?
    } else {
        (i, None)
    };
    let (i, component) = if field.is_some() {
        opt(preceeded_nonzero_integer)(i)?
    } else {
        (i, None)
    };
    let (i, subcomponent) = if component.is_some() {
        opt(preceeded_nonzero_integer)(i)?
    } else {
        (i, None)
    };

    let segment = segment.input.to_string();
    Ok((
        i,
        LocationQuery {
            segment,
            segment_index,
            field,
            repeat,
            component,
            subcomponent,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_preceeded_nonzero_integer() {
        let input = Span::new(".123");
        let actual = preceeded_nonzero_integer(input).unwrap().1;
        assert_eq!(actual, 123);

        let input = Span::new(" 123");
        let actual = preceeded_nonzero_integer(input).unwrap().1;
        assert_eq!(actual, 123);

        let input = Span::new("-123");
        let actual = preceeded_nonzero_integer(input).unwrap().1;
        assert_eq!(actual, 123);

        let input = Span::new("123");
        assert!(preceeded_nonzero_integer(input).is_err());

        let input = Span::new(".abc");
        assert!(preceeded_nonzero_integer(input).is_err());
    }

    #[test]
    fn can_parse_array_access() {
        let input = Span::new("[123]");
        let actual = nonzero_array_access(input).unwrap().1;
        assert_eq!(actual, 123);

        let input = Span::new("[0]");
        assert!(nonzero_array_access(input).is_err());

        let input = Span::new("[-10]");
        assert!(nonzero_array_access(input).is_err());

        let input = Span::new("[abc]");
        assert!(nonzero_array_access(input).is_err());
    }

    #[test]
    fn can_parse_full_query() {
        let input = Span::new("MSH[1].2[3].4.5");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, Some(1));
        assert_eq!(actual.field, Some(2));
        assert_eq!(actual.repeat, Some(3));
        assert_eq!(actual.component, Some(4));
        assert_eq!(actual.subcomponent, Some(5));
    }

    #[test]
    fn can_parse_truncated_queries() {
        let input = Span::new("MSH[1].2[3].4");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, Some(1));
        assert_eq!(actual.field, Some(2));
        assert_eq!(actual.repeat, Some(3));
        assert_eq!(actual.component, Some(4));
        assert_eq!(actual.subcomponent, None);

        let input = Span::new("MSH[1].2[3]");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, Some(1));
        assert_eq!(actual.field, Some(2));
        assert_eq!(actual.repeat, Some(3));
        assert_eq!(actual.component, None);
        assert_eq!(actual.subcomponent, None);

        let input = Span::new("MSH[1].2");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, Some(1));
        assert_eq!(actual.field, Some(2));
        assert_eq!(actual.repeat, None);
        assert_eq!(actual.component, None);
        assert_eq!(actual.subcomponent, None);

        let input = Span::new("MSH[1]");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, Some(1));
        assert_eq!(actual.field, None);
        assert_eq!(actual.repeat, None);
        assert_eq!(actual.component, None);
        assert_eq!(actual.subcomponent, None);

        let input = Span::new("MSH");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "MSH");
        assert_eq!(actual.segment_index, None);
        assert_eq!(actual.field, None);
        assert_eq!(actual.repeat, None);
        assert_eq!(actual.component, None);
        assert_eq!(actual.subcomponent, None);

        let input = Span::new("PID.3");
        let actual = parse_query(input).unwrap().1;
        assert_eq!(actual.segment, "PID");
        assert_eq!(actual.segment_index, None);
        assert_eq!(actual.field, Some(3));
        assert_eq!(actual.repeat, None);
        assert_eq!(actual.component, None);
        assert_eq!(actual.subcomponent, None);
    }
}
