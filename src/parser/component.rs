use std::borrow::Cow;

use super::subcomponent::subcomponent;
use crate::{Component, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn component<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Component<'i>> {
    move |i| parse_component(i, seps)
}

fn parse_component<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Component<'i>> {
    let (i, (subc_src, v)) =
        consumed(separated_list0(char(seps.subcomponent), subcomponent(seps)))(i)?;

    let v = if v.len() == 1 {
        let mut v = v;
        Component {
            value: v.remove(0).value,
            subcomponents: vec![],
        }
    } else {
        Component {
            value: Cow::Borrowed(subc_src),
            subcomponents: v,
        }
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use crate::Subcomponent;

    use super::*;

    #[test]
    fn can_parse_component_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Component {
            value: Cow::Borrowed("foo"),
            subcomponents: vec![],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo&bar";
        let expected = Component {
            value: Cow::Borrowed("foo&bar"),
            subcomponents: vec![
                Subcomponent {
                    value: Cow::Borrowed("foo"),
                },
                Subcomponent {
                    value: Cow::Borrowed("bar"),
                },
            ],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_no_subcomponents_and_escaped_subcomponent_separator() {
        let separators = Separators::default();

        let input = r"foo\&bar";
        let expected = Component {
            value: Cow::Borrowed(r"foo\&bar"),
            subcomponents: vec![],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_at_end_of_line() {
        let separators = Separators::default();

        let input = "foo\rbar";
        let expected = Component {
            value: Cow::Borrowed("foo"),
            subcomponents: vec![],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_in_field() {
        let separators = Separators::default();

        let input = "foo|bar";
        let expected = Component {
            value: Cow::Borrowed("foo"),
            subcomponents: vec![],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);

        let input = "foo&bar|baz";
        let expected = Component {
            value: Cow::Borrowed("foo&bar"),
            subcomponents: vec![
                Subcomponent {
                    value: Cow::Borrowed("foo"),
                },
                Subcomponent {
                    value: Cow::Borrowed("bar"),
                },
            ],
        };
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
