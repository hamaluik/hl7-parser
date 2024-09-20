use std::borrow::Cow;

use super::repeat::repeat;
use crate::message::{Field, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn field<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Field<'i>> {
    move |i| parse_field(i, seps)
}

fn parse_field<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Field<'i>> {
    let (i, (subc_src, v)) = consumed(separated_list0(char(seps.repetition), repeat(seps)))(i)?;

    let v = if !v.is_empty() {
        let mut v = v;
        let first = v.remove(0);
        Field {
            value: first.value,
            repeats: v,
            components: first.components,
        }
    } else {
        Field {
            value: Cow::Borrowed(subc_src),
            repeats: v,
            components: vec![],
        }
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Component, Repeat};
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_field_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Field {
            value: Cow::Borrowed("foo"),
            repeats: vec![],
            components: vec![],
        };
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_field_without_repeats_with_components() {
        let separators = Separators::default();

        let input = "foo^bar^baz";
        let expected = Field {
            value: Cow::Borrowed("foo^bar^baz"),
            repeats: vec![],
            components: vec![
                Component::Value(Cow::Borrowed("foo")),
                Component::Value(Cow::Borrowed("bar")),
                Component::Value(Cow::Borrowed("baz")),
            ],
        };
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_field_with_basic_repeats() {
        let separators = Separators::default();

        let input = "foo~bar~baz";
        let expected = Field {
            value: Cow::Borrowed("foo"),
            repeats: vec![
                Repeat {
                    value: Cow::Borrowed("bar"),
                    components: vec![],
                },
                Repeat {
                    value: Cow::Borrowed("baz"),
                    components: vec![],
                },
            ],
            components: vec![],
        };
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_field_with_repeats_and_components() {
        let separators = Separators::default();

        let input = "foo^bar~baz^qux";
        let expected = Field {
            value: Cow::Borrowed("foo^bar"),
            repeats: vec![Repeat {
                value: Cow::Borrowed("baz^qux"),
                components: vec![
                    Component::Value(Cow::Borrowed("baz")),
                    Component::Value(Cow::Borrowed("qux")),
                ],
            }],
            components: vec![
                Component::Value(Cow::Borrowed("foo")),
                Component::Value(Cow::Borrowed("bar")),
            ],
        };
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
