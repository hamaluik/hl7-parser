use super::subcomponent::subcomponent;
use crate::message::{Component, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn component<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Component<'i>> {
    move |i| parse_component(i, seps)
}

fn parse_component<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Component<'i>> {
    let (i, (_subc_src, v)) =
        consumed(separated_list0(char(seps.subcomponent), subcomponent(seps)))(i)?;

    let v = if v.len() == 1 {
        let mut v = v;
        Component::Value(v.remove(0).0)
    } else {
        Component::Subcomponents(v)
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Component, Subcomponent};
    use std::borrow::Cow;

    #[test]
    fn can_parse_component_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Component::Value(Cow::Borrowed("foo"));
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo&bar";
        let expected = Component::Subcomponents(vec![
            Subcomponent(Cow::Borrowed("foo")),
            Subcomponent(Cow::Borrowed("bar")),
        ]);
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_no_subcomponents_and_escaped_subcomponent_separator() {
        let separators = Separators::default();

        let input = r"foo\&bar";
        let expected = Component::Value(Cow::Borrowed(r"foo\&bar"));
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_at_end_of_line() {
        let separators = Separators::default();

        let input = "foo\rbar";
        let expected = Component::Value(Cow::Borrowed("foo"));
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_in_field() {
        let separators = Separators::default();

        let input = "foo|bar";
        let expected = Component::Value(Cow::Borrowed("foo"));
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);

        let input = "foo&bar|baz";
        let expected = Component::Subcomponents(vec![
            Subcomponent(Cow::Borrowed("foo")),
            Subcomponent(Cow::Borrowed("bar")),
        ]);
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
