use super::component::component;
use crate::message::{Component, Repeat, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn repeat<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Repeat<'i>> {
    move |i| parse_repeat(i, seps)
}

fn parse_repeat<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Repeat<'i>> {
    let (i, (_subc_src, v)) = consumed(separated_list0(char(seps.component), component(seps)))(i)?;

    let v = if v.len() == 1 {
        let mut v = v;
        if let Component::Value(v) = v.remove(0) {
            Repeat::Value(v)
        } else {
            Repeat::Component(v.remove(0))
        }
    } else {
        Repeat::Components(v)
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::message::{Component, Subcomponent};
    use super::*;

    #[test]
    fn can_parse_repeat_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Repeat::Value(Cow::Borrowed("foo"));
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo^bar";
        let expected = Repeat::Components(vec![
            Component::Value(Cow::Borrowed("foo")),
            Component::Value(Cow::Borrowed("bar")),
        ]);
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_repeat_with_no_components_and_escaped_component_separator() {
        let separators = Separators::default();

        let input = r"foo\^bar";
        let expected = Repeat::Value(Cow::Borrowed(r"foo\^bar"));
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_repeat_with_single_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo^bar&baz";
        let expected = Repeat::Component(Component::Subcomponents(vec![
            Subcomponent(Cow::Borrowed("foo")),
            Subcomponent(Cow::Borrowed("bar")),
            Subcomponent(Cow::Borrowed("baz")),
        ]));
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
