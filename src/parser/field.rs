use super::repeat::repeat;
use crate::{Field, Repeat, Separators};
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alpha1, char, none_of, one_of},
    combinator::consumed,
    multi::separated_list0,
    sequence::terminated,
    IResult,
};

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
            value: subc_src,
            repeats: v,
            components: vec![],
        }
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, Repeat};
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_field_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Field {
            value: "foo",
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
            value: "foo^bar^baz",
            repeats: vec![],
            components: vec![
                Component {
                    value: "foo",
                    subcomponents: vec![],
                },
                Component {
                    value: "bar",
                    subcomponents: vec![],
                },
                Component {
                    value: "baz",
                    subcomponents: vec![],
                },
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
            value: "foo",
            repeats: vec![
                Repeat {
                    value: "bar",
                    components: vec![],
                },
                Repeat {
                    value: "baz",
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
            value: "foo^bar",
            repeats: vec![Repeat {
                value: "baz^qux",
                components: vec![
                    Component {
                        value: "baz",
                        subcomponents: vec![],
                    },
                    Component {
                        value: "qux",
                        subcomponents: vec![],
                    },
                ],
            }],
            components: vec![
                Component {
                    value: "foo",
                    subcomponents: vec![],
                },
                Component {
                    value: "bar",
                    subcomponents: vec![],
                },
            ],
        };
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
