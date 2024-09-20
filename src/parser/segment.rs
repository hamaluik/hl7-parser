use std::borrow::Cow;

use super::field::field;
use crate::message::{Segment, Separators};
use nom::{
    bytes::complete::take_while_m_n, character::complete::char, multi::separated_list0,
    sequence::terminated, IResult,
};

pub fn segment<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Segment<'i>> {
    move |i| parse_segment(i, seps)
}

fn segment_name<'i>() -> impl FnMut(&'i str) -> IResult<&'i str, &'i str> {
    move |i| parse_segment_name(i)
}

fn parse_segment_name<'i>(i: &'i str) -> IResult<&'i str, &'i str> {
    take_while_m_n(3, 3, |c: char| c.is_alphanumeric())(i)
}

fn parse_segment<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Segment<'i>> {
    let (i, name) = terminated(segment_name(), char(seps.field))(i)?;
    let (i, v) = separated_list0(char(seps.field), field(seps))(i)?;

    Ok((
        i,
        Segment {
            name: Cow::Borrowed(name),
            fields: v,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use crate::message::{Field, Separators, Segment};
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_segment() {
        let separators = Separators::default();

        let input = "MSH|foo|bar|baz";
        let expected = Segment {
            name: Cow::Borrowed("MSH"),
            fields: vec![
                Field {
                    value: Cow::Borrowed("foo"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("bar"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("baz"),
                    repeats: vec![],
                    components: vec![],
                },
            ],
        };
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_segment_with_number() {
        let separators = Separators::default();

        let input = "DB1|foo|bar|baz";
        let expected = Segment {
            name: Cow::Borrowed("DB1"),
            fields: vec![
                Field {
                    value: Cow::Borrowed("foo"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("bar"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("baz"),
                    repeats: vec![],
                    components: vec![],
                },
            ],
        };
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn cant_parse_segments_with_invalid_names() {
        let separators = Separators::default();

        let input = "MS|foo|bar|baz";
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());

        let input = "MSAX|foo|bar|baz";
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn cant_parse_segments_with_no_names() {
        let separators = Separators::default();

        let input = "|foo|bar|baz";
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn cant_parse_segments_with_no_fields() {
        let separators = Separators::default();

        let input = "MSH";
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn can_parse_segment_with_repeats_components_and_subcomponents() {
        let separators = Separators::default();

        let input = r"MSH|foo^b\~ar^baz&x~qux^quux^quuz|asdf|";
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(actual.name, "MSH");
        assert_eq!(actual.fields.len(), 3);
        assert_eq!(actual.fields[0].value, r"foo^b\~ar^baz&x");
        assert_eq!(actual.fields[0].repeats.len(), 1);
        assert_eq!(actual.fields[0].repeats[0].value, r"qux^quux^quuz");
        assert_eq!(actual.fields[0].repeats[0].components.len(), 3);
        assert_eq!(actual.fields[0].repeats[0].components[0].value, "qux");
        assert_eq!(actual.fields[0].repeats[0].components[1].value, "quux");
        assert_eq!(actual.fields[0].repeats[0].components[2].value, "quuz");
        assert_eq!(
            actual.fields[0].repeats[0].components[0]
                .subcomponents
                .len(),
            0
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[1]
                .subcomponents
                .len(),
            0
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[2]
                .subcomponents
                .len(),
            0
        );
        assert_eq!(actual.fields[1].value, "asdf");
        assert_eq!(actual.fields[2].value, "");
    }
}
