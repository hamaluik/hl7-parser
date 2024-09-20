use std::borrow::Cow;

use super::component::component;
use crate::message::{Repeat, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn repeat<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Repeat<'i>> {
    move |i| parse_repeat(i, seps)
}

fn parse_repeat<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Repeat<'i>> {
    let (i, (subc_src, v)) = consumed(separated_list0(char(seps.component), component(seps)))(i)?;

    let v = if v.len() == 1 {
        let mut v = v;
        Repeat {
            value: v.remove(0).value,
            components: vec![],
        }
    } else {
        Repeat {
            value: Cow::Borrowed(subc_src),
            components: v,
        }
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use crate::message::Component;
    use super::*;

    #[test]
    fn can_parse_repeat_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Repeat {
            value: Cow::Borrowed("foo"),
            components: vec![],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo^bar";
        let expected = Repeat {
            value: Cow::Borrowed("foo^bar"),
            components: vec![
                Component::Value(Cow::Borrowed("foo")),
                Component::Value(Cow::Borrowed("bar")),
            ],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_repeat_with_no_components_and_escaped_component_separator() {
        let separators = Separators::default();

        let input = r"foo\^bar";
        let expected = Repeat {
            value: Cow::Borrowed(r"foo\^bar"),
            components: vec![],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
